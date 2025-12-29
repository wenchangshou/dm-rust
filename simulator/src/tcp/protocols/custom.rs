use async_trait::async_trait;
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info}; // debug, warn unused

use crate::tcp::handler::{HandleResult, ProtocolHandler};
use crate::tcp::state::SimulatorState;

/// 自定义协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProtocolConfig {
    /// 协议名称
    pub name: String,
    /// 协议描述
    pub description: String,
    /// 默认端口
    pub default_port: u16,
    /// 规则列表
    pub rules: Vec<ProtocolRule>,
    /// 校验和设置
    pub checksum: Option<ChecksumConfig>,
}

/// 协议规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRule {
    /// 规则名称
    pub name: String,
    /// 匹配模式
    pub match_pattern: MatchPattern,
    /// 响应动作
    pub action: ResponseAction,
}

/// 匹配模式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MatchPattern {
    /// 正则表达式匹配 (Hex 字符串表示)
    Regex(String),
    /// 十六进制精确匹配
    Hex(String),
    /// 字符串包含匹配
    StringContain(String),
    /// 任意匹配 (作为默认规则)
    Any,
}

/// 响应动作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ResponseAction {
    /// 发送静态数据 (Hex 字符串)
    Static(String),
    /// 发送静态字符串数据
    StaticString(String),
    /// 动态响应 (支持模板变量, Hex 结果)
    Template(String),
    /// 动态响应字符串 (支持模板变量)
    TemplateString(String),
    /// 延迟响应 (毫秒)
    Delay(u64),
    /// 不响应
    None,
}

/// 校验和配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumConfig {
    /// 算法类型
    pub algorithm: ChecksumAlgorithm,
    /// 起始偏移量
    pub range_start: usize,
    /// 结束偏移量 (倒数)
    pub range_end_offset: usize,
    /// 字节序 (true: BigEndian, false: LittleEndian)
    pub big_endian: bool,
}

/// 校验和算法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChecksumAlgorithm {
    Crc16Modbus,
    Sum8,
    XOR,
}

/// 自定义协议处理器
pub struct CustomProtocolHandler {
    config: CustomProtocolConfig,
    compiled_regexes: HashMap<usize, Regex>, // Index -> Regex
}

impl CustomProtocolHandler {
    pub fn new(config: CustomProtocolConfig) -> Self {
        let mut handler = Self {
            config,
            compiled_regexes: HashMap::new(),
        };
        handler.compile_regexes();
        handler
    }

    fn compile_regexes(&mut self) {
        for (i, rule) in self.config.rules.iter().enumerate() {
            if let MatchPattern::Regex(pattern) = &rule.match_pattern {
                match Regex::new(pattern) {
                    Ok(re) => {
                        self.compiled_regexes.insert(i, re);
                    }
                    Err(e) => {
                        error!("Invalid regex pattern '{}': {}", pattern, e);
                    }
                }
            }
        }
    }

    /// 计算校验和
    fn calculate_checksum(&self, data: &[u8]) -> Option<Vec<u8>> {
        if let Some(cfg) = &self.config.checksum {
            let start = cfg.range_start;
            let end = data.len().saturating_sub(cfg.range_end_offset);

            if start >= end {
                return None;
            }

            let slice = &data[start..end];
            let checksum: u16 = match cfg.algorithm {
                ChecksumAlgorithm::Crc16Modbus => crc16::State::<crc16::MODBUS>::calculate(slice),
                ChecksumAlgorithm::Sum8 => {
                    let sum: u16 = slice.iter().map(|&b| b as u16).sum();
                    sum & 0xFF
                }
                ChecksumAlgorithm::XOR => {
                    let mut xor = 0u8;
                    for &b in slice {
                        xor ^= b;
                    }
                    xor as u16
                }
            };

            let mut result = Vec::new();
            if cfg.algorithm == ChecksumAlgorithm::Crc16Modbus {
                if cfg.big_endian {
                    result.push((checksum >> 8) as u8);
                    result.push((checksum & 0xFF) as u8);
                } else {
                    result.push((checksum & 0xFF) as u8);
                    result.push((checksum >> 8) as u8);
                }
            } else {
                result.push((checksum & 0xFF) as u8);
            }
            Some(result)
        } else {
            None
        }
    }
}

