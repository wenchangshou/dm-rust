use super::handler::{HandleResult, ProtocolHandler};
use super::server::ServerConfig;
use super::state::{ClientConnection, PacketDirection, SimulatorState, SimulatorStatus};
use super::transport::SimulatorServer;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// UDP 模拟服务器
pub struct UdpServer {
    /// 服务器配置
    config: ServerConfig,
    /// 协议处理器
    handler: Arc<dyn ProtocolHandler>,
    /// 模拟器状态
    state: Arc<RwLock<SimulatorState>>,
    /// 运行状态
    status: Arc<RwLock<SimulatorStatus>>,
    /// 停止信号发送器
    shutdown_tx: Option<broadcast::Sender<()>>,
    /// 服务器任务句柄
    server_handle: Option<JoinHandle<()>>,
}

#[async_trait]
impl SimulatorServer for UdpServer {
    async fn start(&mut self) -> Result<(), String> {
        // 检查是否已经运行
        {
            let status = self.status.read().await;
            if *status == SimulatorStatus::Running {
                return Err("Server is already running".to_string());
            }
        }

        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);

        // 绑定端口
        let socket = UdpSocket::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind UDP {}: {}", addr, e))?;

        info!(
            "UDP 模拟服务器启动: {} (协议: {})",
            addr,
            self.handler.name()
        );

        // 创建停止信号通道
        let (shutdown_tx, _) = broadcast::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = SimulatorStatus::Running;
        }

        // 启动服务器任务
        let handler = self.handler.clone();
        let state = self.state.clone();
        let status = self.status.clone();
        let buffer_size = self.config.buffer_size;

        let handle = tokio::spawn(async move {
            Self::run_server(socket, handler, state, status, shutdown_tx, buffer_size).await;
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        // 发送停止信号
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // 等待服务器任务结束
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }

        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = SimulatorStatus::Stopped;
        }

        info!("UDP 模拟服务器已停止");
        Ok(())
    }

    fn get_state_ref(&self) -> Arc<RwLock<SimulatorState>> {
        self.state.clone()
    }

    fn get_status_ref(&self) -> Arc<RwLock<SimulatorStatus>> {
        self.status.clone()
    }
}

impl UdpServer {
    /// 创建新的 UDP 服务器
    pub fn new(
        config: ServerConfig,
        handler: Arc<dyn ProtocolHandler>,
        initial_state: SimulatorState,
    ) -> Self {
        Self {
            config,
            handler,
            state: Arc::new(RwLock::new(initial_state)),
            status: Arc::new(RwLock::new(SimulatorStatus::Stopped)),
            shutdown_tx: None,
            server_handle: None,
        }
    }

    /// 服务器主循环
    async fn run_server(
        socket: UdpSocket,
        handler: Arc<dyn ProtocolHandler>,
        state: Arc<RwLock<SimulatorState>>,
        status: Arc<RwLock<SimulatorStatus>>,
        shutdown_rx: broadcast::Sender<()>,
        buffer_size: usize,
    ) {
        let mut shutdown_recv = shutdown_rx.subscribe();
        let mut buf = vec![0u8; buffer_size];
        let socket = Arc::new(socket);

        loop {
            tokio::select! {
                _ = shutdown_recv.recv() => {
                    info!("收到停止信号，UDP 服务器正在关闭...");
                    break;
                }
                res = socket.recv_from(&mut buf) => {
                    match res {
                        Ok((n, peer_addr)) => {
                            if n == 0 {
                                continue;
                            }

                            let data = &buf[0..n];
                            let socket_clone = socket.clone();
                            let handler_clone = handler.clone();
                            let state_clone = state.clone();
                            let data_vec = data.to_vec();

                            // 处理每个请求
                            // 注意：UDP 是无连接的，所以这里简单处理每个数据包
                            // 如果处理逻辑很慢，可能会阻塞接收循环，所以可以 spawn 任务
                            // 但为了简化和避免过度并发，这里先顺序处理，或者 spawn
                            tokio::spawn(async move {
                                Self::handle_packet(
                                    socket_clone,
                                    peer_addr,
                                    data_vec,
                                    handler_clone,
                                    state_clone
                                ).await;
                            });
                        }
                        Err(e) => {
                            error!("UDP 接收错误: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                else => {
                     break;
                }
            }
        }

        // 更新状态为已停止
        let mut s = status.write().await;
        *s = SimulatorStatus::Stopped;
    }

    async fn handle_packet(
        socket: Arc<UdpSocket>,
        peer_addr: std::net::SocketAddr,
        data: Vec<u8>,
        handler: Arc<dyn ProtocolHandler>,
        state: Arc<RwLock<SimulatorState>>,
    ) {
        let client_id = peer_addr.to_string();

        // 更新状态：记录接收到的数据
        {
            let mut state_guard = state.write().await;

            // 简单的“连接”跟踪（仅用于统计和日志，UDP无实际连接）
            if !state_guard.clients.contains_key(&client_id) {
                state_guard.clients.insert(
                    client_id.clone(),
                    ClientConnection {
                        id: client_id.clone(),
                        peer_addr: peer_addr.to_string(),
                        connected_at: chrono::Utc::now(),
                        last_activity: chrono::Utc::now(),
                        bytes_received: 0,
                        bytes_sent: 0,
                    },
                );
                state_guard.stats.record_connection();
            } else if let Some(client) = state_guard.clients.get_mut(&client_id) {
                client.last_activity = chrono::Utc::now();
            }

            state_guard.packet_monitor.record(
                PacketDirection::Received,
                &client_id,
                &data,
                None, // UDP 通常没有头部解析逻辑，或视协议而定
            );
            state_guard.stats.record_received(data.len() as u64);
        }

        // 调用协议处理器
        // 注意：ProtocolHandler 接口目前假设有 state 访问权
        // 对于 UDP，可能需要特殊的 state 处理?
        // handle 方法签名: async fn handle(&self, data: &[u8], state: &mut SimulatorState) -> HandleResult

        let result = {
            let mut state_guard = state.write().await;
            handler.handle(&data, &mut state_guard).await
        };

        match result {
            HandleResult::Response(resp_data) => {
                // 更新状态：记录发送的数据
                {
                    let mut state_guard = state.write().await;
                    state_guard.packet_monitor.record(
                        PacketDirection::Sent,
                        &client_id,
                        &resp_data,
                        None,
                    );
                    state_guard.stats.record_sent(resp_data.len() as u64);
                }

                // 发送响应
                if let Err(e) = socket.send_to(&resp_data, peer_addr).await {
                    error!("UDP 发送响应失败到 {}: {}", peer_addr, e);
                }
            }
            HandleResult::NoResponse | HandleResult::NeedMoreData => {
                // 无需响应
            }
            HandleResult::Error(e) => {
                error!("协议处理错误: {}", e);
            }
        }
    }
}
