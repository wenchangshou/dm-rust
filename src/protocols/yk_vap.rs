use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tracing::{debug, error, info, trace, warn};

use crate::protocols::Protocol;
use crate::utils::{DeviceError, Result};

/// YK-VAP（文本协议）
///
/// 目前实现的核心指令：
/// - CALL: 调用场景/读取窗口列表（scene_id=0）
/// - RSCS: 读取当前场景
///
/// 报文格式示例：
/// - <CALL,1> / <CALL,1,2>
/// - <RSCS,1> / <RSCS,1,2>
///
/// 注意：该协议是按行读取（以 \n 结尾）。
pub struct YkVapProtocol {
    channel_id: u32,
    addr: String,
    port: u16,
    timeout: std::time::Duration,
    transport: YkVapTransport,
}

#[derive(Debug, Clone, Copy)]
enum YkVapTransport {
    Tcp,
    Udp,
}

impl YkVapProtocol {
    fn build_frame(cmd: &str, args: &[String]) -> String {
        // 添加换行符，部分设备需要以 \n 结尾才会响应
        let frame = if args.is_empty() {
            format!("<{}>\n", cmd)
        } else {
            format!("<{},{}>\n", cmd, args.join(","))
        };
        trace!(
            "构建帧: cmd={}, args={:?}, frame={}",
            cmd,
            args,
            frame.trim_end()
        );
        frame
    }

    fn parse_frame(line: &str) -> Option<(String, Vec<String>)> {
        let trimmed = line.trim();
        if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
            trace!("解析帧失败: 非法格式 line={}", trimmed);
            return None;
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut parts = inner.split(',').map(|s| s.trim().to_string());
        let cmd = parts.next()?;
        let args = parts.collect::<Vec<_>>();
        trace!("解析帧成功: cmd={}, args={:?}", cmd, args);
        Some((cmd, args))
    }

    async fn connect_tcp(&self) -> Result<TcpStream> {
        let addr = format!("{}:{}", self.addr, self.port);
        debug!("[channel {}] TCP 正在连接 {}...", self.channel_id, addr);

        match tokio::time::timeout(self.timeout, TcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => {
                // 禁用 Nagle 算法，确保小数据包立即发送
                if let Err(e) = stream.set_nodelay(true) {
                    warn!("[channel {}] 设置 TCP_NODELAY 失败: {}", self.channel_id, e);
                }
                info!("[channel {}] TCP 连接成功 {}", self.channel_id, addr);
                Ok(stream)
            }
            Ok(Err(e)) => {
                error!("[channel {}] TCP 连接失败 {}: {}", self.channel_id, addr, e);
                Err(DeviceError::ConnectionError(format!("连接失败: {}", e)))
            }
            Err(_) => {
                error!(
                    "[channel {}] TCP 连接超时 {} (timeout={:?})",
                    self.channel_id, addr, self.timeout
                );
                Err(DeviceError::Timeout)
            }
        }
    }

    async fn connect_udp(&self) -> Result<UdpSocket> {
        let addr = format!("{}:{}", self.addr, self.port);
        debug!("[channel {}] UDP 正在绑定本地端口...", self.channel_id);

        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => {
                debug!(
                    "[channel {}] UDP 绑定成功 local={:?}",
                    self.channel_id,
                    s.local_addr()
                );
                s
            }
            Err(e) => {
                error!("[channel {}] UDP 绑定失败: {}", self.channel_id, e);
                return Err(DeviceError::ConnectionError(format!("UDP bind失败: {}", e)));
            }
        };