#[async_trait]
impl ProtocolHandler for CustomProtocolHandler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn default_port(&self) -> u16 {
        self.config.default_port
    }

    async fn handle(&self, data: &[u8], _state: &mut SimulatorState) -> HandleResult {
        for (i, rule) in self.config.rules.iter().enumerate() {
            let matched = match &rule.match_pattern {
                MatchPattern::Regex(_) => {
                    if let Some(re) = self.compiled_regexes.get(&i) {
                        re.is_match(data)
                    } else {
                        false
                    }
                }
                MatchPattern::Hex(hex_str) => {
                    if let Ok(pattern_bytes) = hex::decode(hex_str) {
                        data.starts_with(&pattern_bytes)
                    } else {
                        false
                    }
                }
                MatchPattern::StringContain(s) => {
                    // String contain check on raw bytes? Assuming ASCII/UTF-8
                    if let Ok(data_str) = std::str::from_utf8(data) {
                        data_str.contains(s)
                    } else {
                        false
                    }
                }
                MatchPattern::Any => true,
            };

            if matched {
                info!("Rule matched: {}", rule.name);
                match &rule.action {
                    ResponseAction::Static(hex_resp) => {
                        if let Ok(mut resp_bytes) = hex::decode(hex_resp) {
                            // Apply checksum if configured
                            if let Some(checksum) = self.calculate_checksum(&resp_bytes) {
                                resp_bytes.extend(checksum);
                            }
                            return HandleResult::Response(resp_bytes);
                        } else {
                            return HandleResult::Error("Invalid static response hex".to_string());
                        }
                    }
                    ResponseAction::StaticString(s) => {
                        let mut resp_bytes = s.as_bytes().to_vec();
                        // Apply checksum if configured
                        if let Some(checksum) = self.calculate_checksum(&resp_bytes) {
                            resp_bytes.extend(checksum);
                        }
                        return HandleResult::Response(resp_bytes);
                    }
                    ResponseAction::Template(template_str) => {
                        let mut resp_str = template_str.clone();
                        // 基础变量替换
                        for (key, value) in &_state.values {
                            let placeholder = format!("{{{{ {} }}}}", key);
                            let val_str = match value {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => continue,
                            };
                            resp_str = resp_str.replace(&placeholder, &val_str);
                        }

                        if let Ok(mut resp_bytes) = hex::decode(&resp_str) {
                            // Apply checksum if configured
                            if let Some(checksum) = self.calculate_checksum(&resp_bytes) {
                                resp_bytes.extend(checksum);
                            }
                            return HandleResult::Response(resp_bytes);
                        } else {
                            return HandleResult::Error(format!(
                                "Template result invalid hex: {}",
                                resp_str
                            ));
                        }
                    }
                    ResponseAction::TemplateString(template_str) => {
                        let mut resp_str = template_str.clone();
                        // 基础变量替换
                        for (key, value) in &_state.values {
                            let placeholder = format!("{{{{ {} }}}}", key);
                            let val_str = match value {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                _ => continue,
                            };
                            resp_str = resp_str.replace(&placeholder, &val_str);
                        }

                        let mut resp_bytes = resp_str.as_bytes().to_vec();
                        // Apply checksum if configured
                        if let Some(checksum) = self.calculate_checksum(&resp_bytes) {
                            resp_bytes.extend(checksum);
                        }
                        return HandleResult::Response(resp_bytes);
                    }
                    ResponseAction::Delay(ms) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(*ms)).await;
                        // Delay is usually part of a sequence, but here it's the action
                        // Maybe we need a sequence of actions?
                        // For simplicty, Delay just waits and returns NoResponse
                        return HandleResult::NoResponse;
                    }
                    ResponseAction::None => {
                        return HandleResult::NoResponse;
                    }
                }
            }
        }

        HandleResult::NoResponse
    }

    fn supported_commands(&self) -> Vec<String> {
        self.config.rules.iter().map(|r| r.name.clone()).collect()
    }
}
