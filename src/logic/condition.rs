use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::character::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Condition {
    Character { character: String },
    Type { r#type: Type, amount: TypeCond },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeCond {
    None,
    Any,
    SaturatingSub(BTreeSet<u8>),
    Add(BTreeSet<i8>),
    // TODO: doesn't have anything to do with a type
    IncreasePlayerCount(u8),
}
