use serde::{Deserialize, Serialize};

use super::character::Character;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncludedData {
    pub characters: Vec<Character>,
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserData {
    pub characters: Vec<Character>,
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Script {
    pub name: String,
    pub characters: Vec<String>,
}

pub fn import_script(json: &str) -> Result<Script, serde_json::Error> {
    let vec: Vec<serde_json::Value> = serde_json::from_str(json)?;

    let objects = vec.iter().filter_map(|v| v.as_object()).collect::<Vec<_>>();

    let name = objects
        .iter()
        .find(|v| v.get("id").is_some_and(|id| id.as_str() == Some("_meta")))
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .and_then(|s| (!s.is_empty()).then_some(s))
        .unwrap_or("My Script")
        .to_string();

    let characters = objects
        .iter()
        .filter_map(|v| v.get("id"))
        .filter_map(|id| id.as_str())
        .filter(|id| !id.is_empty())
        .filter(|id| !id.starts_with('_'))
        .chain(vec.iter().filter_map(|v| v.as_str()))
        .map(|id| id.replace(['-', '_'], ""))
        .collect();

    Ok(Script { name, characters })
}
