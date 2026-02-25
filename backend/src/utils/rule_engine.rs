use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub event_type: String, // attendance, activity, etc.
    pub condition: serde_json::Value,
    pub action: serde_json::Value, // score change, notification, etc.
    pub priority: i32,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::NaiveDateTime,
}

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn process_event(&self, event: &Event) -> Vec<serde_json::Value> {
        let mut actions = Vec::new();

        // 按优先级排序规则
        let mut applicable_rules = self
            .rules
            .iter()
            .filter(|r| r.enabled && r.event_type == event.event_type)
            .collect::<Vec<_>>();
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 处理每个规则
        for rule in applicable_rules {
            if self.evaluate_condition(&rule.condition, &event.data) {
                actions.push(rule.action.clone());
            }
        }

        actions
    }

    fn evaluate_condition(
        &self,
        _condition: &serde_json::Value,
        _event_data: &serde_json::Value,
    ) -> bool {
        // 这里实现条件评估逻辑
        // 简化实现，实际应该更复杂
        true
    }
}
