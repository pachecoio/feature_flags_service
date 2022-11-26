use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FeatureFlag {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub label: String,
    pub enabled: bool,
    pub rules: Vec<Rule>,
}

impl FeatureFlag {
    pub fn new(name: &str, label: &str) -> Self {
        FeatureFlag {
            id: None,
            name: name.to_string(),
            label: label.to_string(),
            enabled: false,
            rules: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Environment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
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
                return true;
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
        let flag = FeatureFlag::new("sample_flag", "Sample Flag");
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
        let flag = FeatureFlag::new("sample_flag", "Sample Flag");
        env.add_flag(&flag);
        env.add_flag(&flag); // Should not add repeated flag
        assert_eq!(env.flags.len(), 1);

        env.remove_flag(&flag);
        assert_eq!(env.flags.len(), 0);
    }

    #[test]
    fn test_environment_get_flags_from_context() {
        let mut env = Environment::new("development");
        let flag = FeatureFlag::new("sample_flag", "Sample Flag");
        env.add_flag(&flag);
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
    fn test_environment_get_flags() {}
}
