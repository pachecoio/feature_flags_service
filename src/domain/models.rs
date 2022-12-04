use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashSet};
use std::hash::Hash;
use chrono::{DateTime, Utc, serde::ts_seconds::{
    serialize as to_ts,
    deserialize as from_ts,
}};
use mongodb::bson::serde_helpers;
use crate::utils;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FeatureFlag {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub label: String,
    pub enabled: bool,
    pub rules: Vec<Rule>,

    #[serde(with = "utils::date_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "utils::date_format")]
    pub updated_at: DateTime<Utc>
}

impl FeatureFlag {
    pub fn new(name: &str, label: &str, enabled: bool, rules: Vec<Rule>) -> Self {
        FeatureFlag {
            id: None,
            name: name.to_string(),
            label: label.to_string(),
            enabled,
            rules,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_context_valid(&self, context: &Map<String, Value>) -> bool {
        let mut valid = true;
        for rule in self.rules.iter() {
            if !rule.check(context) {
                valid = false;
                break;
            }
        }
        valid
    }
}

#[derive(Serialize, Deserialize)]
pub struct Environment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub flags: HashSet<FeatureFlag>,
}

impl Environment {
    pub fn new(name: &str) -> Self {
        Environment {
            id: None,
            name: name.to_string(),
            flags: HashSet::new(),
        }
    }

    pub fn add_flag(&mut self, flag: &FeatureFlag) {
        self.flags = self.flags.clone().into_iter().filter(|f| f.name != flag.name).collect();
        self.flags.insert(flag.clone());
    }

    pub fn remove_flag(&mut self, flag: &FeatureFlag) {
        self.flags = self
            .flags
            .clone()
            .into_iter()
            .filter(|f| f.name != flag.name)
            .collect();
    }

    pub fn remove_flag_by_name(&mut self, flag_name: &str) {
        self.flags = self
            .flags
            .clone()
            .into_iter()
            .filter(|f| f.name != flag_name)
            .collect();
    }

    pub fn set_flags(&mut self, flags: HashSet<FeatureFlag>) {
        self.flags = flags;
    }

