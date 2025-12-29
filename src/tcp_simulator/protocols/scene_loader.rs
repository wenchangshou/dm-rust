/// 场景加载协议模拟器
///
/// 协议格式：
/// - 命令帧 (21 bytes): 55 aa [data...] XX SUM_L SUM_H
/// - 响应帧 (20 bytes): aa 55 [data...] SUM_L SUM_H
///
/// 其中:
/// - 55 aa: 命令帧头
/// - aa 55: 响应帧头
/// - XX: 场景号 (0x00-0x09 对应场景 1-10)
/// - SUM = 数据字节和 + 0x5555
/// - SUM_L: 校验和低8位
/// - SUM_H: 校验和高8位

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::tcp_simulator::handler::{HandleResult, ProtocolHandler};
use crate::tcp_simulator::state::SimulatorState;

/// 场景加载协议处理器
pub struct SceneLoaderHandler {
    /// 命令帧头
    frame_header: [u8; 2],
    /// 响应帧头
    response_header: [u8; 2],
    /// 预期的命令长度
    command_length: usize,
    /// 预期的固定数据部分
    expected_data: [u8; 16],
}

impl SceneLoaderHandler {
    pub fn new() -> Self {
        Self {
            frame_header: [0x55, 0xaa],
            response_header: [0xaa, 0x55],
            command_length: 21,
            // 固定数据部分: 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00
            expected_data: [
                0x00, 0x00, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x01, 0x00, 0x00, 0x01, 0x51, 0x13, 0x01, 0x00,
            ],
        }
    }

    /// 计算校验和
    ///
    /// SUM = 数据字节和 + 0x5555
    fn calculate_checksum(data: &[u8]) -> u16 {
        let sum: u16 = data.iter().map(|&b| b as u16).sum();
        sum.wrapping_add(0x5555)
    }

    /// 验证命令校验和
    fn verify_checksum(&self, data: &[u8]) -> bool {
        if data.len() < self.command_length {
            return false;
        }

        // 提取数据部分（不含帧头和校验和）
        let payload = &data[2..19]; // 索引 2-18，共17字节
        let calculated = Self::calculate_checksum(payload);

        // 提取接收的校验和（小端序）
        let received = (data[19] as u16) | ((data[20] as u16) << 8);

        let valid = calculated == received;
        if !valid {
            debug!(
                "校验和验证失败: 计算={:04x}, 接收={:04x}",
                calculated, received
            );
        }
        valid
    }

    /// 解析命令，返回场景号
    fn parse_command(&self, data: &[u8]) -> Option<u8> {
        // 检查长度
        if data.len() < self.command_length {
            debug!("数据长度不足: {} < {}", data.len(), self.command_length);
            return None;
        }

        // 检查帧头
        if data[0..2] != self.frame_header {
            debug!(
                "帧头不匹配: {:02x?} != {:02x?}",
                &data[0..2],
                self.frame_header
            );
            return None;
        }

        // 检查固定数据部分
        if data[2..18] != self.expected_data {
            debug!("固定数据不匹配");
            return None;
        }

        // 验证校验和
        if !self.verify_checksum(data) {
            warn!("校验和验证失败");
            // 仍然返回场景号，但记录警告
        }

        // 提取场景号
        let scene = data[18];
        if scene > 9 {
            warn!("场景号超出范围: {}", scene);
        }

        Some(scene)
    }

    /// 构建响应数据
    fn build_response(&self, _scene: u8) -> Vec<u8> {
        // 响应固定数据: 00 00 00 fe 00 00 00 00 01 00 00 01 51 13 00 00
        let response_data: [u8; 16] = [
            0x00, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x01, 0x51, 0x13, 0x00, 0x00,
        ];

        // 计算校验和
        let checksum = Self::calculate_checksum(&response_data);

        // 构建完整响应
        let mut response = Vec::with_capacity(20);
        response.extend_from_slice(&self.response_header);
        response.extend_from_slice(&response_data);
        response.push((checksum & 0xff) as u8); // SUM_L
        response.push((checksum >> 8) as u8); // SUM_H

        response
    }
}

impl Default for SceneLoaderHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolHandler for SceneLoaderHandler {
    fn name(&self) -> &str {
        "scene_loader"
    }

    fn description(&self) -> &str {
        "场景加载协议 - 用于控制场景切换的二进制协议"
    }

    fn default_port(&self) -> u16 {
        5000
    }

