use crate::utils::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// 协议trait定义
///
/// 框架标准：
/// 1. 通过 from_config 创建协议实例，协议自己解析配置参数
/// 2. 通过 execute 执行命令，协议自己解析命令格式
/// 3. 框架只负责调度，不关心具体协议细节
#[async_trait]
pub trait Protocol: Send + Sync {
    /// 从配置创建协议实例
    ///
    /// # 参数
    /// - channel_id: 通道ID
    /// - params: 协议特定参数（HashMap<String, Value>），由具体协议自行解析
    ///
    /// # 示例
    /// PJLink协议期望: {"addr": "192.168.1.100", "port": 4352}
    /// Modbus协议期望: {"type": "tcp", "addr": "192.168.1.101", "port": 502, "slave_id": 1}
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>>
    where
        Self: Sized;

    /// 执行命令
    ///
    /// # 参数
    /// - command: 命令名称（由协议定义）
    /// - params: 命令参数（JSON格式，由协议解析）
    ///
    /// # 返回
    /// 命令执行结果（JSON格式）
    async fn execute(&mut self, command: &str, params: Value) -> Result<Value>;

    /// 获取状态
    async fn get_status(&self) -> Result<Value>;

    /// 写入数据（简化接口）
    async fn write(&mut self, id: u32, value: i32) -> Result<()>;

    /// 读取数据（简化接口）
    async fn read(&self, id: u32) -> Result<i32>;

    /// 协议名称
    fn name(&self) -> &str;

    /// 调用自定义方法
    ///
    /// # 参数
    /// - method_name: 方法名称
    /// - args: 方法参数（JSON格式）
    ///
    /// # 返回
    /// 方法执行结果（JSON格式）
    ///
    /// # 默认实现
    /// 返回错误，协议可以选择性实现此方法
    async fn call_method(&mut self, method_name: &str, _args: Value) -> Result<Value> {
        Err(crate::utils::DeviceError::Other(format!(
            "协议 {} 不支持自定义方法: {}",
            self.name(),
            method_name
        )))
    }

    /// 获取支持的方法列表
    ///
    /// # 返回
    /// 方法名称列表
    ///
    /// # 默认实现
    /// 返回空列表
    fn get_methods(&self) -> Vec<String> {
        vec![]
    }
}

pub mod computer_control;
pub mod custom;
pub mod hs_power_sequencer;
pub mod mock;
pub mod modbus;
pub mod modbus_slave;
pub mod novastar;
pub mod pjlink;
pub mod qn_smart_plc;
pub mod screen_njlg_plc;
pub mod splicer_3d;
pub mod storage;
pub mod tpris_pdu;
pub mod xfusion;
pub mod xinke_q1;
pub mod yk_vap;

pub use computer_control::ComputerControlProtocol;
pub use custom::CustomProtocol;
pub use hs_power_sequencer::HsPowerSequencerProtocol;
pub use mock::MockProtocol;
pub use modbus::ModbusProtocol;
pub use modbus_slave::ModbusSlaveProtocol;
pub use novastar::NovastarProtocol;
pub use pjlink::PjlinkProtocol;
pub use qn_smart_plc::QnSmartPlcProtocol;
pub use screen_njlg_plc::ScreenNjlgPlcProtocol;
pub use splicer_3d::Splicer3dProtocol;
pub use tpris_pdu::TprisPduProtocol;
pub use xfusion::XFusionProtocol;
pub use xinke_q1::XinkeQ1Protocol;
pub use yk_vap::YkVapProtocol;
