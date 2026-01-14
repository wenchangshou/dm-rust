use crate::protocols::Protocol;
use crate::protocols::storage::get_or_init_storage;
use crate::utils::{DeviceError, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use mac_address::MacAddress;
use tracing::{debug, error, info, warn};

struct XFusionNode {
    id: u32,
    mac_text: String,
    mac_bytes: [u8; 6],
    ip: Option<Ipv4Addr>,
    port: Option<u16>,
    last_heartbeat: Option<Instant>,
    // iBMC Redfish API 配置
    ibmc_url: String,
    ibmc_username: String,
    ibmc_password: String,
    system_id: String,
}

#[derive(Deserialize)]
struct XFusionConfigItem {
    id: u32,
    mac: String,
    ip: Option<String>,
    port: Option<u16>,
    ibmc_url: String,
    ibmc_username: String,
    ibmc_password: String,
    #[serde(default = "default_system_id")]
    system_id: String,
}

fn default_system_id() -> String {
    "1".to_string()
}

/// xFusion 服务器控制协议（iBMC Redfish API + 状态监控）
pub struct XFusionProtocol {
    channel_id: u32,
    nodes: Vec<XFusionNode>,
    broadcast_addr: Ipv4Addr,
    wol_port: u16,
    shutdown_port: u16,
    http_client: reqwest::Client,
    /// 缓存的会话 Token (node_id -> token) - 内存缓存
    token_cache: Arc<RwLock<HashMap<u32, String>>>,
}


impl XFusionProtocol {
    /// 获取 Token 的存储键
    fn token_storage_key(node_id: u32) -> String {
        format!("token_{}", node_id)
    }

    /// 获取缓存的 Token，如果不存在则返回 None
    /// 优先从内存缓存获取，其次从持久化存储获取
    async fn get_cached_token(&self, node_id: u32) -> Option<String> {
        // 先从内存缓存获取
        {
            let cache = self.token_cache.read().await;
            if let Some(token) = cache.get(&node_id).cloned() {
                return Some(token);
            }
        }

        // 从持久化存储获取
        let storage = get_or_init_storage().await;
        if let Some(token) = storage.get_string(self.channel_id, &Self::token_storage_key(node_id)).await {
            // 同步到内存缓存
            let mut cache = self.token_cache.write().await;
            cache.insert(node_id, token.clone());
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 从持久化存储恢复 Token",
                self.channel_id, node_id
            );
            return Some(token);
        }

        None
    }

    /// 缓存 Token（同时保存到内存和持久化存储）
    async fn cache_token(&self, node_id: u32, token: String) {
        // 保存到内存缓存
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(node_id, token.clone());
        }

        // 保存到持久化存储
        let storage = get_or_init_storage().await;
        storage.set(
            self.channel_id,
            &Self::token_storage_key(node_id),
            serde_json::json!(token),
        ).await;

        debug!(
            "通道 {} [xFusion]: 节点 ID:{} Token 已缓存到内存和持久化存储",
            self.channel_id, node_id
        );
    }

    /// 清除缓存的 Token（同时从内存和持久化存储删除）
    async fn invalidate_token(&self, node_id: u32) {
        // 从内存缓存删除
        {
            let mut cache = self.token_cache.write().await;
            cache.remove(&node_id);
        }

        // 从持久化存储删除
        let storage = get_or_init_storage().await;
        storage.remove(self.channel_id, &Self::token_storage_key(node_id)).await;

        debug!(
            "通道 {} [xFusion]: 节点 ID:{} Token 已失效，已从缓存中移除",
            self.channel_id, node_id
        );
    }


    /// 创建新的会话 Token
    async fn create_session_token(&self, node: &XFusionNode) -> Result<String> {
        let session_url = format!("{}/redfish/v1/SessionService/Sessions", node.ibmc_url);

        info!(
            "通道 {} [xFusion]: 节点 ID:{} 正在创建新的 iBMC 会话 Token",
            self.channel_id, node.id
        );
        debug!(
            "通道 {} [xFusion]: 会话请求 URL: {}, 用户名: {}",
            self.channel_id, session_url, node.ibmc_username
        );

        let body = serde_json::json!({
            "UserName": node.ibmc_username,
            "Password": node.ibmc_password
        });

        debug!(
            "通道 {} [xFusion]: 发送会话请求, Body: {{\"UserName\": \"{}\", \"Password\": \"***\"}}",
            self.channel_id, node.ibmc_username
        );

        let start_time = std::time::Instant::now();
        let response = self
            .http_client
            .post(&session_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!(
                    "通道 {} [xFusion]: 节点 ID:{} 获取会话失败: {}, 耗时: {:?}",
                    self.channel_id, node.id, e, start_time.elapsed()
                );
                DeviceError::ProtocolError(format!("iBMC 会话请求失败: {}", e))
            })?;

        let elapsed = start_time.elapsed();
        let status = response.status();
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 会话响应状态: {}, 耗时: {:?}",
            self.channel_id, node.id, status, elapsed
        );

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!(
                "通道 {} [xFusion]: 节点 ID:{} 会话请求返回错误状态: {}, 响应: {}",
                self.channel_id, node.id, status, text
            );
            return Err(DeviceError::ProtocolError(format!(
                "iBMC 会话请求返回错误: {}",
                status
            )));
        }

        let json: Value = response.json().await.map_err(|e| {
            error!(
                "通道 {} [xFusion]: 节点 ID:{} 解析会话响应失败: {}",
                self.channel_id, node.id, e
            );
            DeviceError::ProtocolError(format!("解析会话响应失败: {}", e))
        })?;

        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 会话响应 JSON: {}",
            self.channel_id, node.id, json
        );

        // 从 Oem.xFusion.X-Auth-Token 获取 Token
        let token = json
            .get("Oem")
            .and_then(|oem| oem.get("xFusion"))
            .and_then(|xf| xf.get("X-Auth-Token"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                error!(
                    "通道 {} [xFusion]: 节点 ID:{} 会话响应中未找到 Token, JSON: {}",
                    self.channel_id, node.id, json
                );
                DeviceError::ProtocolError("会话响应中未找到 X-Auth-Token".into())
            })?;

        info!(
            "通道 {} [xFusion]: 节点 ID:{} 成功创建会话 Token: {}..., 耗时: {:?}",
            self.channel_id,
            node.id,
            &token[..8.min(token.len())],
            elapsed
        );

        let token_string = token.to_string();
        // 缓存 Token
        self.cache_token(node.id, token_string.clone()).await;

        Ok(token_string)
    }

    /// 获取会话 Token（优先使用缓存）
    async fn get_session_token(&self, node: &XFusionNode) -> Result<String> {
        // 先尝试从缓存获取
        if let Some(cached_token) = self.get_cached_token(node.id).await {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 使用缓存的 Token: {}...",
                self.channel_id,
                node.id,
                &cached_token[..8.min(cached_token.len())]
            );
            return Ok(cached_token);
        }

        // 缓存中没有，创建新的
        self.create_session_token(node).await
    }

    /// 检查响应是否表示会话已过期
    fn is_session_expired(&self, response_text: &str) -> bool {
        // 检查是否包含 NoValidSession 错误
        if response_text.contains("NoValidSession") {
            return true;
        }
        // 检查其他可能的会话过期标志
        if response_text.contains("no valid session") {
            return true;
        }
        false
    }

    /// 执行电源操作
    async fn power_action(&self, node: &XFusionNode, reset_type: &str) -> Result<()> {
        info!(
            "通道 {} [xFusion]: 节点 ID:{} 开始执行电源操作 '{}'",
            self.channel_id, node.id, reset_type
        );
        debug!(
            "通道 {} [xFusion]: 节点信息 - MAC: {}, iBMC: {}, SystemID: {}",
            self.channel_id, node.mac_text, node.ibmc_url, node.system_id
        );

        let reset_url = format!(
            "{}/redfish/v1/Systems/{}/Actions/ComputerSystem.Reset",
            node.ibmc_url, node.system_id
        );

        let body = serde_json::json!({
            "ResetType": reset_type
        });

        // 尝试使用缓存的 Token，如果失败则刷新后重试
        for attempt in 0..2 {
            let token = if attempt == 0 {
                self.get_session_token(node).await?
            } else {
                // 第二次尝试，强制创建新 Token
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} Token 可能已过期，正在刷新",
                    self.channel_id, node.id
                );
                self.invalidate_token(node.id).await;
                self.create_session_token(node).await?
            };

            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 电源操作 URL: {}, 尝试次数: {}",
                self.channel_id, node.id, reset_url, attempt + 1
            );

            let start_time = std::time::Instant::now();
            let response = self
                .http_client
                .post(&reset_url)
                .header("X-Auth-Token", &token)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    error!(
                        "通道 {} [xFusion]: 节点 ID:{} 电源操作请求失败: {}, 耗时: {:?}",
                        self.channel_id, node.id, e, start_time.elapsed()
                    );
                    DeviceError::ProtocolError(format!("iBMC 电源操作请求失败: {}", e))
                })?;

            let elapsed = start_time.elapsed();
            let status = response.status();
            
            // 先获取响应文本
            let response_text = response.text().await.unwrap_or_default();
            
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 电源操作响应状态: {}, 耗时: {:?}",
                self.channel_id, node.id, status, elapsed
            );

            // 检查是否是会话过期（401 或响应体包含 NoValidSession）
            if (status.as_u16() == 401 || self.is_session_expired(&response_text)) && attempt == 0 {
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} 会话已过期，将刷新后重试",
                    self.channel_id, node.id
                );
                continue;
            }

            if !status.is_success() || response_text.contains("error") {
                error!(
                    "通道 {} [xFusion]: 节点 ID:{} 电源操作 '{}' 返回错误状态: {}, 响应: {}",
                    self.channel_id, node.id, reset_type, status, response_text
                );
                return Err(DeviceError::ProtocolError(format!(
                    "iBMC 电源操作返回错误: {} - {}",
                    status, response_text
                )));
            }

            info!(
                "通道 {} [xFusion]: 节点 ID:{} 电源命令 '{}' 执行成功, 耗时: {:?}",
                self.channel_id, node.id, reset_type, elapsed
            );

            return Ok(());
        }

        Err(DeviceError::ProtocolError("电源操作重试次数已用尽".into()))
    }


    /// 开机 (使用 iBMC Redfish API)
    async fn power_on(&self, node: &XFusionNode) -> Result<()> {
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 调用开机 (On)",
            self.channel_id, node.id
        );
        self.power_action(node, "On").await
    }

    /// 关机 (使用 iBMC Redfish API)
    async fn power_off(&self, node: &XFusionNode) -> Result<()> {
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 调用关机 (GracefulShutdown)",
            self.channel_id, node.id
        );
        self.power_action(node, "GracefulShutdown").await
    }

    /// 强制关机
    async fn force_off(&self, node: &XFusionNode) -> Result<()> {
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 调用强制关机 (ForceOff)",
            self.channel_id, node.id
        );
        self.power_action(node, "ForceOff").await
    }

    /// 强制重启
    async fn force_restart(&self, node: &XFusionNode) -> Result<()> {
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 调用强制重启 (ForceRestart)",
            self.channel_id, node.id
        );
        self.power_action(node, "ForceRestart").await
    }

    /// 强制下电再上电
    async fn force_power_cycle(&self, node: &XFusionNode) -> Result<()> {
        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 调用强制下电再上电 (ForcePowerCycle)",
            self.channel_id, node.id
        );
        self.power_action(node, "ForcePowerCycle").await
    }

    async fn send_udp(
        &self,
        ip: Ipv4Addr,
        port: u16,
        command: &str,
        wait_response: bool,
    ) -> Result<Option<String>> {
        debug!(
            "通道 {} [UDP]: 发送命令 '{}' 到 {}:{}",
            self.channel_id, command, ip, port
        );
        let socket = tokio::net::UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))
            .await
            .map_err(|e| DeviceError::ProtocolError(format!("绑定 UDP socket 失败: {}", e)))?;

        let to = std::net::SocketAddrV4::new(ip, port);
        socket
            .send_to(command.as_bytes(), to)
            .await
            .map_err(|e| DeviceError::ProtocolError(format!("发送 UDP 命令失败: {}", e)))?;

        if wait_response {
            let mut buf = [0u8; 64];
            let timeout = Duration::from_millis(500);
            match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await {
                Ok(Ok((n, _))) => {
                    let response = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                    debug!(
                        "通道 {} [UDP]: 收到来自 {}:{} 的响应: {}",
                        self.channel_id, ip, port, response
                    );
                    Ok(Some(response))
                }
                _ => {
                    debug!(
                        "通道 {} [UDP]: 等待 {}:{} 响应超时",
                        self.channel_id, ip, port
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn ping_node(&self, node: &XFusionNode) -> bool {
        if let (Some(ip), Some(port)) = (node.ip, node.port) {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 正在 ping {}:{}",
                self.channel_id, node.id, ip, port
            );
            if let Ok(Some(resp)) = self.send_udp(ip, port, "ping", true).await {
                let is_pong = resp.eq_ignore_ascii_case("pong");
                debug!(
                    "通道 {} [xFusion]: 节点 ID:{} ping 响应: '{}', 结果: {}",
                    self.channel_id, node.id, resp, if is_pong { "在线" } else { "无效响应" }
                );
                return is_pong;
            }
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} ping 无响应",
                self.channel_id, node.id
            );
        } else {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 未配置 IP/端口, 跳过 ping",
                self.channel_id, node.id
            );
        }
        false
    }

    fn update_heartbeat(&mut self, mac: &str) -> bool {
        for node in &mut self.nodes {
            if node.mac_text.eq_ignore_ascii_case(mac) {
                debug!(
                    "通道 {} [Heartbeat]: 更新节点 ID:{} (MAC:{}) 的心跳",
                    self.channel_id, node.id, mac
                );
                node.last_heartbeat = Some(Instant::now());
                return true;
            }
        }
        warn!(
            "通道 {} [Heartbeat]: 收到未知 MAC 地址的心跳: {}",
            self.channel_id, mac
        );
        false
    }

    fn find_node_by_id(&self, id: u32) -> Option<&XFusionNode> {
        self.nodes.iter().find(|c| c.id == id)
    }

    /// 通过 iBMC Redfish API 获取电源状态
    async fn get_power_state(&self, node: &XFusionNode) -> Result<String> {
        let system_url = format!(
            "{}/redfish/v1/Systems/{}",
            node.ibmc_url, node.system_id
        );

        // 尝试使用缓存的 Token，如果失败则刷新后重试
        for attempt in 0..2 {
            let token = if attempt == 0 {
                self.get_session_token(node).await?
            } else {
                // 第二次尝试，强制创建新 Token
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} Token 可能已过期，正在刷新",
                    self.channel_id, node.id
                );
                self.invalidate_token(node.id).await;
                self.create_session_token(node).await?
            };

            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 查询电源状态, URL: {}, 尝试次数: {}",
                self.channel_id, node.id, system_url, attempt + 1
            );

            let start_time = std::time::Instant::now();
            let response = self
                .http_client
                .get(&system_url)
                .header("X-Auth-Token", &token)
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(|e| {
                    error!(
                        "通道 {} [xFusion]: 节点 ID:{} 查询电源状态失败: {}, 耗时: {:?}",
                        self.channel_id, node.id, e, start_time.elapsed()
                    );
                    DeviceError::ProtocolError(format!("iBMC 查询电源状态失败: {}", e))
                })?;

            let elapsed = start_time.elapsed();
            let status = response.status();
            
            // 先获取响应文本
            let response_text = response.text().await.unwrap_or_default();

            // 检查是否是会话过期（401 或响应体包含 NoValidSession）
            if (status.as_u16() == 401 || self.is_session_expired(&response_text)) && attempt == 0 {
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} 会话已过期，将刷新后重试",
                    self.channel_id, node.id
                );
                continue;
            }

            if !status.is_success() {
                error!(
                    "通道 {} [xFusion]: 节点 ID:{} 查询电源状态返回错误: {}, 响应: {}",
                    self.channel_id, node.id, status, response_text
                );
                return Err(DeviceError::ProtocolError(format!(
                    "iBMC 查询电源状态返回错误: {}",
                    status
                )));
            }

            // 检查响应中是否有 error 字段
            let json: Value = serde_json::from_str(&response_text).map_err(|e| {
                error!(
                    "通道 {} [xFusion]: 节点 ID:{} 解析电源状态响应失败: {}",
                    self.channel_id, node.id, e
                );
                DeviceError::ProtocolError(format!("解析电源状态响应失败: {}", e))
            })?;

            // 检查是否有错误
            if json.get("error").is_some() && attempt == 0 {
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} 响应包含错误，可能会话过期，将刷新后重试",
                    self.channel_id, node.id
                );
                continue;
            }

            // 提取 PowerState 字段
            let power_state = json
                .get("PowerState")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            info!(
                "通道 {} [xFusion]: 节点 ID:{} 电源状态: {}, 耗时: {:?}",
                self.channel_id, node.id, power_state, elapsed
            );

            return Ok(power_state);
        }

        Err(DeviceError::ProtocolError("查询电源状态重试次数已用尽".into()))
    }



    /// 检查节点是否开机 (优先使用 iBMC API)
    async fn is_node_powered_on(&self, node: &XFusionNode) -> bool {
        match self.get_power_state(node).await {
            Ok(state) => {
                let is_on = state.eq_ignore_ascii_case("On");
                debug!(
                    "通道 {} [xFusion]: 节点 ID:{} iBMC 电源状态: {}, 开机: {}",
                    self.channel_id, node.id, state, is_on
                );
                is_on
            }
            Err(e) => {
                warn!(
                    "通道 {} [xFusion]: 节点 ID:{} 获取 iBMC 电源状态失败: {}, 回退到 ping 检测",
                    self.channel_id, node.id, e
                );
                // 回退到 ping 检测
                self.ping_node(node).await
            }
        }
    }

    async fn is_node_online(&self, node: &XFusionNode) -> bool {
        debug!(
            "通道 {} [xFusion]: 检查节点 ID:{} 在线状态",
            self.channel_id, node.id
        );

        // 优先使用 iBMC API 检测电源状态
        if self.is_node_powered_on(node).await {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 通过 iBMC API 检测开机",
                self.channel_id, node.id
            );
            return true;
        }

        // 回退到 ping 检测
        if self.ping_node(node).await {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 通过 ping 检测在线",
                self.channel_id, node.id
            );
            return true;
        }

        // 最后检查心跳
        if let Some(last) = node.last_heartbeat {
            let elapsed = Instant::now().duration_since(last);
            let is_online = elapsed < Duration::from_secs(10);
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 上次心跳距今: {:?}, 在线: {}",
                self.channel_id, node.id, elapsed, is_online
            );
            return is_online;
        }

        debug!(
            "通道 {} [xFusion]: 节点 ID:{} 所有检测方式均失败, 离线",
            self.channel_id, node.id
        );
        false
    }

    async fn get_audio_status(&self, node: &XFusionNode) -> (Option<i32>, Option<bool>, bool) {
        debug!(
            "通道 {} [xFusion]: 获取节点 ID:{} 音频状态",
            self.channel_id, node.id
        );
        let is_online = self.is_node_online(node).await;
        if !is_online {
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 离线, 跳过音频状态获取",
                self.channel_id, node.id
            );
            return (None, None, false);
        }

        if let (Some(ip), Some(port)) = (node.ip, node.port) {
            match self.send_udp(ip, port, "get", true).await {
                Ok(Some(resp)) => {
                    debug!(
                        "通道 {} [xFusion]: 节点 ID:{} 音频状态响应: {}",
                        self.channel_id, node.id, resp
                    );
                    let mut volume = None;
                    let mut mute = None;

                    for part in resp.split(',') {
                        let kv: Vec<&str> = part.split(':').collect();
                        if kv.len() == 2 {
                            let key = kv[0].trim().to_lowercase();
                            let value = kv[1].trim();
                            if key == "volume" {
                                volume = value.parse::<i32>().ok();
                            } else if key == "mute" {
                                mute = value.parse::<bool>().ok();
                            }
                        }
                    }
                    debug!(
                        "通道 {} [xFusion]: 节点 ID:{} 解析结果 - volume: {:?}, mute: {:?}",
                        self.channel_id, node.id, volume, mute
                    );
                    (volume, mute, true)
                }
                _ => {
                    debug!(
                        "通道 {} [xFusion]: 节点 ID:{} 音频状态无响应",
                        self.channel_id, node.id
                    );
                    (None, None, true)
                }
            }
        } else {
            (None, None, is_online)
        }
    }
}