    fn supported_commands(&self) -> Vec<String> {
        vec![
            "load_scene(0-9)".to_string(),
        ]
    }

    async fn handle(&self, data: &[u8], state: &mut SimulatorState) -> HandleResult {
        // 检查设备是否在线
        if !state.online {
            debug!("设备离线，忽略命令");
            return HandleResult::NoResponse;
        }

        // 检查是否有故障
        if let Some(fault) = &state.fault {
            debug!("设备故障: {}", fault);
            return HandleResult::NoResponse;
        }

        // 检查数据长度
        if data.len() < self.command_length {
            return HandleResult::NeedMoreData;
        }

        // 解析命令
        match self.parse_command(data) {
            Some(scene) => {
                let scene_number = scene + 1; // 转换为 1-10
                debug!("加载场景 {}", scene_number);

                // 更新状态
                state.set_value("current_scene", serde_json::json!(scene_number));
                state.set_value("last_command", serde_json::json!("load_scene"));
                state.set_value("last_scene_raw", serde_json::json!(scene));

                // 构建响应
                let response = self.build_response(scene);
                debug!("发送响应: {:02x?}", response);

                HandleResult::Response(response)
            }
            None => {
                warn!("无法解析命令: {:02x?}", data);
                HandleResult::Error("Invalid command format".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        // 测试校验和计算
        let data = [
            0x00, 0x00, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x01, 0x51, 0x13, 0x01, 0x00, 0x00,
        ];
        let checksum = SceneLoaderHandler::calculate_checksum(&data);
        // 0x00+0x00+0xfe+...+0x00+0x00 + 0x5555 = 0x56ba
        assert_eq!(checksum, 0x56ba, "场景1校验和计算错误");
    }

    #[test]
    fn test_parse_scene1_command() {
        let handler = SceneLoaderHandler::new();

        // 场景1命令: 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 00 ba 56
        let cmd: [u8; 21] = [
            0x55, 0xaa, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
            0x01, 0x00, 0x00, 0xba, 0x56,
        ];

        let scene = handler.parse_command(&cmd);
        assert_eq!(scene, Some(0), "应解析出场景0（场景1）");
    }

    #[test]
    fn test_parse_scene2_command() {
        let handler = SceneLoaderHandler::new();

        // 场景2命令: 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 01 bb 56
        let cmd: [u8; 21] = [
            0x55, 0xaa, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
            0x01, 0x00, 0x01, 0xbb, 0x56,
        ];

        let scene = handler.parse_command(&cmd);
        assert_eq!(scene, Some(1), "应解析出场景1（场景2）");
    }

    #[test]
    fn test_build_response() {
        let handler = SceneLoaderHandler::new();
        let response = handler.build_response(0);

        // 预期响应: aa 55 00 00 00 fe 00 00 00 00 01 00 00 01 51 13 00 00 b9 56
        assert_eq!(response.len(), 20, "响应长度应为20字节");
        assert_eq!(response[0], 0xaa, "响应帧头[0]错误");
        assert_eq!(response[1], 0x55, "响应帧头[1]错误");
        assert_eq!(response[18], 0xb9, "校验和低位错误");
        assert_eq!(response[19], 0x56, "校验和高位错误");
    }

    #[tokio::test]
    async fn test_handle_command() {
        let handler = SceneLoaderHandler::new();
        let mut state = SimulatorState::new();

        let cmd: [u8; 21] = [
            0x55, 0xaa, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
            0x01, 0x00, 0x00, 0xba, 0x56,
        ];

        let result = handler.handle(&cmd, &mut state).await;

        match result {
            HandleResult::Response(resp) => {
                assert_eq!(resp.len(), 20);
            }
            _ => panic!("应返回Response"),
        }

        // 检查状态更新
        assert_eq!(state.get_i32("current_scene"), Some(1));
    }

    #[tokio::test]
    async fn test_offline_device() {
        let handler = SceneLoaderHandler::new();
        let mut state = SimulatorState::new();
        state.online = false;

        let cmd: [u8; 21] = [
            0x55, 0xaa, 0x00, 0x00, 0xfe, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
            0x01, 0x00, 0x00, 0xba, 0x56,
        ];

        let result = handler.handle(&cmd, &mut state).await;

        match result {
            HandleResult::NoResponse => {}
            _ => panic!("离线设备应返回NoResponse"),
        }
    }
}
