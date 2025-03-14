use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::character::Character;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    pub characters: Vec<Character>,
    pub scripts: Vec<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Script {
    pub name: String,
    pub characters: BTreeSet<String>,
}

