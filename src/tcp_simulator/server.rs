/// TCP 模拟服务器
///
/// 管理单个 TCP 监听器，处理客户端连接和数据收发。

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::handler::{HandleResult, ProtocolHandler};
use super::state::{SimulatorState, SimulatorStatus};

/// TCP 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub port: u16,
    pub buffer_size: usize,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 5000,
            buffer_size: 1024,
            max_connections: 100,
        }
    }
}

/// TCP 模拟服务器
pub struct TcpSimulatorServer {
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

impl TcpSimulatorServer {
    /// 创建新的 TCP 服务器
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

    /// 启动服务器
    pub async fn start(&mut self) -> Result<(), String> {
        // 检查是否已经运行
        {
            let status = self.status.read().await;
            if *status == SimulatorStatus::Running {
                return Err("Server is already running".to_string());
            }
        }

        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);

        // 绑定端口
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind {}: {}", addr, e))?;

        info!("TCP 模拟服务器启动: {} (协议: {})", addr, self.handler.name());

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
            Self::run_server(listener, handler, state, status, shutdown_tx, buffer_size).await;
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    /// 停止服务器
    pub async fn stop(&mut self) -> Result<(), String> {
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

        info!("TCP 模拟服务器已停止");
        Ok(())
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> SimulatorState {
        self.state.read().await.clone()
    }

    /// 修改状态
    pub async fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut SimulatorState),
    {
        let mut state = self.state.write().await;
        f(&mut state);
    }

    /// 获取运行状态
    pub async fn get_status(&self) -> SimulatorStatus {
        *self.status.read().await
    }

    /// 服务器主循环
    async fn run_server(
        listener: TcpListener,
        handler: Arc<dyn ProtocolHandler>,
        state: Arc<RwLock<SimulatorState>>,
        status: Arc<RwLock<SimulatorStatus>>,
        shutdown_tx: broadcast::Sender<()>,
        buffer_size: usize,
    ) {
        let mut shutdown_rx = shutdown_tx.subscribe();

        // 启动值生成器任务（如果是 Modbus 协议）
        let generator_handle = if handler.name() == "modbus" {
            let state_clone = state.clone();
            let mut gen_shutdown_rx = shutdown_tx.subscribe();
            Some(tokio::spawn(async move {
                Self::run_generator_tick(state_clone, &mut gen_shutdown_rx).await;
            }))
        } else {
            None
        };

        loop {
            tokio::select! {
                // 等待新连接
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            info!("新连接: {}", peer_addr);

                            // 更新连接统计
                            {
                                let mut state = state.write().await;
                                state.stats.record_connection();
                            }

                            // 为每个连接创建处理任务
                            let handler = handler.clone();
                            let state = state.clone();
                            let mut shutdown_rx = shutdown_tx.subscribe();

                            tokio::spawn(async move {
                                Self::handle_connection(
                                    stream,
                                    handler,
                                    state,
                                    &mut shutdown_rx,
                                    buffer_size,
                                )
                                .await;
                            });
                        }
                        Err(e) => {
                            error!("接受连接失败: {}", e);
                        }
                    }
                }

                // 等待停止信号
                _ = shutdown_rx.recv() => {
                    info!("收到停止信号");
                    break;
                }
            }
        }

        // 停止生成器任务
        if let Some(handle) = generator_handle {
            handle.abort();
        }

        // 更新状态
        {
            let mut status_guard = status.write().await;
            *status_guard = SimulatorStatus::Stopped;
        }
    }

    /// 值生成器定时任务
    async fn run_generator_tick(
        state: Arc<RwLock<SimulatorState>>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) {
        use super::protocols::ModbusValues;

        // 每 100ms 检查一次生成器
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let mut state_guard = state.write().await;
                    let mut values = ModbusValues::from_state(&state_guard);
                    if values.tick_generators() {
                        values.save_to_state(&mut state_guard);
                    }
                }

                _ = shutdown_rx.recv() => {
                    debug!("生成器任务收到停止信号");
                    break;
                }
            }
        }
    }

    /// 处理单个连接
    async fn handle_connection(
        mut stream: TcpStream,
        handler: Arc<dyn ProtocolHandler>,
        state: Arc<RwLock<SimulatorState>>,
        shutdown_rx: &mut broadcast::Receiver<()>,
        buffer_size: usize,
    ) {
        let peer_addr = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // 连接建立回调
        {
            let mut state_guard = state.write().await;
            if let Some(welcome) = handler.on_connect(&mut state_guard).await {
                if let Err(e) = stream.write_all(&welcome).await {
                    warn!("发送欢迎消息失败: {}", e);
                }
            }
        }

        let mut buffer = vec![0u8; buffer_size];
        let mut accumulated_data = Vec::new();

        loop {
            tokio::select! {
                // 读取数据
                result = stream.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            // 连接关闭
                            debug!("连接关闭: {}", peer_addr);
                            break;
                        }
                        Ok(n) => {
                            debug!("收到 {} 字节: {:02x?}", n, &buffer[..n]);

                            let received_data = &buffer[..n];

                            // 更新统计并记录接收报文
                            {
                                let mut state_guard = state.write().await;
                                state_guard.stats.record_received(n as u64);
                                state_guard.packet_monitor.record_received(
                                    &peer_addr,
                                    received_data,
                                    None,
                                );
                            }

                            // 累积数据
                            accumulated_data.extend_from_slice(received_data);

                            // 处理数据
                            let result = {
                                let mut state_guard = state.write().await;
                                handler.handle(&accumulated_data, &mut state_guard).await
                            };

                            match result {
                                HandleResult::Response(response) => {
                                    debug!("发送响应: {:02x?}", response);

                                    // 更新统计并记录发送报文
                                    {
                                        let mut state_guard = state.write().await;
                                        state_guard.stats.record_sent(response.len() as u64);
                                        state_guard.packet_monitor.record_sent(
                                            &peer_addr,
                                            &response,
                                            None,
                                        );
                                    }

                                    if let Err(e) = stream.write_all(&response).await {
                                        error!("发送响应失败: {}", e);
                                        break;
                                    }

                                    // 清空累积数据
                                    accumulated_data.clear();
                                }
                                HandleResult::NeedMoreData => {
                                    // 等待更多数据
                                    debug!("等待更多数据");
                                }
                                HandleResult::NoResponse => {
                                    // 无需响应
                                    accumulated_data.clear();
                                }
                                HandleResult::Error(msg) => {
                                    warn!("处理错误: {}", msg);
                                    accumulated_data.clear();
                                }
                            }
                        }
                        Err(e) => {
                            error!("读取数据失败: {}", e);
                            break;
                        }
                    }
                }

                // 停止信号
                _ = shutdown_rx.recv() => {
                    debug!("连接处理收到停止信号");
                    break;
                }
            }
        }

        // 连接断开回调
        {
            let mut state_guard = state.write().await;
            handler.on_disconnect(&mut state_guard).await;
            state_guard.stats.record_disconnection();
        }

        debug!("连接处理结束: {}", peer_addr);
    }
}

impl Drop for TcpSimulatorServer {
    fn drop(&mut self) {
        // 发送停止信号
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
