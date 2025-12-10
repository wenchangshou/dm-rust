/// 通道管理器 - 负责物理设备通信层
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error};

use crate::config::{ChannelConfig, StatuteType};
use crate::protocols::{Protocol, PjlinkProtocol, ModbusProtocol, ModbusSlaveProtocol,
                       XinkeQ1Protocol, ComputerControlProtocol, CustomProtocol,
                       ScreenNjlgPlcProtocol, HsPowerSequencerProtocol, NovastarProtocol,
                       MockProtocol};
use crate::utils::{Result, DeviceError};
use super::DeviceEvent;

/// 通道管理器
pub struct ChannelManager {
    channels: DashMap<u32, Channel>,
    event_tx: broadcast::Sender<DeviceEvent>,
}

/// 单个通道
struct Channel {
    id: u32,
    protocol: Arc<RwLock<Box<dyn Protocol>>>,
    config: ChannelConfig,
}

impl ChannelManager {
    /// 创建通道管理器
    pub async fn new(
        configs: &[ChannelConfig],
        event_tx: broadcast::Sender<DeviceEvent>,
    ) -> Result<Self> {
        let channels = DashMap::new();

        for config in configs {
            if !config.enable {
                continue;
            }

            match Self::create_channel(config).await {
                Ok(channel) => {
                    info!("通道 {} ({:?}) 初始化成功", config.channel_id, config.statute);
                    channels.insert(config.channel_id, channel);

                    // 发送连接事件
                    let _ = event_tx.send(DeviceEvent::ChannelConnected {
                        channel_id: config.channel_id,
                    });
                }
                Err(e) => {
                    warn!("通道 {} 初始化失败: {:?}", config.channel_id, e);
                }
            }
        }

        Ok(Self { channels, event_tx })
    }

    /// 创建单个通道
    async fn create_channel(config: &ChannelConfig) -> Result<Channel> {
        // 合并参数：优先使用 arguments，如果没有则使用 params（兼容旧配置）
        let mut params = if let Some(args) = &config.arguments {
            // 如果 arguments 是对象，转换为 HashMap
            if let Some(obj) = args.as_object() {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                // 如果不是对象，使用空 HashMap
                std::collections::HashMap::new()
            }
        } else {
            // 使用旧的 params
            config.params.clone()
        };

        // 添加 auto_call 配置（如果存在）
        if let Some(auto_call) = &config.auto_call {
            params.insert("auto_call".to_string(), serde_json::to_value(auto_call).unwrap());
        }

        // 使用协议的 from_config 方法创建实例，协议自己解析配置
        let protocol: Box<dyn Protocol> = match config.statute {
            StatuteType::Pjlink => {
                PjlinkProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::Modbus => {
                ModbusProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::ModbusSlave => {
                ModbusSlaveProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::XinkeQ1 => {
                XinkeQ1Protocol::from_config(config.channel_id, &params)?
            }

            StatuteType::ComputerControl => {
                ComputerControlProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::Custom => {
                CustomProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::ScreenNjlgPlc => {
                ScreenNjlgPlcProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::HsPowerSequencer => {
                HsPowerSequencerProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::Novastar => {
                NovastarProtocol::from_config(config.channel_id, &params)?
            }

            StatuteType::Mock => {
                MockProtocol::from_config(config.channel_id, &params)?
            }

            _ => {
                return Err(DeviceError::ProtocolError(
                    format!("不支持的协议类型: {:?}", config.statute)
                ));
            }
        };

        Ok(Channel {
            id: config.channel_id,
            protocol: Arc::new(RwLock::new(protocol)),
            config: config.clone(),
        })
    }

    /// 写入数据到指定通道的设备
    pub async fn write(&self, channel_id: u32, device_id: u32, value: i32) -> Result<()> {
        let channel = self.channels.get(&channel_id)
            .ok_or_else(|| DeviceError::ChannelNotFound(channel_id))?;

        let mut protocol = channel.protocol.write().await;
        protocol.write(device_id, value).await
    }

    /// 从指定通道的设备读取数据
    pub async fn read(&self, channel_id: u32, device_id: u32) -> Result<i32> {
        let channel = self.channels.get(&channel_id)
            .ok_or_else(|| DeviceError::ChannelNotFound(channel_id))?;

        let protocol = channel.protocol.read().await;
        protocol.read(device_id).await
    }

    /// 执行通道命令
    pub async fn execute(
        &self,
        channel_id: u32,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let channel = self.channels.get(&channel_id)
            .ok_or_else(|| DeviceError::ChannelNotFound(channel_id))?;

        let mut protocol = channel.protocol.write().await;
        protocol.execute(command, params).await
    }

    /// 获取所有通道状态
    pub async fn get_all_status(&self) -> Result<serde_json::Value> {
        let mut statuses = Vec::new();

        for entry in self.channels.iter() {
            let channel_id = *entry.key();
            let channel = entry.value();
            let protocol = channel.protocol.read().await;

            match protocol.get_status().await {
                Ok(status) => {
                    statuses.push(serde_json::json!({
                        "channel_id": channel_id,
                        "statute": format!("{:?}", channel.config.statute),
                        "status": status,
                    }));
                }
                Err(e) => {
                    warn!("获取通道 {} 状态失败: {:?}", channel_id, e);
                }
            }
        }

        Ok(serde_json::json!(statuses))
    }

    /// 获取通道数量
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// 调用通道的自定义方法
    pub async fn call_method(
        &self,
        channel_id: u32,
        method_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let channel = self.channels.get(&channel_id)
            .ok_or_else(|| DeviceError::ChannelNotFound(channel_id))?;

        let mut protocol = channel.protocol.write().await;
        protocol.call_method(method_name, args).await
    }

    /// 获取通道支持的方法列表
    pub async fn get_channel_methods(&self, channel_id: u32) -> Result<Vec<String>> {
        let channel = self.channels.get(&channel_id)
            .ok_or_else(|| DeviceError::ChannelNotFound(channel_id))?;

        let protocol = channel.protocol.read().await;
        Ok(protocol.get_methods())
    }
}
