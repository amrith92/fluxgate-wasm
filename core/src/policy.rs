use crate::config::CheckRequest;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyMatcher {
    clauses: Vec<MatchClause>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MatchClause {
    kind: MatchKind,
    pattern: MatchPattern,
    key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum MatchKind {
    Ip,
    Route,
    Header,
    Attr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
enum MatchPattern {
    Any,
    Equals(String),
    Prefix(String),
    Exists,
}

impl PolicyMatcher {
    pub fn from_rule(rule: &str) -> Result<Self, String> {
        let mut clauses = Vec::new();
        for token in rule.split_whitespace().filter(|token| !token.is_empty()) {
            if let Some(rest) = token.strip_prefix("ip:") {
                clauses.push(MatchClause {
                    kind: MatchKind::Ip,
                    pattern: MatchPattern::parse(rest)?,
                    key: "ip".to_string(),
                });
            } else if let Some(rest) = token.strip_prefix("route:") {
                clauses.push(MatchClause {
                    kind: MatchKind::Route,
                    pattern: MatchPattern::parse(rest)?,
                    key: "route".to_string(),
                });
            } else if let Some(rest) = token.strip_prefix("header:") {
                let (name, pattern) = parse_header_clause(rest)?;
                clauses.push(MatchClause {
                    kind: MatchKind::Header,
                    pattern,
                    key: name,
                });
            } else if let Some(rest) = token.strip_prefix("attr:") {
                let (name, pattern) = parse_attr_clause(rest)?;
                clauses.push(MatchClause {
                    kind: MatchKind::Attr,
                    pattern,
                    key: name,
                });
            } else {
                return Err(format!("unsupported matcher token: {token}"));
            }
        }

        if clauses.is_empty() {
            return Err("policy match rule must contain at least one predicate".to_string());
        }

        Ok(Self { clauses })
    }

    pub fn matches(&self, request: &CheckRequest) -> Option<IndexMap<String, String>> {
        let mut captured = IndexMap::new();
        for clause in &self.clauses {
            let source_value = match clause.kind {
                MatchKind::Ip => request.ip.clone(),
                MatchKind::Route => request.route.clone(),
                MatchKind::Header => request
                    .headers
                    .as_ref()
                    .and_then(|headers| headers.get(&clause.key))
                    .cloned()
                    .flatten(),
                MatchKind::Attr => request
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.get(&clause.key))
                    .map(value_to_string),
            };

            let capture = match_value(&clause.pattern, source_value)?;
            captured.insert(clause.key.clone(), capture);
        }

        Some(captured)
    }
}

impl MatchPattern {
    fn parse(input: &str) -> Result<Self, String> {
        if input.is_empty() || input == "*" {
            return Ok(MatchPattern::Any);
        }

        if input == "?" {
            return Ok(MatchPattern::Exists);
        }

        if let Some(prefix) = input.strip_suffix('*') {
            if prefix.is_empty() {
                return Ok(MatchPattern::Any);
            }
            return Ok(MatchPattern::Prefix(prefix.to_string()));
        }

        Ok(MatchPattern::Equals(input.to_string()))
    }
}

fn parse_header_clause(input: &str) -> Result<(String, MatchPattern), String> {
    let mut parts = input.splitn(2, '=');
    let name = parts
        .next()
        .ok_or_else(|| "header clause missing name".to_string())?
        .to_string();
    let name = name.trim().to_string();
    let value = parts.next().unwrap_or("*");
    let pattern = MatchPattern::parse(value.trim())?;
    Ok((name, pattern))
}

fn parse_attr_clause(input: &str) -> Result<(String, MatchPattern), String> {
    let mut parts = input.splitn(2, '=');
    let name = parts
        .next()
        .ok_or_else(|| "attr clause missing name".to_string())?
        .to_string();
    let value = parts.next().unwrap_or("*");
    let pattern = MatchPattern::parse(value.trim())?;
    Ok((name, pattern))
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
        serde_json::Value::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
    }
}

fn match_value(pattern: &MatchPattern, value: Option<String>) -> Option<String> {
    match pattern {
        MatchPattern::Any => value,
        MatchPattern::Exists => value,
        MatchPattern::Equals(expected) => value.filter(|val| val == expected),
        MatchPattern::Prefix(prefix) => value.filter(|val| val.starts_with(prefix)),
    }
}