        debug!("[channel {}] UDP 正在连接 {}...", self.channel_id, addr);
        match socket.connect(&addr).await {
            Ok(_) => {
                info!("[channel {}] UDP 连接成功 {}", self.channel_id, addr);
                Ok(socket)
            }
            Err(e) => {
                error!("[channel {}] UDP 连接失败 {}: {}", self.channel_id, addr, e);
                Err(DeviceError::ConnectionError(format!(
                    "UDP connect失败: {}",
                    e
                )))
            }
        }
    }

    async fn send_and_read_lines(
        &self,
        frame: &str,
        expected_cmd: &str,
    ) -> Result<Vec<(String, Vec<String>)>> {
        debug!(
            "[channel {}] 准备发送帧: transport={:?}, expected_cmd={}",
            self.channel_id, self.transport, expected_cmd
        );
        let result = match self.transport {
            YkVapTransport::Tcp => self.send_and_read_lines_tcp(frame, expected_cmd).await,
            YkVapTransport::Udp => self.send_and_read_lines_udp(frame, expected_cmd).await,
        };
        match &result {
            Ok(frames) => debug!(
                "[channel {}] 通信完成, 收到 {} 个帧",
                self.channel_id,
                frames.len()
            ),
            Err(e) => warn!("[channel {}] 通信失败: {:?}", self.channel_id, e),
        }
        result
    }

    async fn send_and_read_lines_tcp(
        &self,
        frame: &str,
        expected_cmd: &str,
    ) -> Result<Vec<(String, Vec<String>)>> {
        let mut stream = self.connect_tcp().await?;

        info!(
            "YK-VAP TX [channel {}] -> {} (hex: {:02X?})",
            self.channel_id,
            frame.trim_end(),
            frame.as_bytes()
        );

        match tokio::time::timeout(self.timeout, stream.write_all(frame.as_bytes())).await {
            Ok(Ok(_)) => {
                debug!(
                    "[channel {}] TCP 发送成功, {} bytes",
                    self.channel_id,
                    frame.len()
                );
            }
            Ok(Err(e)) => {
                error!("[channel {}] TCP 发送失败: {}", self.channel_id, e);
                return Err(DeviceError::ConnectionError(format!("发送失败: {}", e)));
            }
            Err(_) => {
                error!("[channel {}] TCP 发送超时", self.channel_id);
                return Err(DeviceError::Timeout);
            }
        }

        // 确保数据发送出去
        if let Err(e) = stream.flush().await {
            error!("[channel {}] TCP flush失败: {}", self.channel_id, e);
            return Err(DeviceError::ConnectionError(format!("flush失败: {}", e)));
        }
        info!("[channel {}] TCP flush成功", self.channel_id);

        // 使用原始字节读取，通过检测 '<' 和 '>' 来解析帧
        // 而不是依赖换行符，因为设备可能不发送换行符
        let mut out = Vec::new();
        let mut frame_count = 0u32;
        let mut buffer = Vec::with_capacity(4096);
        let mut temp_buf = [0u8; 1024];

        loop {
            trace!("[channel {}] TCP 等待接收数据...", self.channel_id);

            let n = match tokio::time::timeout(self.timeout, stream.read(&mut temp_buf)).await {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    error!("[channel {}] TCP 读取失败: {}", self.channel_id, e);
                    return Err(DeviceError::ConnectionError(format!("读取失败: {}", e)));
                }
                Err(_) => {
                    if out.is_empty() {
                        error!(
                            "[channel {}] TCP 读取超时 (无数据, buffer={:?})",
                            self.channel_id,
                            String::from_utf8_lossy(&buffer)
                        );
                    } else {
                        // 已经收到 OK，超时是正常的
                        info!(
                            "[channel {}] TCP 读取完成 (已收到 {} 帧)",
                            self.channel_id,
                            out.len()
                        );
                        break;
                    }
                    return Err(DeviceError::Timeout);
                }
            };

            if n == 0 {
                debug!("[channel {}] TCP 连接关闭 (EOF)", self.channel_id);
                break;
            }

            // 打印原始接收数据
            info!(
                "YK-VAP RX raw [channel {}] <- {} bytes: {:02X?} ({})",
                self.channel_id,
                n,
                &temp_buf[..n],
                String::from_utf8_lossy(&temp_buf[..n])
            );

            buffer.extend_from_slice(&temp_buf[..n]);

            // 解析缓冲区中的所有完整帧 <...>
            loop {
                // 查找帧起始 '<'
                let start = match buffer.iter().position(|&b| b == b'<') {
                    Some(pos) => pos,
                    None => {
                        buffer.clear(); // 没有 '<'，清空无效数据
                        break;
                    }
                };

                // 查找帧结束 '>'
                let end = match buffer[start..].iter().position(|&b| b == b'>') {
                    Some(pos) => start + pos,
                    None => break, // 帧不完整，等待更多数据
                };

                // 提取完整帧
                let frame_bytes = &buffer[start..=end];
                let frame_str = String::from_utf8_lossy(frame_bytes).to_string();

                frame_count += 1;
                info!(
                    "YK-VAP RX frame [channel {}] <- {} (frame #{})",
                    self.channel_id, frame_str, frame_count
                );

                // 解析帧
                if let Some((cmd, args)) = Self::parse_frame(&frame_str) {
                    if cmd.eq_ignore_ascii_case(expected_cmd) {
                        debug!(
                            "[channel {}] 匹配帧: cmd={}, args={:?}",
                            self.channel_id, cmd, args
                        );
                        out.push((cmd.clone(), args.clone()));

                        // 通用结束：<X,OK>
                        if args.len() == 1 && args[0].eq_ignore_ascii_case("OK") {
                            info!("[channel {}] 收到 OK 结束帧", self.channel_id);
                            // 移除已处理的数据
                            buffer.drain(..=end);
                            // 返回结果
                            info!(
                                "[channel {}] TCP 通信完成, 共收到 {} 帧, {} 个有效帧",
                                self.channel_id,
                                frame_count,
                                out.len()
                            );
                            return Ok(out);
                        }
                    } else {
                        warn!(
                            "YK-VAP ignore frame [channel {}]: cmd={} (expected={})",
                            self.channel_id, cmd, expected_cmd
                        );
                    }
                } else {
                    warn!("[channel {}] 无法解析的帧: {}", self.channel_id, frame_str);
                }

                // 移除已处理的数据
                buffer.drain(..=end);
            }
        }

        info!(
            "[channel {}] TCP 通信完成, 共收到 {} 帧, {} 个有效帧",
            self.channel_id,
            frame_count,
            out.len()
        );
        Ok(out)
    }

    async fn send_and_read_lines_udp(
        &self,
        frame: &str,
        expected_cmd: &str,
    ) -> Result<Vec<(String, Vec<String>)>> {
        let socket = self.connect_udp().await?;

        info!(
            "YK-VAP(UDP) TX [channel {}] -> {} (hex: {:02X?})",
            self.channel_id,
            frame.trim_end(),
            frame.as_bytes()
        );

        match tokio::time::timeout(self.timeout, socket.send(frame.as_bytes())).await {
            Ok(Ok(n)) => {
                debug!("[channel {}] UDP 发送成功, {} bytes", self.channel_id, n);
            }
            Ok(Err(e)) => {
                error!("[channel {}] UDP 发送失败: {}", self.channel_id, e);
                return Err(DeviceError::ConnectionError(format!("UDP发送失败: {}", e)));
            }
            Err(_) => {
                error!("[channel {}] UDP 发送超时", self.channel_id);
                return Err(DeviceError::Timeout);
            }
        }

        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        let mut recv_count = 0u32;

        loop {
            let mut buf = vec![0u8; 4096];
            trace!("[channel {}] UDP 等待接收数据...", self.channel_id);

            let n = match tokio::time::timeout(self.timeout, socket.recv(&mut buf)).await {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => {
                    error!("[channel {}] UDP 读取失败: {}", self.channel_id, e);
                    return Err(DeviceError::ConnectionError(format!("UDP读取失败: {}", e)));
                }
                Err(_) => {
                    if out.is_empty() {
                        error!("[channel {}] UDP 读取超时 (无数据)", self.channel_id);
                    } else {
                        warn!(
                            "[channel {}] UDP 读取超时 (已收到 {} 帧)",
                            self.channel_id,
                            out.len()
                        );
                    }
                    return Err(DeviceError::Timeout);
                }
            };

            if n == 0 {
                debug!("[channel {}] UDP 收到空包, 继续等待", self.channel_id);
                continue;
            }

            recv_count += 1;
            let text = String::from_utf8_lossy(&buf[..n]);
            debug!(
                "[channel {}] UDP 收到数据包 #{}, {} bytes",
                self.channel_id, recv_count, n
            );

            info!(
                "YK-VAP(UDP) RX raw [channel {}]: {} (hex: {:02X?})",
                self.channel_id,
                text.trim_end(),
                &buf[..n]
            );
            for line in text.lines() {
                info!("YK-VAP(UDP) RX [channel {}] <- {}", self.channel_id, line);

                if let Some((cmd, args)) = Self::parse_frame(line) {
                    if cmd.eq_ignore_ascii_case(expected_cmd) {
                        debug!(
                            "[channel {}] 匹配帧: cmd={}, args={:?}",
                            self.channel_id, cmd, args
                        );
                        out.push((cmd.clone(), args.clone()));
                        if args.len() == 1 && args[0].eq_ignore_ascii_case("OK") {
                            info!(
                                "[channel {}] UDP 收到 OK 结束帧, 共收到 {} 个数据包",
                                self.channel_id, recv_count
                            );
                            return Ok(out);
                        }
                    } else {
                        warn!(
                            "YK-VAP(UDP) ignore frame [channel {}]: cmd={} (expected={})",
                            self.channel_id, cmd, expected_cmd
                        );
                    }
                } else {
                    warn!("[channel {}] UDP 无法解析的帧: {}", self.channel_id, line);
                }
            }
        }
    }
}

