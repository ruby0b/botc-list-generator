use serde::{Deserialize, Serialize};

use super::condition::Condition;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Character {
    pub name: String,
    pub description: String,
    pub r#type: Type,
    pub icon: Option<String>,
    pub conditions: Option<Vec<Condition>>,
}

impl Character {
    pub fn id(&self) -> String {
        self.name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_lowercase())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    Townsfolk,
    Outsider,
    Minion,
    Demon,
    Fabled,
    Traveller,
}

impl Type {
    pub fn plural_str(&self) -> &str {
        match self {
            Type::Townsfolk => "Townsfolk",
            Type::Outsider => "Outsiders",
            Type::Minion => "Minions",
            Type::Demon => "Demons",
            Type::Fabled => "Fabled",
            Type::Traveller => "Travellers",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Type::Townsfolk => crate::consts::TOWNSFOLK_ICON,
            Type::Outsider => crate::consts::OUTSIDER_ICON,
            Type::Minion => crate::consts::MINION_ICON,
            Type::Demon => crate::consts::DEMON_ICON,
            Type::Fabled => crate::consts::FABLED_ICON,
            Type::Traveller => crate::consts::TRAVELLER_ICON,
        }
    }
}