    pub fn get_flags_from_context(&self, context: &Map<String, Value>) -> Map<String, Value> {
        let mut flags = Map::new();
        for flag in self.flags.iter() {
            flags.insert(flag.name.clone(), Value::Bool(flag.is_context_valid(context)));
        }
        flags
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Rule {
    pub parameter: String,
    pub operator: Operator,
}

impl Rule {
    pub fn check(&self, input: &Map<String, Value>) -> bool {
        match input.get(&self.parameter) {
            None => self.validate(&Value::Null),
            Some(value) => self.validate(value),
        }
    }

    fn validate(&self, value: &Value) -> bool {
        match value {
            Value::Null => self.validate_string(None),
            Value::String(v) => self.validate_string(Some(v)),
            Value::Array(values) => {
                for v in values {
                    if !self.validate(v) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn validate_string(&self, value: Option<&str>) -> bool {
        match &self.operator {
            Operator::Is(v) => value == Some(v),
            Operator::IsNot(v) => value.is_none() || value != Some(v),
            Operator::Contains(v) => value.unwrap_or("").contains(v),
            Operator::IsOneOf(v) => value.is_some() && v.contains(&value.unwrap().to_string()),
            Operator::IsNotOneOf(v) => value.is_none() || !v.contains(&value.unwrap().to_string()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Operator {
    Is(String),
    IsNot(String),
    Contains(String),
    IsOneOf(Vec<String>),
    IsNotOneOf(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_instance() {
        let flag = FeatureFlag::new(
            "sample_flag",
            "Sample Flag",
            false,
            vec![]
        );
        assert_eq!(flag.name, "sample_flag");
    }

    #[test]
    fn test_environment_instance() {
        let env = Environment::new("development");
        assert_eq!(env.name, "development");
        assert_eq!(env.flags.len(), 0);
    }

    #[test]
    fn test_environment_add_flag() {
        let mut env = Environment::new("development");
        let flag = FeatureFlag::new(
            "sample_flag",
            "Sample Flag",
            false,
            vec![]
        );
        env.add_flag(&flag);
        env.add_flag(&flag); // Should not add repeated flag
        assert_eq!(env.flags.len(), 1);

        env.remove_flag(&flag);
        assert_eq!(env.flags.len(), 0);
    }

}

#[cfg(test)]
mod test_rules {
    use super::*;

    #[test]
    fn test_rule_instance() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::Is("tenant_1".to_string()),
        };
        assert_eq!(rule.parameter, "tenant");
        assert_eq!(rule.operator, Operator::Is("tenant_1".to_string()))
    }

    #[test]
    fn test_rule_is() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::Is("tenant_1".to_string()),
        };
        let mut payload = Map::new();
        payload.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        assert!(rule.check(&payload));
        payload.clear();
        assert!(!rule.check(&payload));
        payload.insert("tenant".to_string(), Value::String("tenant_2".to_string()));
        assert!(!rule.check(&payload));
    }

    #[test]
    fn test_rule_is_not() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::IsNot("tenant_1".to_string()),
        };
        let mut payload = Map::new();
        payload.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        assert!(!rule.check(&payload));
        payload.clear();
        assert!(rule.check(&payload));
        payload.insert("tenant".to_string(), Value::String("tenant_2".to_string()));
        assert!(rule.check(&payload));
    }

    #[test]
    fn test_rule_contains() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::Contains("tenant".to_string()),
        };
        let mut payload = Map::new();
        payload.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        assert!(rule.check(&payload));
        payload.clear();
        assert!(!rule.check(&payload));
        payload.insert("tenant".to_string(), Value::String("test".to_string()));
        assert!(!rule.check(&payload));
    }

    #[test]
    fn test_rule_is_one_of() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::IsOneOf(Vec::from(["tenant_1".to_string()])),
        };
        let mut payload = Map::new();
        payload.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        assert!(rule.check(&payload));
        payload.clear();
        assert!(!rule.check(&payload));
        payload.insert("tenant".to_string(), Value::String("test".to_string()));
        assert!(!rule.check(&payload));
    }

    #[test]
    fn test_rule_is_not_one_of() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::IsNotOneOf(Vec::from(["tenant_1".to_string()])),
        };
        let mut payload = Map::new();
        payload.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        assert!(!rule.check(&payload));
        payload.clear();
        assert!(rule.check(&payload));
        payload.insert("tenant".to_string(), Value::String("test".to_string()));
        assert!(rule.check(&payload));
    }

    #[test]
    fn test_rule_with_input_array() {
        let rule = Rule {
            parameter: "tenant".to_string(),
            operator: Operator::Is("tenant_1".to_string()),
        };
        let mut payload = Map::new();
        payload.insert(
            "tenant".to_string(),
            Value::Array(Vec::from([Value::String("tenant_1".to_string())])),
        );
        assert!(rule.check(&payload));
    }
}

#[cfg(test)]
mod test_environment {
    use super::*;

    #[test]
    fn test_environment_get_flags_from_context() {
        let mut env = Environment::new("development");
        let flag_1 = FeatureFlag::new(
            "flag_1",
            "Flag 1",
            true,
            vec![
                Rule {
                    parameter: "tenant".to_string(),
                    operator: Operator::Is("tenant_1".to_string()),
                },
                Rule {
                    parameter: "user".to_string(),
                    operator: Operator::IsOneOf(vec![
                        "user_1".to_string(),
                        "user_2".to_string(),
                    ]),
                },
            ]
        );
        let flag_2 = FeatureFlag::new(
            "flag_2",
            "Flag 2",
            true,
            vec![
                Rule {
                    parameter: "custom_prop".to_string(),
                    operator: Operator::IsNot("custom".to_string()),
                },
            ]
        );
        env.add_flag(&flag_1);
        env.add_flag(&flag_2);

        let mut context = Map::new();
        context.insert("tenant".to_string(), Value::String("tenant_1".to_string()));
        context.insert("user".to_string(), Value::String("user_1".to_string()));
        let flags = env.get_flags_from_context(&context);
        assert_eq!(flags.len(), 2);
    }
}
