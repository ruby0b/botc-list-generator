use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::character::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Condition {
    Character { character: String },
    Type { r#type: Type, amount: TypeCond },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeCond {
    None,
    Any,
    SaturatingSub(BTreeSet<u8>),
    Add(BTreeSet<i8>),
}
