use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::character::Character;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncludedData {
    pub characters: Vec<Character>,
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UserData {
    pub characters: BTreeSet<Character>,
    pub scripts: BTreeSet<Script>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Script {
    pub name: String,
    pub characters: BTreeSet<String>,
}

pub fn import_script(json: &str) -> Result<Script, serde_json::Error> {
    let vec: Vec<serde_json::Value> = serde_json::from_str(json)?;
    let name = vec
        .iter()
        .filter_map(|v| v.as_object())
        .next()
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("My Script")
        .to_string();
    let characters: BTreeSet<_> = vec
        .iter()
        .filter_map(|v| v.as_str())
        .map(str::to_string)
        .collect();
    Ok(Script { name, characters })
}
