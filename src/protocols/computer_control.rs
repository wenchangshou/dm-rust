use crate::protocols::Protocol;
use crate::utils::{DeviceError, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::{Duration, Instant};

use mac_address::MacAddress;
use tracing::{debug, error, info, warn};
use wake_on_lan::MagicPacket;

struct ComputerNode {
    id: u32,
    mac_text: String,
    mac_bytes: [u8; 6],
    ip: Option<Ipv4Addr>,
    port: Option<u16>,
    last_heartbeat: Option<Instant>,
}

#[derive(Deserialize)]
struct ComputerConfigItem {
    id: u32,
    mac: String,
    ip: Option<String>,
    port: Option<u16>,
}

/// 电脑控制协议（WOL + 状态监控）
pub struct ComputerControlProtocol {
    channel_id: u32,
    computers: Vec<ComputerNode>,
    broadcast_addr: Ipv4Addr,
    wol_port: u16,
    shutdown_port: u16,
}

impl ComputerControlProtocol {
    async fn wake(&self, mac: &str, mac_bytes: &[u8; 6]) -> Result<()> {
        let to_addr = (self.broadcast_addr, self.wol_port);
        let mac_val = *mac_bytes;

        info!(
            "通道 {} [WOL]: 发送魔术包到 {} ({}), 目标广播: {}:{}",
            self.channel_id, mac, hex::encode(mac_bytes), self.broadcast_addr, self.wol_port
        );

        tokio::task::spawn_blocking(move || {
            let packet = MagicPacket::new(&mac_val);
            packet.send_to(to_addr, (Ipv4Addr::new(0, 0, 0, 0), 0))
        })
        .await
        .map_err(|e| {
            error!("通道 {} [WOL]: 任务执行失败: {}", self.channel_id, e);
            DeviceError::ProtocolError(format!("WOL 任务执行失败: {}", e))
        })?
        .map_err(|e| {
            error!("通道 {} [WOL]: 发送魔术包失败: {}", self.channel_id, e);
            DeviceError::ProtocolError(format!("发送 WOL 魔术包失败: {}", e))
        })?;

        Ok(())
    }

    async fn request_shutdown(&self, computer: &ComputerNode) -> Result<()> {
        if let (Some(ip), Some(port)) = (computer.ip, computer.port) {
            info!(
                "通道 {} [Shutdown]: 向电脑 ID:{} 发送 UDP 命令 (IP:{}, Port:{})",
                self.channel_id, computer.id, ip, port
            );
            self.send_udp(ip, port, "shutdown", false).await?;
        } else {
            info!(
                "通道 {} [Shutdown]: 向电脑 ID:{} 发送广播命令 (MAC:{}, 广播地址: {}:{})",
                self.channel_id, computer.id, computer.mac_text, self.broadcast_addr, self.shutdown_port
            );
            // Legacy broadcast shutdown using MAC address
            let socket = tokio::net::UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))
                .await
                .map_err(|e| DeviceError::ProtocolError(format!("绑定 UDP socket 失败: {}", e)))?;

            socket
                .set_broadcast(true)
                .map_err(|e| DeviceError::ProtocolError(format!("设置 UDP 广播失败: {}", e)))?;

            let to = SocketAddrV4::new(self.broadcast_addr, self.shutdown_port);

            socket
                .send_to(computer.mac_text.as_bytes(), to)
                .await
                .map_err(|e| DeviceError::ProtocolError(format!("发送关机 UDP 失败: {}", e)))?;
        }

        Ok(())
    }

    async fn send_udp(
        &self,
        ip: Ipv4Addr,
        port: u16,
        command: &str,
        wait_response: bool,
    ) -> Result<Option<String>> {
        debug!("通道 {} [UDP]: 发送命令 '{}' 到 {}:{}", self.channel_id, command, ip, port);
        let socket = tokio::net::UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))
            .await
            .map_err(|e| DeviceError::ProtocolError(format!("绑定 UDP socket 失败: {}", e)))?;

        let to = SocketAddrV4::new(ip, port);
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
                    debug!("通道 {} [UDP]: 收到来自 {}:{} 的响应: {}", self.channel_id, ip, port, response);
                    Ok(Some(response))
                }
                _ => {
                    debug!("通道 {} [UDP]: 等待 {}:{} 响应超时", self.channel_id, ip, port);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn ping_computer(&self, computer: &ComputerNode) -> bool {
        if let (Some(ip), Some(port)) = (computer.ip, computer.port) {
            if let Ok(Some(resp)) = self.send_udp(ip, port, "ping", true).await {
                return resp.eq_ignore_ascii_case("pong");
            }
        }
        false
    }

    fn update_heartbeat(&mut self, mac: &str) -> bool {
        // 标准化 MAC 地址对比 (忽略大小写)
        for computer in &mut self.computers {
            if computer.mac_text.eq_ignore_ascii_case(mac) {
                debug!("通道 {} [Heartbeat]: 更新电脑 ID:{} (MAC:{}) 的心跳", self.channel_id, computer.id, mac);
                computer.last_heartbeat = Some(Instant::now());
                return true;
            }
        }
        warn!("通道 {} [Heartbeat]: 收到未知 MAC 地址的心跳: {}", self.channel_id, mac);
        false
    }

    fn find_computer_by_id(&self, id: u32) -> Option<&ComputerNode> {
        self.computers.iter().find(|c| c.id == id)
    }

    async fn is_computer_online(&self, computer: &ComputerNode) -> bool {
        if self.ping_computer(computer).await {
            true
        } else if let Some(last) = computer.last_heartbeat {
            Instant::now().duration_since(last) < Duration::from_secs(10)
        } else {
            false
        }
    }

    async fn get_audio_status(&self, computer: &ComputerNode) -> (Option<i32>, Option<bool>, bool) {
        let is_online = self.is_computer_online(computer).await;
        if !is_online {
            return (None, None, false);
        }

        if let (Some(ip), Some(port)) = (computer.ip, computer.port) {
            match self.send_udp(ip, port, "get", true).await {
                Ok(Some(resp)) => {
                    // 解析格式: "volume: 50, mute: false"
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
                    (volume, mute, true)
                }
                _ => (None, None, true),
            }
        } else {
            (None, None, is_online)
        }
    }
}