#[async_trait]
impl Protocol for YkVapProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        let transport = match params
            .get("type")
            .or_else(|| params.get("transport"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("udp") => YkVapTransport::Udp,
            Some("tcp") | None => YkVapTransport::Tcp,
            Some(other) => {
                return Err(DeviceError::ConfigError(format!(
                    "YK-VAP type 仅支持 tcp/udp，实际: {}",
                    other
                )))
            }
        };

        let addr = params
            .get("addr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeviceError::ConfigError("YK-VAP缺少addr参数".into()))?
            .to_string();

        let port = params
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| DeviceError::ConfigError("YK-VAP缺少port参数".into()))?
            as u16;

        let timeout_ms = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(3000);

        info!(
            "YK-VAP 协议初始化: channel_id={}, addr={}:{}, transport={:?}, timeout={}ms",
            channel_id, addr, port, transport, timeout_ms
        );

        Ok(Box::new(Self {
            channel_id,
            addr,
            port,
            timeout: std::time::Duration::from_millis(timeout_ms),
            transport,
        }))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!(
            "[channel {}] 执行命令: command={}, params={}",
            self.channel_id, command, params
        );

        match command {
            "call_scene" | "call" => {
                let scene_id =
                    params
                        .get("scene_id")
                        .and_then(|v| v.as_u64())
                        .ok_or_else(|| {
                            error!(
                                "[channel {}] call_scene 缺少 scene_id 参数",
                                self.channel_id
                            );
                            DeviceError::Other("缺少 scene_id 参数".to_string())
                        })?;

                let group = params.get("group").and_then(|v| v.as_u64());

                let mut args = vec![scene_id.to_string()];
                if let Some(g) = group {
                    args.push(g.to_string());
                }

                debug!(
                    "[channel {}] call_scene: scene_id={}, group={:?}",
                    self.channel_id, scene_id, group
                );

                let frame = Self::build_frame("CALL", &args);
                let frames = self.send_and_read_lines(&frame, "CALL").await?;

                // 1) scene_id == 0：返回窗口列表直到 OK
                // 2) scene_id != 0：可能只返回 <CALL,OK> 或也返回窗口信息
                let mut windows = Vec::new();
                let mut ok = false;

                for (_cmd, a) in frames {
                    if a.len() == 1 && a[0].eq_ignore_ascii_case("OK") {
                        ok = true;
                        continue;
                    }
                    // <CALL,W_ID,Channel,x0,y0,x1,y1,SubChannel>
                    if a.len() == 7 {
                        let w_id = a[0].parse::<u32>().ok();
                        let channel = a[1].parse::<u32>().ok();
                        let x0 = a[2].parse::<i32>().ok();
                        let y0 = a[3].parse::<i32>().ok();
                        let x1 = a[4].parse::<i32>().ok();
                        let y1 = a[5].parse::<i32>().ok();
                        let sub_channel = a[6].parse::<u32>().ok();

                        windows.push(json!({
                            "w_id": w_id,
                            "channel": channel,
                            "x0": x0,
                            "y0": y0,
                            "x1": x1,
                            "y1": y1,
                            "sub_channel": sub_channel,
                            "raw": a,
                        }));
                    } else {
                        // 其它帧格式先原样返回，避免丢信息
                        windows.push(json!({"raw": a}));
                    }
                }

                if !ok {
                    error!("[channel {}] call_scene 未收到 OK 结束帧", self.channel_id);
                    return Err(DeviceError::ProtocolError(
                        "未收到 <CALL,OK> 结束帧".to_string(),
                    ));
                }

                info!(
                    "[channel {}] call_scene 成功: scene_id={}, windows={}",
                    self.channel_id,
                    scene_id,
                    windows.len()
                );

                Ok(json!({
                    "status": "ok",
                    "scene_id": scene_id,
                    "group": group,
                    "windows": windows
                }))
            }

            "read_scene" | "rscs" => {
                let wall_index = params
                    .get("wallIndex")
                    .or_else(|| params.get("wall_index"))
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        error!(
                            "[channel {}] read_scene 缺少 wallIndex 参数",
                            self.channel_id
                        );
                        DeviceError::Other("缺少 wallIndex 参数".to_string())
                    })?;

                let group = params.get("group").and_then(|v| v.as_u64());

                let mut args = vec![wall_index.to_string()];
                if let Some(g) = group {
                    args.push(g.to_string());
                }

                debug!(
                    "[channel {}] read_scene: wall_index={}, group={:?}",
                    self.channel_id, wall_index, group
                );

                let frame = Self::build_frame("RSCS", &args);
                let frames = self.send_and_read_lines(&frame, "RSCS").await?;

                // 期望：
                // <RSCS,index>
                // <RSCS,OK>
                let mut index: Option<u64> = None;
                let mut ok = false;
                for (_cmd, a) in frames {
                    if a.len() == 1 && a[0].eq_ignore_ascii_case("OK") {
                        ok = true;
                    } else if a.len() == 1 {
                        index = a[0].parse::<u64>().ok();
                    }
                }

                if !ok {
                    error!("[channel {}] read_scene 未收到 OK 结束帧", self.channel_id);
                    return Err(DeviceError::ProtocolError(
                        "未收到 <RSCS,OK> 结束帧".to_string(),
                    ));
                }

                info!(
                    "[channel {}] read_scene 成功: wall_index={}, index={:?}",
                    self.channel_id, wall_index, index
                );

                Ok(json!({
                    "status": "ok",
                    "wallIndex": wall_index,
                    "group": group,
                    "index": index
                }))
            }

            _ => {
                error!("[channel {}] 不支持的命令: {}", self.channel_id, command);
                Err(DeviceError::ProtocolError(format!(
                    "YK-VAP 不支持的命令: {}",
                    command
                )))
            }
        }
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(json!({
            "protocol": "yk-vap",
            "channel_id": self.channel_id,
            "addr": format!("{}:{}", self.addr, self.port),
            "type": match self.transport {
                YkVapTransport::Tcp => "tcp",
                YkVapTransport::Udp => "udp",
            },
            "connected": true
        }))
    }

    async fn write(&mut self, _id: u32, value: i32) -> Result<()> {
        // 将 value 作为 scene_id 调用场景
        let scene_id = value as u64;
        info!("[channel {}] write: scene_id={}", self.channel_id, scene_id);

        let args = vec![scene_id.to_string()];
        let frame = Self::build_frame("CALL", &args);
        let frames = self.send_and_read_lines(&frame, "CALL").await?;

        // 检查是否收到 OK 响应
        for (_cmd, a) in frames {
            if a.len() == 1 && a[0].eq_ignore_ascii_case("OK") {
                info!("[channel {}] write 成功", self.channel_id);
                return Ok(());
            }
        }

        error!("[channel {}] write 未收到 OK 结束帧", self.channel_id);
        Err(DeviceError::ProtocolError(
            "未收到 <CALL,OK> 结束帧".to_string(),
        ))
    }

    async fn read(&self, _id: u32) -> Result<i32> {
        warn!(
            "[channel {}] read 不支持, 请使用 execute(read_scene)",
            self.channel_id
        );
        Err(DeviceError::ProtocolError(
            "YK-VAP 不支持简化 read(id)；请使用 execute(read_scene 等)".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "yk-vap"
    }

    fn get_methods(&self) -> Vec<String> {
        vec!["call_scene".to_string(), "read_scene".to_string()]
    }
}
