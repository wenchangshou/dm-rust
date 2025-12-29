/// MQTT 规则引擎
///
/// 支持基于 Topic 匹配的自定义规则处理
use serde::{Deserialize, Serialize};

/// MQTT 规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// Topic 匹配模式（支持 + 和 # 通配符）
    pub topic_pattern: String,
    /// Payload 匹配条件（可选）
    pub payload_match: Option<PayloadMatcher>,
    /// 触发动作
    pub action: MqttRuleAction,
    /// 优先级（数字越小优先级越高）
    pub priority: i32,
}

/// Payload 匹配器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PayloadMatcher {
    /// 精确匹配
    Exact { value: String },
    /// 前缀匹配
    Prefix { value: String },
    /// 包含匹配
    Contains { value: String },
    /// 正则表达式匹配
    Regex { pattern: String },
    /// JSON 字段匹配
    JsonField { path: String, value: String },
    /// 十六进制匹配
    Hex { pattern: String },
}

/// 规则触发动作
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MqttRuleAction {
    /// 发布响应消息
    Respond {
        topic: String,
        payload: String,
        /// 是否将原始 topic 中的通配符部分替换到响应 topic
        use_topic_vars: bool,
    },
    /// 转发到其他 Topic
    Forward { target_topic: String },
    /// 仅记录日志
    Log { message: Option<String> },
    /// 静默（不处理）
    Silence,
    /// 执行脚本/模板
    Transform {
        /// 输出 Topic 模板
        output_topic: String,
        /// 输出 Payload 模板
        output_payload: String,
    },
}

impl MqttRule {
    /// 创建新规则
    pub fn new(name: String, topic_pattern: String, action: MqttRuleAction) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            enabled: true,
            topic_pattern,
            payload_match: None,
            action,
            priority: 0,
        }
    }

    /// 检查 Topic 是否匹配规则
    pub fn matches_topic(&self, topic: &str) -> bool {
        topic_matches(&self.topic_pattern, topic)
    }

    /// 检查 Payload 是否匹配
    pub fn matches_payload(&self, payload: &[u8]) -> bool {
        match &self.payload_match {
            None => true,
            Some(matcher) => match matcher {
                PayloadMatcher::Exact { value } => String::from_utf8_lossy(payload) == *value,
                PayloadMatcher::Prefix { value } => {
                    String::from_utf8_lossy(payload).starts_with(value)
                }
                PayloadMatcher::Contains { value } => {
                    String::from_utf8_lossy(payload).contains(value)
                }
                PayloadMatcher::Regex { pattern } => regex::Regex::new(pattern)
                    .map(|re| re.is_match(&String::from_utf8_lossy(payload)))
                    .unwrap_or(false),
                PayloadMatcher::JsonField { path, value } => {
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(payload) {
                        get_json_field(&json, path)
                            .map(|v| v.to_string().trim_matches('"') == value)
                            .unwrap_or(false)
                    } else {
                        false
                    }
                }
                PayloadMatcher::Hex { pattern } => hex::encode(payload).contains(pattern),
            },
        }
    }

    /// 检查消息是否匹配规则
    pub fn matches(&self, topic: &str, payload: &[u8]) -> bool {
        self.enabled && self.matches_topic(topic) && self.matches_payload(payload)
    }
}

/// MQTT Topic 通配符匹配
///
/// 支持:
/// - `+` 匹配单个层级 (e.g., `sensor/+/temperature`)
/// - `#` 匹配多个层级 (e.g., `sensor/#`)
pub fn topic_matches(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();

    let mut pi = 0;
    let mut ti = 0;

    while pi < pattern_parts.len() && ti < topic_parts.len() {
        match pattern_parts[pi] {
            "#" => return true, // # 匹配剩余所有
            "+" => {
                // + 匹配当前层级，继续下一层
                pi += 1;
                ti += 1;
            }
            part => {
                if part != topic_parts[ti] {
                    return false;
                }
                pi += 1;
                ti += 1;
            }
        }
    }

    // 检查是否完全匹配
    pi == pattern_parts.len() && ti == topic_parts.len()
}

/// 从 JSON 中获取指定路径的字段
fn get_json_field<'a>(json: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

/// 规则集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MqttRuleSet {
    pub rules: Vec<MqttRule>,
}

impl MqttRuleSet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: MqttRule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
    }

    /// 删除规则
    pub fn remove_rule(&mut self, rule_id: &str) -> Option<MqttRule> {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule_id) {
            Some(self.rules.remove(pos))
        } else {
            None
        }
    }

    /// 查找匹配的规则
    pub fn find_matching(&self, topic: &str, payload: &[u8]) -> Vec<&MqttRule> {
        self.rules
            .iter()
            .filter(|r| r.matches(topic, payload))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_exact_match() {
        assert!(topic_matches("sensor/temp", "sensor/temp"));
        assert!(!topic_matches("sensor/temp", "sensor/humidity"));
    }

    #[test]
    fn test_topic_single_wildcard() {
        assert!(topic_matches("sensor/+/temp", "sensor/room1/temp"));
        assert!(topic_matches("sensor/+/temp", "sensor/room2/temp"));
        assert!(!topic_matches("sensor/+/temp", "sensor/room1/humidity"));
    }

    #[test]
    fn test_topic_multi_wildcard() {
        assert!(topic_matches("sensor/#", "sensor/room1/temp"));
        assert!(topic_matches("sensor/#", "sensor/room1/room2/temp"));
        assert!(topic_matches("#", "any/topic/here"));
    }
}