#[async_trait]
impl Protocol for ComputerControlProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        // 期望的配置格式:
        // {
        //   "mac_address": [ {"id": 1, "mac": "00:11:22..."}, ... ],
        //   "broadcast_addr": "255.255.255.255",
        //   "wol_port": 9,
        //   "shutdown_port": 4001
        // }

        let computer_list_json = params.get("mac_address").ok_or_else(|| {
            DeviceError::ConfigError("computerControl 缺少 mac_address 参数".into())
        })?;

        let config_items: Vec<ComputerConfigItem> =
            serde_json::from_value(computer_list_json.clone())
                .map_err(|e| DeviceError::ConfigError(format!("mac_address 解析失败: {}", e)))?;

        let mut computers = Vec::new();
        for item in config_items {
            let mac_bytes = item
                .mac
                .parse::<MacAddress>()
                .map_err(|e| {
                    DeviceError::ConfigError(format!("MAC地址格式错误 {}: {}", item.mac, e))
                })?
                .bytes();

            let ip = item.ip.as_ref().and_then(|s| s.parse::<Ipv4Addr>().ok());

            computers.push(ComputerNode {
                id: item.id,
                mac_text: item.mac,
                mac_bytes,
                ip,
                port: item.port,
                last_heartbeat: None,
            });
        }

        let broadcast_addr = params
            .get("broadcast_addr")
            .or_else(|| params.get("broadcast"))
            .and_then(|v| v.as_str())
            .map(|s| {
                s.parse::<Ipv4Addr>().map_err(|e| {
                    DeviceError::ConfigError(format!(
                        "computerControl broadcast_addr 解析失败: {} (值: {})",
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

        debug!(
            "通道 {} [Config]: 初始化 ComputerControlProtocol, 包含 {} 台电脑, 广播地址: {}",
            channel_id,
            computers.len(),
            broadcast_addr
        );

        Ok(Box::new(Self {
            channel_id,
            computers,
            broadcast_addr,
            wol_port,
            shutdown_port,
        }))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        debug!("通道 {} [Execute]: 收到命令 '{}', 参数: {}", self.channel_id, command, params);
        match command {
            "powerOn" | "wake" | "wol" => {
                // 如果指定了 id，只唤醒该 ID；否则唤醒所有？或者报错？
                // 这里假设必须指定 id 或者 mac
                let id = params.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);
                let mac = params.get("mac").and_then(|v| v.as_str());

                let mut targets = Vec::new();

                if let Some(id) = id {
                    if let Some(comp) = self.find_computer_by_id(id) {
                        targets.push(comp);
                    }
                } else if let Some(mac) = mac {
                    if let Some(comp) = self
                        .computers
                        .iter()
                        .find(|c| c.mac_text.eq_ignore_ascii_case(mac))
                    {
                        targets.push(comp);
                    }
                }

                if targets.is_empty() {
                    warn!("通道 {} [Execute]: {} 命令未找到匹配的电脑, 参数: {}", self.channel_id, command, params);
                    return Err(DeviceError::ProtocolError(
                        "powerOn 需要指定 id 或 mac".into(),
                    ));
                }

                for comp in targets {
                    let mac_text = comp.mac_text.clone();
                    let mac_bytes = comp.mac_bytes;
                    self.wake(&mac_text, &mac_bytes).await?;
                }

                Ok(serde_json::json!({ "status": "ok", "action": "wake" }))
            }
            "powerOff" | "shutdown" => {
                let id = params.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);
                let mac = params.get("mac").and_then(|v| v.as_str());

                let mut targets = Vec::new();
                if let Some(id) = id {
                    if let Some(comp) = self.find_computer_by_id(id) {
                        targets.push(comp);
                    }
                } else if let Some(mac) = mac {
                    if let Some(comp) = self
                        .computers
                        .iter()
                        .find(|c| c.mac_text.eq_ignore_ascii_case(mac))
                    {
                        targets.push(comp);
                    }
                }

                if targets.is_empty() {
                    warn!("通道 {} [Execute]: {} 命令未找到匹配的电脑, 参数: {}", self.channel_id, command, params);
                    return Err(DeviceError::ProtocolError(
                        "powerOff 需要指定 id 或 mac".into(),
                    ));
                }

                for comp in targets {
                    self.request_shutdown(comp).await?;
                }

                Ok(serde_json::json!({ "status": "ok", "action": "shutdown" }))
            }
            "method" => {
                let id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .ok_or_else(|| DeviceError::ProtocolError("method 命令需要 id 参数".into()))?;
                let method = params
                    .get("method")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        DeviceError::ProtocolError("method 命令需要 method 参数".into())
                    })?;

                let computer = self.find_computer_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的电脑", id))
                })?;

                if let (Some(ip), Some(port)) = (computer.ip, computer.port) {
                    self.send_udp(ip, port, method, false).await?;
                    Ok(serde_json::json!({ "status": "ok", "method": method }))
                } else {
                    warn!("通道 {} [Execute]: ID 为 {} 的电脑缺少 IP 或 端口配置", self.channel_id, id);
                    Err(DeviceError::ProtocolError(format!(
                        "ID 为 {} 的电脑缺少 IP 或 端口配置",
                        id
                    )))
                }
            }
            "heartbeat" => {
                // "上报当前心跳 然后传入的是一个mac地址"
                let mac = params.get("mac").and_then(|v| v.as_str()).ok_or_else(|| {
                    DeviceError::ProtocolError("heartbeat 命令需要 mac 参数".into())
                })?;

                if self.update_heartbeat(mac) {
                    Ok(serde_json::json!({ "status": "ok", "msg": "heartbeat updated" }))
                } else {
                    // 这里可以选择报错，或者忽略
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

                let computer = self.find_computer_by_id(id).ok_or_else(|| {
                    DeviceError::ProtocolError(format!("未找到 ID 为 {} 的电脑", id))
                })?;

                let (volume, mute, online) = self.get_audio_status(computer).await;
                Ok(serde_json::json!({
                    "id": id,
                    "online": online,
                    "volume": volume,
                    "mute": mute,
                }))
            }
            "getAllStatus" => self.get_status().await,
            _ => {
                warn!("通道 {} [Execute]: 未知命令: {}", self.channel_id, command);
                Err(DeviceError::ProtocolError(format!("未知命令: {}", command)))
            }
        }
    }

    async fn get_status(&self) -> Result<Value> {
        // "获取所有的主机的状态"
        // "检测到10秒没有心跳，就将当前设置成离线"
        let now = Instant::now();
        let timeout = Duration::from_secs(10);

        let mut status_list = Vec::new();

        for comp in &self.computers {
            let is_online = self.is_computer_online(comp).await;

            status_list.push(serde_json::json!({
                "id": comp.id,
                "mac": comp.mac_text,
                "ip": comp.ip.map(|i| i.to_string()),
                "port": comp.port,
                "online": is_online
            }));
        }

        Ok(serde_json::json!({
            "channel_id": self.channel_id,
            "list": status_list
        }))
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        // write 通过 id 控制具体哪台机器
        // 1: 开机, 0: 关机
        debug!("通道 {} [Write]: ID {}, Value {}", self.channel_id, id, value);
        let comp = self
            .computers
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| DeviceError::ProtocolError(format!("未找到 ID 为 {} 的电脑", id)))?;

        let mac_text = comp.mac_text.clone();
        let mac_bytes = comp.mac_bytes;

        match value {
            1 => self.wake(&mac_text, &mac_bytes).await,
            0 => self.request_shutdown(comp).await,
            v => {
                warn!("通道 {} [Write]: 不支持的值 {}", self.channel_id, v);
                Err(DeviceError::ProtocolError(format!(
                    "computerControl write 仅支持 0(关机) 或 1(开机)，收到: {}",
                    v
                )))
            }
        }
    }

    async fn read(&self, id: u32) -> Result<i32> {
        // 返回在线状态？ 1 online, 0 offline
        let comp = self
            .computers
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| DeviceError::ProtocolError(format!("未找到 ID 为 {} 的电脑", id)))?;

        let now = Instant::now();
        let timeout = Duration::from_secs(10);
        let is_online = self.is_computer_online(comp).await;

        debug!("通道 {} [Read]: ID {} 在线状态: {}", self.channel_id, id, is_online);
        Ok(if is_online { 1 } else { 0 })
    }

    fn name(&self) -> &str {
        "computerControl"
    }
}
