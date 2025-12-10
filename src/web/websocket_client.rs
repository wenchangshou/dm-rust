use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use crate::config::WebSocketConfig;
use crate::device::DeviceController;
use crate::utils::Result;
use tracing::{info, warn, error};

#[derive(Serialize, Deserialize, Debug)]
struct WsMessage {
    #[serde(rename = "messageType")]
    message_type: Option<String>,
    #[serde(rename = "SocketName")]
    socket_name: Option<String>,
    #[serde(rename = "SocketType")]
    socket_type: Option<String>,
    #[serde(rename = "Service")]
    service: Option<String>,
    #[serde(rename = "Action")]
    action: Option<String>,
    #[serde(flatten)]
    other: serde_json::Value,
}

pub async fn run(config: WebSocketConfig, controller: DeviceController) -> Result<()> {
    let url = format!("ws://{}:{}", config.ip, config.port);
    
    loop {
        info!("连接到WebSocket服务器: {}", url);
        
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("WebSocket连接成功");
                
                let (mut write, mut read) = ws_stream.split();
                
                // 发送注册消息
                let register_msg = WsMessage {
                    message_type: Some("RegisterToDaemon".to_string()),
                    socket_name: Some(config.socket_name.clone()),
                    socket_type: Some(config.socket_type.clone()),
                    service: None,
                    action: None,
                    other: serde_json::json!({}),
                };
                
                if let Ok(msg_str) = serde_json::to_string(&register_msg) {
                    if let Err(e) = write.send(Message::Text(msg_str)).await {
                        warn!("发送注册消息失败: {:?}", e);
                    }
                }
                
                // 处理消息
                while let Some(msg_result) = read.next().await {
                    match msg_result {
                        Ok(Message::Text(text)) => {
                            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                handle_message(ws_msg, &controller, &mut write).await;
                            }
                        }
                        Ok(Message::Ping(_)) => {
                            // 自动回复Pong
                        }
                        Ok(Message::Close(_)) => {
                            info!("WebSocket连接关闭");
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket错误: {:?}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("WebSocket连接失败: {:?}", e);
            }
        }
        
        // 重连延迟
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn handle_message(
    msg: WsMessage,
    controller: &DeviceController,
    write: &mut futures::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message
    >,
) {
    if let Some(action) = &msg.action {
        match action.as_str() {
            "write" => {
                // TODO: 处理写入命令
                info!("收到写入命令: {:?}", msg);
            }
            "execute" => {
                // TODO: 处理执行命令
                info!("收到执行命令: {:?}", msg);
            }
            _ => {
                warn!("未知命令: {}", action);
            }
        }
    }
}