#[async_trait]
impl Protocol for XFusionProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        let node_list_json = params.get("nodes").or_else(|| params.get("mac_address")).ok_or_else(|| {
            DeviceError::ConfigError("xFusion 缺少 nodes 参数".into())
        })?;

        let config_items: Vec<XFusionConfigItem> =
            serde_json::from_value(node_list_json.clone())
                .map_err(|e| DeviceError::ConfigError(format!("nodes 解析失败: {}", e)))?;

        let mut nodes = Vec::new();
        for item in config_items {
            let mac_bytes = item
                .mac
                .parse::<MacAddress>()
                .map_err(|e| {
                    DeviceError::ConfigError(format!("MAC地址格式错误 {}: {}", item.mac, e))
                })?
                .bytes();

            let ip = item.ip.as_ref().and_then(|s| s.parse::<Ipv4Addr>().ok());

            nodes.push(XFusionNode {
                id: item.id,
                mac_text: item.mac,
                mac_bytes,
                ip,
                port: item.port,
                last_heartbeat: None,
                ibmc_url: item.ibmc_url,
                ibmc_username: item.ibmc_username,
                ibmc_password: item.ibmc_password,
                system_id: item.system_id,
            });
        }

        let broadcast_addr = params
            .get("broadcast_addr")
            .or_else(|| params.get("broadcast"))
            .and_then(|v| v.as_str())
            .map(|s| {
                s.parse::<Ipv4Addr>().map_err(|e| {
                    DeviceError::ConfigError(format!(
                        "xFusion broadcast_addr 解析失败: {} (值: {})",
                        e, s
                    ))
                })
            })
            .transpose()?
            .unwrap_or_else(|| Ipv4Addr::new(255, 255, 255, 255));

        let wol_port = params
            .get("wol_port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16)
            .unwrap_or(9);

        let shutdown_port = params
            .get("shutdown_port")
            .and_then(|v| v.as_u64())
            .map(|p| p as u16)
            .unwrap_or(wol_port);

        // 创建 HTTP 客户端，忽略 SSL 证书验证（iBMC 通常使用自签名证书）
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| DeviceError::ConfigError(format!("创建 HTTP 客户端失败: {}", e)))?;

        info!(
            "通道 {} [Config]: 初始化 XFusionProtocol, 包含 {} 个节点",
            channel_id, nodes.len()
        );
        for node in &nodes {
            debug!(
                "通道 {} [Config]: 节点 ID:{}, MAC:{}, IP:{:?}, Port:{:?}, iBMC:{}",
                channel_id, node.id, node.mac_text, node.ip, node.port, node.ibmc_url
            );
        }

        Ok(Box::new(Self {
            channel_id,
            nodes,
            broadcast_addr,
            wol_port,
            shutdown_port,
            http_client,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }))

    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        debug!(
            "通道 {} [Execute]: 收到命令 '{}', 参数: {}",
            self.channel_id, command, params
        );
        match command {
            "powerOn" | "on" => {
                let id = params.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);
                let mac = params.get("mac").and_then(|v| v.as_str());

                let mut targets = Vec::new();

                if let Some(id) = id {
                    if let Some(node) = self.find_node_by_id(id) {
                        targets.push(node);
                    }
                } else if let Some(mac) = mac {
                    if let Some(node) = self
                        .nodes
                        .iter()
                        .find(|c| c.mac_text.eq_ignore_ascii_case(mac))
                    {
                        targets.push(node);
                    }
                }

                if targets.is_empty() {
                    warn!(
                        "通道 {} [Execute]: {} 命令未找到匹配的节点, 参数: {}",
                        self.channel_id, command, params
                    );
                    return Err(DeviceError::ProtocolError(
                        "powerOn 需要指定 id 或 mac".into(),
                    ));
                }

                for node in targets {
                    self.power_on(node).await?;
                }

                Ok(serde_json::json!({ "status": "ok", "action": "powerOn" }))
            }
            "powerOff" | "shutdown" => {
                let id = params.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);
                let mac = params.get("mac").and_then(|v| v.as_str());

                let mut targets = Vec::new();
                if let Some(id) = id {
                    if let Some(node) = self.find_node_by_id(id) {
                        targets.push(node);
                    }
                } else if let Some(mac) = mac {
                    if let Some(node) = self
                        .nodes
                        .iter()
                        .find(|c| c.mac_text.eq_ignore_ascii_case(mac))
                    {
                        targets.push(node);
                    }
                }

                if targets.is_empty() {
                    warn!(
                        "通道 {} [Execute]: {} 命令未找到匹配的节点, 参数: {}",
                        self.channel_id, command, params
                    );
                    return Err(DeviceError::ProtocolError(
                        "powerOff 需要指定 id 或 mac".into(),
                    ));
                }

                for node in targets {
                    self.power_off(node).await?;
                }

                Ok(serde_json::json!({ "status": "ok", "action": "powerOff" }))
            }
            "forceOff" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| DeviceError::ProtocolError("forceOff 需要 id 参数".into()))?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                self.force_off(node).await?;
                Ok(serde_json::json!({ "status": "ok", "action": "forceOff" }))
            }
            "forceRestart" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| {
                        DeviceError::ProtocolError("forceRestart 需要 id 参数".into())
                    })?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                self.force_restart(node).await?;
                Ok(serde_json::json!({ "status": "ok", "action": "forceRestart" }))
            }
            "forcePowerCycle" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| {
                        DeviceError::ProtocolError("forcePowerCycle 需要 id 参数".into())
                    })?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                self.force_power_cycle(node).await?;
                Ok(serde_json::json!({ "status": "ok", "action": "forcePowerCycle" }))
            }
            "reset" => {
                // 通用重置命令，支持所有 ResetType
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| DeviceError::ProtocolError("reset 需要 id 参数".into()))?;

                let reset_type = params
                    .get("resetType")
                    .or_else(|| params.get("type"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        DeviceError::ProtocolError("reset 需要 resetType 参数".into())
                    })?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                self.power_action(node, reset_type).await?;
                Ok(serde_json::json!({ "status": "ok", "action": "reset", "resetType": reset_type }))
            }
            "heartbeat" => {
                let mac = params.get("mac").and_then(|v| v.as_str()).ok_or_else(|| {
                    DeviceError::ProtocolError("heartbeat 命令需要 mac 参数".into())
                })?;

                if self.update_heartbeat(mac) {
                    Ok(serde_json::json!({ "status": "ok", "msg": "heartbeat updated" }))
                } else {
                    Err(DeviceError::ProtocolError(format!(
                        "heartbeat: 未找到 mac 为 {} 的设备",
                        mac
                    )))
                }
            }
            "get" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| DeviceError::ProtocolError("get 命令需要 id 参数".into()))?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                // 通过 iBMC API 获取电源状态
                let power_state = self.get_power_state(node).await.ok();
                let is_powered_on = power_state.as_ref().map(|s| s.eq_ignore_ascii_case("On")).unwrap_or(false);

                let (volume, mute, _) = self.get_audio_status(node).await;
                Ok(serde_json::json!({
                    "id": id,
                    "online": is_powered_on,
                    "powerState": power_state,
                    "volume": volume,
                    "mute": mute,
                }))
            }
            "getPowerState" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| DeviceError::ProtocolError("getPowerState 命令需要 id 参数".into()))?;

                let node = self.find_node_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
                })?;

                let power_state = self.get_power_state(node).await?;
                Ok(serde_json::json!({
                    "id": id,
                    "powerState": power_state,
                    "isPoweredOn": power_state.eq_ignore_ascii_case("On"),
                }))
            }
            "getAllStatus" => self.get_status().await,

            _ => {
                warn!(
                    "通道 {} [Execute]: 未知命令: {}",
                    self.channel_id, command
                );
                Err(DeviceError::ProtocolError(format!("未知命令: {}", command)))
            }
        }
    }

    async fn get_status(&self) -> Result<Value> {
        debug!(
            "通道 {} [xFusion]: 获取所有节点状态, 共 {} 个节点",
            self.channel_id, self.nodes.len()
        );
        let mut status_list = Vec::new();

        for node in &self.nodes {
            let is_online = self.is_node_online(node).await;
            debug!(
                "通道 {} [xFusion]: 节点 ID:{} 状态 - 在线: {}",
                self.channel_id, node.id, is_online
            );

            status_list.push(serde_json::json!({
                "id": node.id,
                "mac": node.mac_text,
                "ip": node.ip.map(|i| i.to_string()),
                "port": node.port,
                "online": is_online,
                "ibmc_url": node.ibmc_url,
            }));
        }

        info!(
            "通道 {} [xFusion]: 获取状态完成, {} 个节点",
            self.channel_id, status_list.len()
        );

        Ok(serde_json::json!({
            "channel_id": self.channel_id,
            "list": status_list
        }))
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        info!(
            "通道 {} [Write]: 节点 ID:{}, 值: {} ({})",
            self.channel_id, id, value, if value == 1 { "开机" } else if value == 0 { "关机" } else { "未知" }
        );
        let node = self
            .nodes
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| {
                warn!("通道 {} [Write]: 未找到 ID 为 {} 的节点", self.channel_id, id);
                DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
            })?;

        match value {
            1 => {
                info!("通道 {} [Write]: 节点 ID:{} 执行开机", self.channel_id, id);
                self.power_on(node).await
            }
            0 => {
                info!("通道 {} [Write]: 节点 ID:{} 执行关机", self.channel_id, id);
                self.power_off(node).await
            }
            v => {
                warn!("通道 {} [Write]: 节点 ID:{} 不支持的值 {}", self.channel_id, id, v);
                Err(DeviceError::ProtocolError(format!(
                    "xFusion write 仅支持 0(关机) 或 1(开机)，收到: {}",
                    v
                )))
            }
        }
    }

    async fn read(&self, id: u32) -> Result<i32> {
        debug!("通道 {} [Read]: 读取节点 ID:{} 状态", self.channel_id, id);
        let node = self
            .nodes
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| {
                warn!("通道 {} [Read]: 未找到 ID 为 {} 的节点", self.channel_id, id);
                DeviceError::ProtocolError(format!("未找到 ID 为 {} 的节点", id))
            })?;

        // 通过 iBMC API 获取电源状态
        let is_powered_on = self.is_node_powered_on(node).await;

        info!(
            "通道 {} [Read]: 节点 ID:{} 电源状态: {} ({})",
            self.channel_id, id, if is_powered_on { "开机" } else { "关机" }, if is_powered_on { 1 } else { 0 }
        );
        Ok(if is_powered_on { 1 } else { 0 })
    }

    fn name(&self) -> &str {
        "xFusion"
    }
}
