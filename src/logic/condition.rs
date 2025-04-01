use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::character::Type;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Condition {
    /// require a character
    Character { character: String },
    /// modify the amount of characters allowed of a type
    Type { r#type: Type, amount: TypeCond },
    /// add additional characters past the player count
    ExtraCharacters { extra_characters: ExtraCharacters },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeCond {
    None,
    Any,
    SaturatingSub(BTreeSet<u8>),
    Add(BTreeSet<i8>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum ExtraCharacters {
    /// add u8 extra characters
    Const(u8),
    /// add (count of $Type characters + i8) extra characters
    Type(Type, i8),
}
