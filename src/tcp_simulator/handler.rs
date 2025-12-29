/// TCP 协议处理器 trait
///
/// 所有协议模拟器必须实现此 trait，定义协议的解析和响应逻辑。
///
/// # 示例
/// ```rust
/// use async_trait::async_trait;
///
/// pub struct MyProtocolHandler;
///
/// #[async_trait]
/// impl ProtocolHandler for MyProtocolHandler {
///     fn name(&self) -> &str { "my_protocol" }
///     fn description(&self) -> &str { "自定义协议" }
///     fn default_port(&self) -> u16 { 5000 }
///
///     async fn handle(&self, data: &[u8], state: &mut SimulatorState) -> HandleResult {
///         // 解析数据，更新状态，返回响应
///         HandleResult::Response(vec![0x01, 0x02])
///     }
/// }
/// ```

use async_trait::async_trait;
use serde_json::Value;

use super::state::SimulatorState;

/// 协议处理结果
#[derive(Debug, Clone)]
pub enum HandleResult {
    /// 返回响应数据
    Response(Vec<u8>),
    /// 需要更多数据（用于分帧协议）
    NeedMoreData,
    /// 无响应（静默处理）
    NoResponse,
    /// 处理错误
    Error(String),
}

/// 协议处理器 trait
///
/// 每个协议实现此 trait 来定义：
/// - 协议基本信息（名称、描述、默认端口）
/// - 数据处理逻辑（解析命令、生成响应）
/// - 可选的初始化和清理逻辑
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// 协议唯一标识名称
    fn name(&self) -> &str;

    /// 协议描述
    fn description(&self) -> &str;

    /// 默认监听端口
    fn default_port(&self) -> u16;

    /// 处理接收到的数据
    ///
    /// # 参数
    /// - `data`: 接收到的原始字节数据
    /// - `state`: 模拟器状态（可修改）
    ///
    /// # 返回
    /// - `HandleResult::Response(bytes)`: 返回响应数据
    /// - `HandleResult::NeedMoreData`: 数据不完整，等待更多数据
    /// - `HandleResult::NoResponse`: 处理成功但无需响应
    /// - `HandleResult::Error(msg)`: 处理错误
    async fn handle(&self, data: &[u8], state: &mut SimulatorState) -> HandleResult;

    /// 获取协议支持的命令列表（用于文档/调试）
    fn supported_commands(&self) -> Vec<String> {
        vec![]
    }

    /// 连接建立时的回调
    async fn on_connect(&self, _state: &mut SimulatorState) -> Option<Vec<u8>> {
        None
    }

    /// 连接断开时的回调
    async fn on_disconnect(&self, _state: &mut SimulatorState) {
        // 默认空实现
    }

    /// 获取协议元数据（用于 API 展示）
    fn metadata(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "description": self.description(),
            "default_port": self.default_port(),
            "commands": self.supported_commands()
        })
    }
}
