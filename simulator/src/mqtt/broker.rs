/// MQTT Broker 实现
///
/// 使用 rumqttd 作为嵌入式 MQTT Broker
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use rumqttc::v5 as mqttv5;
use rumqttc::v5::mqttbytes::v5::Packet as PacketV5;
use rumqttc::v5::mqttbytes::QoS as QoSV5;
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use std::time::Duration;
use chrono::Utc;

use super::rules::MqttRuleSet;
use super::state::{MqttPacketRecord, MqttSimulatorState, MqttVersion, PacketDirection};

/// MQTT Broker 服务器
pub struct MqttBroker {
    port: u16,
    bind_addr: String,
    mqtt_versions: Vec<MqttVersion>,
    state: Arc<RwLock<MqttSimulatorState>>,
    rules: Arc<RwLock<MqttRuleSet>>,
    broker_handle: Option<std::thread::JoinHandle<()>>,
    monitor_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl MqttBroker {
    pub fn new(
        port: u16,
        bind_addr: String,
        mqtt_versions: Vec<MqttVersion>,
        state: Arc<RwLock<MqttSimulatorState>>,
        rules: Arc<RwLock<MqttRuleSet>>,
    ) -> Self {
        Self {
            port,
            bind_addr,
            mqtt_versions,
            state,
            rules,
            broker_handle: None,
            monitor_handle: None,
            shutdown_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 启动 Broker
    pub async fn start(&mut self) -> Result<(), String> {
        info!("Broker::start() called");

        if self.broker_handle.is_some() {
            error!("Broker already has a handle, returning error");
            return Err("Broker already running".to_string());
        }

        let port = self.port;
        let bind_addr = self.bind_addr.clone();
        let _state = self.state.clone();
        let _rules = self.rules.clone();

        info!("Starting MQTT Broker on {}:{}", bind_addr, port);

        // 创建 rumqttd 配置
        info!("Creating broker configuration...");
        let config = self.create_config();
        info!("Broker config created: listen={}:{}", bind_addr, port);

        // 用于接收启动错误的通道
        info!("Creating error channel...");
        let (tx, rx) = std::sync::mpsc::channel();

        // 启动 broker 在单独的线程
        info!("Spawning broker thread...");
        let handle = std::thread::spawn(move || {
            info!("Broker thread started, creating Broker instance...");
            let mut broker = Broker::new(config);
            info!("Broker instance created, calling broker.start()...");

            // 运行 Broker
            if let Err(e) = broker.start() {
                error!("MQTT Broker start() failed: {:?}", e);
                // 发送错误到主线程（如果主线程还在等待）
                let _ = tx.send(format!("MQTT Broker failed to start: {:?}", e));
                error!("Error message sent to channel");
            } else {
                info!("MQTT Broker started successfully on {}:{}", bind_addr, port);
            }
        });

        info!("Broker thread spawned, saving handle...");
        self.broker_handle = Some(handle);

        // 等待 500ms 看看是否有错误返回
        // rumqttd::Broker::start() 如果端口被占用会立即失败
        info!("Waiting for broker startup (500ms timeout)...");
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(err_msg) => {
                // 收到错误，说明启动失败
                error!("Received error from broker thread: {}", err_msg);
                self.broker_handle = None; // 线程已退出
                return Err(err_msg);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // 超时未收到错误，假设启动成功（已进入阻塞循环）
                info!("No error received within timeout, assuming broker started successfully");

                // 更新统计信息
                info!("Updating broker state statistics...");
                {
                    let mut state_guard = self.state.write().await;
                    state_guard.stats.active_connections = 0;
                    state_guard.stats.total_connections = 0;
                    info!("State updated: active_connections=0, total_connections=0");
                    // Manager 会负责将 Status 设置为 Running
                }

                // 启动监控客户端来拦截消息和应用规则
                info!("Starting monitor client to intercept messages...");
                self.start_monitor_client().await?;

                info!("Broker started successfully");
                Ok(())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // 发送端断开，说明线程退出了但没有发送错误？不应该发生
                error!("Broker error channel disconnected unexpectedly");
                self.broker_handle = None;
                Err("Broker thread exited unexpectedly".to_string())
            }
        }
    }

    /// 停止 Broker
    pub async fn stop(&mut self) -> Result<(), String> {
        self.shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);

        // 停止监控客户端
        if let Some(handle) = self.monitor_handle.take() {
            handle.abort();
            info!("MQTT monitor client stopped");
        }

        if let Some(_handle) = self.broker_handle.take() {
            info!("MQTT Broker stop requested");
            // 实际上无法优雅停止 rumqttd 阻塞线程，除非 send signal 或它自己支持
            // 这里只能移除 handle，让它在后台并在进程退出时结束
            // 这是一个 rumqttd 嵌入使用的已知限制
        }

        Ok(())
    }

    /// 启动监控客户端来拦截消息和执行规则
    async fn start_monitor_client(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        let rules = self.rules.clone();
        let port = self.port;
        let mqtt_versions = self.mqtt_versions.clone();

        // 确定要连接的端口和协议版本
        let has_v4 = mqtt_versions.contains(&MqttVersion::V4);
        let has_v5 = mqtt_versions.contains(&MqttVersion::V5);

        // 如果只有v5，使用配置端口和v5协议；如果有v4，优先使用v4端口和v4协议
        let (connect_port, use_v5) = if has_v4 {
            (port, false)  // v4端口，使用v4协议
        } else if has_v5 {
            (port, true)   // v5端口，使用v5协议
        } else {
            (port, false)  // 默认v4
        };

        info!("Monitor client will connect to localhost:{} using MQTT {}",
            connect_port, if use_v5 { "v5" } else { "v3.1.1" });

        let handle = if use_v5 {
            // 使用 MQTT v5 客户端
            Self::spawn_v5_monitor(state, rules, connect_port)
        } else {
            // 使用 MQTT v3.1.1 客户端
            Self::spawn_v4_monitor(state, rules, connect_port)
        };

        self.monitor_handle = Some(handle);
        info!("Monitor client task spawned");
        Ok(())
    }

    /// 启动 v4 监控客户端
    fn spawn_v4_monitor(
        state: Arc<RwLock<MqttSimulatorState>>,
        rules: Arc<RwLock<MqttRuleSet>>,
        port: u16,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // 等待broker完全启动
            tokio::time::sleep(Duration::from_millis(200)).await;

            info!("Monitor client starting with MQTT v3.1.1, connecting to broker...");

            // 创建监控客户端
            let client_id = format!("broker_monitor_{}", uuid::Uuid::new_v4());
            let mut mqtt_options = MqttOptions::new(client_id.clone(), "127.0.0.1", port);
            mqtt_options.set_keep_alive(Duration::from_secs(30));

            info!("Monitor client ID: {}, connecting to 127.0.0.1:{}", client_id, port);

            let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

            // 订阅所有消息
            info!("Monitor client subscribing to # topic...");
            if let Err(e) = client.subscribe("#", QoS::AtLeastOnce).await {
                error!("Monitor client failed to subscribe: {:?}", e);
                return;
            }
            info!("Monitor client subscribed successfully");

            // 事件循环
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        match event {
                            Event::Incoming(Packet::Publish(publish)) => {
                                let topic = publish.topic.clone();
                                let payload = publish.payload.to_vec();

                                debug!("Broker received message: topic={}, len={}", topic, payload.len());

                                // 记录报文
                                {
                                    let mut state_guard = state.write().await;
                                    let id = state_guard.packet_counter + 1;
                                    state_guard.add_packet(MqttPacketRecord {
                                        id,
                                        timestamp: Utc::now(),
                                        direction: PacketDirection::Received,
                                        client_id: None,
                                        packet_type: "PUBLISH".to_string(),
                                        topic: Some(topic.clone()),
                                        payload: Some(String::from_utf8_lossy(&payload).to_string()),
                                        payload_hex: Some(hex::encode(&payload)),
                                        qos: Some(publish.qos as u8),
                                    });
                                    state_guard.stats.messages_received += 1;
                                    state_guard.stats.bytes_received += payload.len() as u64;
                                    state_guard.stats.last_activity = Some(Utc::now());
                                }

                                // 规则匹配处理
                                let matched_rules = {
                                    let rules_guard = rules.read().await;
                                    rules_guard.find_matching(&topic, &payload)
                                        .into_iter()
                                        .map(|r| r.clone())
                                        .collect::<Vec<_>>()
                                };

                                if !matched_rules.is_empty() {
                                    info!("Broker: {} rules matched for topic {}", matched_rules.len(), topic);
                                    for rule in matched_rules {
                                        info!("Broker rule matched: {} - {}", rule.name, rule.id);

                                        // 执行规则动作
                                        match &rule.action {
                                            super::rules::MqttRuleAction::Log { message } => {
                                                let default_msg = format!("Rule '{}' triggered", rule.name);
                                                let log_msg = message.as_ref()
                                                    .map(|m| m.as_str())
                                                    .unwrap_or(&default_msg);
                                                info!("Rule action [Log]: {}", log_msg);
                                            }
                                            super::rules::MqttRuleAction::Respond { topic: resp_topic, payload: resp_payload, .. } => {
                                                info!("Rule action [Respond]: publishing to {} with payload {}", resp_topic, resp_payload);
                                                if let Err(e) = client.publish(resp_topic, QoS::AtLeastOnce, false, resp_payload.as_bytes()).await {
                                                    error!("Failed to publish response: {:?}", e);
                                                }
                                            }
                                            super::rules::MqttRuleAction::Forward { target_topic } => {
                                                info!("Rule action [Forward]: forwarding to {}", target_topic);
                                                if let Err(e) = client.publish(target_topic, QoS::AtLeastOnce, false, payload.clone()).await {
                                                    error!("Failed to forward message: {:?}", e);
                                                }
                                            }
                                            super::rules::MqttRuleAction::Silence => {
                                                debug!("Rule action [Silence]: message silenced");
                                            }
                                            super::rules::MqttRuleAction::Transform { output_topic, output_payload } => {
                                                info!("Rule action [Transform]: publishing to {} with transformed payload", output_topic);
                                                if let Err(e) = client.publish(output_topic, QoS::AtLeastOnce, false, output_payload.as_bytes()).await {
                                                    error!("Failed to publish transformed message: {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    debug!("No rules matched for topic {}", topic);
                                }
                            }
                            Event::Incoming(Packet::ConnAck(_)) => {
                                info!("Monitor client connected to broker");
                                let mut state_guard = state.write().await;
                                state_guard.stats.total_connections += 1;
                                state_guard.stats.active_connections += 1;
                            }
                            Event::Incoming(Packet::Disconnect) => {
                                warn!("Monitor client disconnected from broker");
                                let mut state_guard = state.write().await;
                                if state_guard.stats.active_connections > 0 {
                                    state_guard.stats.active_connections -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Monitor client eventloop error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        })
    }

    /// 启动 v5 监控客户端
    fn spawn_v5_monitor(
        state: Arc<RwLock<MqttSimulatorState>>,
        rules: Arc<RwLock<MqttRuleSet>>,
        port: u16,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            // 等待broker完全启动
            tokio::time::sleep(Duration::from_millis(200)).await;

            info!("Monitor client starting with MQTT v5, connecting to broker...");

            // 创建监控客户端
            let client_id = format!("broker_monitor_{}", uuid::Uuid::new_v4());
            let mut mqtt_options = mqttv5::MqttOptions::new(client_id.clone(), "127.0.0.1", port);
            mqtt_options.set_keep_alive(Duration::from_secs(30));

            info!("Monitor client ID: {}, connecting to 127.0.0.1:{}", client_id, port);

            let (client, mut eventloop) = mqttv5::AsyncClient::new(mqtt_options, 100);

            // 订阅所有消息
            info!("Monitor client subscribing to # topic...");
            if let Err(e) = client.subscribe("#", QoSV5::AtLeastOnce).await {
                error!("Monitor client failed to subscribe: {:?}", e);
                return;
            }
            info!("Monitor client subscribed successfully");

            // 事件循环
            loop {
                match eventloop.poll().await {
                    Ok(event) => {
                        use mqttv5::Event;
                        match event {
                            Event::Incoming(PacketV5::Publish(publish)) => {
                                let topic = String::from_utf8_lossy(&publish.topic).to_string();
                                let payload = publish.payload.to_vec();

                                debug!("Broker received message: topic={}, len={}", topic, payload.len());

                                // 记录报文
                                {
                                    let mut state_guard = state.write().await;
                                    let id = state_guard.packet_counter + 1;
                                    state_guard.add_packet(MqttPacketRecord {
                                        id,
                                        timestamp: Utc::now(),
                                        direction: PacketDirection::Received,
                                        client_id: None,
                                        packet_type: "PUBLISH".to_string(),
                                        topic: Some(topic.clone()),
                                        payload: Some(String::from_utf8_lossy(&payload).to_string()),
                                        payload_hex: Some(hex::encode(&payload)),
                                        qos: Some(publish.qos as u8),
                                    });
                                    state_guard.stats.messages_received += 1;
                                    state_guard.stats.bytes_received += payload.len() as u64;
                                    state_guard.stats.last_activity = Some(Utc::now());
                                }

                                // 规则匹配处理
                                let matched_rules = {
                                    let rules_guard = rules.read().await;
                                    rules_guard.find_matching(&topic, &payload)
                                        .into_iter()
                                        .map(|r| r.clone())
                                        .collect::<Vec<_>>()
                                };

                                if !matched_rules.is_empty() {
                                    info!("Broker: {} rules matched for topic {}", matched_rules.len(), topic);
                                    for rule in matched_rules {
                                        info!("Broker rule matched: {} - {}", rule.name, rule.id);

                                        // 执行规则动作
                                        match &rule.action {
                                            super::rules::MqttRuleAction::Log { message } => {
                                                let default_msg = format!("Rule '{}' triggered", rule.name);
                                                let log_msg = message.as_ref()
                                                    .map(|m| m.as_str())
                                                    .unwrap_or(&default_msg);
                                                info!("Rule action [Log]: {}", log_msg);
                                            }
                                            super::rules::MqttRuleAction::Respond { topic: resp_topic, payload: resp_payload, .. } => {
                                                let resp_topic = resp_topic.clone();
                                                let resp_payload_bytes = resp_payload.as_bytes().to_vec();
                                                info!("Rule action [Respond]: publishing to {} with payload {}", resp_topic, resp_payload);
                                                if let Err(e) = client.publish(&resp_topic, QoSV5::AtLeastOnce, false, resp_payload_bytes).await {
                                                    error!("Failed to publish response: {:?}", e);
                                                }
                                            }
                                            super::rules::MqttRuleAction::Forward { target_topic } => {
                                                let target_topic = target_topic.clone();
                                                info!("Rule action [Forward]: forwarding to {}", target_topic);
                                                if let Err(e) = client.publish(&target_topic, QoSV5::AtLeastOnce, false, payload.clone()).await {
                                                    error!("Failed to forward message: {:?}", e);
                                                }
                                            }
                                            super::rules::MqttRuleAction::Silence => {
                                                debug!("Rule action [Silence]: message silenced");
                                            }
                                            super::rules::MqttRuleAction::Transform { output_topic, output_payload } => {
                                                let output_topic = output_topic.clone();
                                                let output_payload_bytes = output_payload.as_bytes().to_vec();
                                                info!("Rule action [Transform]: publishing to {} with transformed payload", output_topic);
                                                if let Err(e) = client.publish(&output_topic, QoSV5::AtLeastOnce, false, output_payload_bytes).await {
                                                    error!("Failed to publish transformed message: {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    debug!("No rules matched for topic {}", topic);
                                }
                            }
                            Event::Incoming(PacketV5::ConnAck(_)) => {
                                info!("Monitor client connected to broker");
                                let mut state_guard = state.write().await;
                                state_guard.stats.total_connections += 1;
                                state_guard.stats.active_connections += 1;
                            }
                            Event::Incoming(PacketV5::Disconnect(_)) => {
                                warn!("Monitor client disconnected from broker");
                                let mut state_guard = state.write().await;
                                if state_guard.stats.active_connections > 0 {
                                    state_guard.stats.active_connections -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Monitor client eventloop error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        })
    }

    /// 创建 rumqttd 配置
    fn create_config(&self) -> Config {
        let connection_settings = ConnectionSettings {
            connection_timeout_ms: 60000,
            max_payload_size: 262144,
            max_inflight_count: 500,
            auth: None,
            external_auth: None,
            dynamic_filters: false,
        };

        let has_v4 = self.mqtt_versions.contains(&MqttVersion::V4);
        let has_v5 = self.mqtt_versions.contains(&MqttVersion::V5);

        // 根据mqtt_versions配置v4和v5服务器
        // 如果同时启用v4和v5，它们需要监听不同的端口
        // v4使用原端口，v5使用原端口+1
        let v4_servers = if has_v4 {
            let mut servers = HashMap::new();
            let listen_addr = format!("{}:{}", self.bind_addr, self.port);
            servers.insert(
                "v4-1".to_string(),
                ServerSettings {
                    name: "v4-1".to_string(),
                    listen: listen_addr.parse().unwrap_or_else(|_| {
                        format!("0.0.0.0:{}", self.port).parse().unwrap()
                    }),
                    tls: None,
                    next_connection_delay_ms: 1,
                    connections: connection_settings.clone(),
                },
            );
            info!("MQTT v4 server enabled on {}", listen_addr);
            Some(servers)
        } else {
            None
        };

        let v5_servers = if has_v5 {
            let mut servers = HashMap::new();
            // 如果v4和v5都启用，v5使用端口+1；否则使用原端口
            let v5_port = if has_v4 { self.port + 1 } else { self.port };
            let listen_addr = format!("{}:{}", self.bind_addr, v5_port);

            servers.insert(
                "v5-1".to_string(),
                ServerSettings {
                    name: "v5-1".to_string(),
                    listen: listen_addr.parse().unwrap_or_else(|_| {
                        format!("0.0.0.0:{}", v5_port).parse().unwrap()
                    }),
                    tls: None,
                    next_connection_delay_ms: 1,
                    connections: connection_settings,
                },
            );
            if has_v4 {
                info!("MQTT v5 server enabled on {} (v4 also enabled, using port+1)", listen_addr);
            } else {
                info!("MQTT v5 server enabled on {}", listen_addr);
            }
            Some(servers)
        } else {
            None
        };

        let mut router = RouterConfig::default();
        router.max_segment_size = 104857600;
        router.max_segment_count = 10;
        router.max_connections = 10000;
        router.max_outgoing_packet_count = 200;

        Config {
            id: 0,
            router,
            v4: v4_servers,
            v5: v5_servers,
            ws: None,
            cluster: None,
            console: None,
            bridge: None,
            prometheus: None,
            metrics: None,
        }
    }
}
