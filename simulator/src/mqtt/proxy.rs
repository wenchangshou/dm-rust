/// MQTT 代理模式
///
/// 作为中介连接到上游 Broker，转发消息并进行监控
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::rules::MqttRuleSet;
use super::state::{MqttPacketRecord, MqttSimulatorState, PacketDirection, ProxyConfig};
use chrono::Utc;

/// MQTT 代理服务器
pub struct MqttProxy {
    config: ProxyConfig,
    local_port: u16,
    state: Arc<RwLock<MqttSimulatorState>>,
    rules: Arc<RwLock<MqttRuleSet>>,
    upstream_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MqttProxy {
    pub fn new(
        config: ProxyConfig,
        local_port: u16,
        state: Arc<RwLock<MqttSimulatorState>>,
        rules: Arc<RwLock<MqttRuleSet>>,
    ) -> Self {
        Self {
            config,
            local_port,
            state,
            rules,
            upstream_handle: None,
        }
    }

    /// 启动代理
    pub async fn start(&mut self) -> Result<(), String> {
        info!("Proxy::start() called");

        if self.upstream_handle.is_some() {
            error!("Proxy already has a handle, returning error");
            return Err("Proxy already running".to_string());
        }

        let config = self.config.clone();
        let state = self.state.clone();
        let rules = self.rules.clone();

        info!(
            "Starting MQTT Proxy: local:{} -> upstream:{}:{}",
            self.local_port, config.upstream_host, config.upstream_port
        );

        info!("Spawning proxy task...");
        let handle = tokio::spawn(async move {
            info!("Proxy task started");

            // 连接到上游 Broker
            let client_id = format!("{}_{}", config.client_id_prefix, uuid::Uuid::new_v4());
            info!("Generated client ID: {}", client_id);

            info!("Creating MQTT options for upstream {}:{}", config.upstream_host, config.upstream_port);
            let mut mqtt_options =
                MqttOptions::new(client_id.clone(), &config.upstream_host, config.upstream_port);
            mqtt_options.set_keep_alive(std::time::Duration::from_secs(30));

            if let (Some(username), Some(password)) =
                (&config.upstream_username, &config.upstream_password)
            {
                info!("Setting credentials for upstream connection");
                mqtt_options.set_credentials(username, password);
            } else {
                info!("No credentials configured for upstream connection");
            }

            info!("Creating AsyncClient...");
            let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

            // 订阅所有消息
            info!("Subscribing to # topic...");
            if let Err(e) = client.subscribe("#", QoS::AtLeastOnce).await {
                error!("Failed to subscribe: {:?}", e);
                return;
            }
            info!("Subscribed to # topic successfully");

            info!("Connected to upstream MQTT broker, entering event loop");

            // 事件循环
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        match event {
                            Event::Incoming(Packet::Publish(publish)) => {
                                let topic = publish.topic.clone();
                                let payload = publish.payload.to_vec();

                                debug!("Proxy received: topic={}, len={}", topic, payload.len());

                                // 记录报文
                                {
                                    let mut state_guard = state.write().await;
                                    let id = state_guard.packet_counter + 1;
                                    state_guard.add_packet(MqttPacketRecord {
                                        id,
                                        timestamp: Utc::now(),
                                        direction: PacketDirection::Forwarded,
                                        client_id: None,
                                        packet_type: "PUBLISH".to_string(),
                                        topic: Some(topic.clone()),
                                        payload: Some(
                                            String::from_utf8_lossy(&payload).to_string(),
                                        ),
                                        payload_hex: Some(hex::encode(&payload)),
                                        qos: Some(publish.qos as u8),
                                    });
                                    state_guard.stats.messages_received += 1;
                                    state_guard.stats.bytes_received += payload.len() as u64;
                                    state_guard.stats.last_activity = Some(Utc::now());
                                }

                                // 规则匹配处理
                                let rules_guard = rules.read().await;
                                for rule in rules_guard.find_matching(&topic, &payload) {
                                    debug!("Proxy rule matched: {}", rule.name);
                                }
                            }
                            Event::Incoming(Packet::ConnAck(_)) => {
                                info!("Upstream connection acknowledged");
                                let mut state_guard = state.write().await;
                                state_guard.stats.total_connections += 1;
                                state_guard.stats.active_connections = 1;
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                warn!("Upstream disconnected");
                                let mut state_guard = state.write().await;
                                state_guard.stats.active_connections = 0;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Proxy eventloop error: {:?}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        info!("Proxy task spawned, saving handle...");
        self.upstream_handle = Some(handle);
        info!("Proxy started successfully");
        Ok(())
    }

    /// 停止代理
    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(handle) = self.upstream_handle.take() {
            handle.abort();
            info!("MQTT Proxy stopped");
        }
        Ok(())
    }
}
