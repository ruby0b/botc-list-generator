use std::collections::{BTreeMap, HashMap};

use rand::prelude::IndexedRandom as _;
use serde::{Deserialize, Serialize};

use super::{
    character::{Character, Type},
    condition::{Condition, TypeCond},
    data::{Data, Script},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub script: String,
    pub selected: BTreeMap<String, bool>,
    pub player_count: u8,
    pub type_counts_locked: bool,
    pub outsider_count: u8,
    pub minion_count: u8,
    pub demon_count: u8,
    pub data: Data,
}

impl State {
    pub fn townsfolk_count(&self) -> u8 {
        self.player_count
            .saturating_sub(self.outsider_count)
            .saturating_sub(self.minion_count)
            .saturating_sub(self.demon_count)
    }

    pub fn update_type_counts(&mut self) {
        if !self.type_counts_locked || self.player_count < 5 {
            return;
        }

        self.demon_count = 1;

        if self.player_count == 5 {
            self.outsider_count = 0;
            self.minion_count = 1;
        } else if self.player_count == 6 {
            self.outsider_count = 1;
            self.minion_count = 1;
        } else {
            self.outsider_count = (self.player_count - 4) % 3;
            self.minion_count = (self.player_count - 4) / 3;
        }
    }

    pub fn selected_characters(&self) -> Vec<&Character> {
        self.selected
            .keys()
            .filter_map(|id| self.get_character(id))
            .collect()
    }

    pub fn characters(&self) -> Vec<&Character> {
        let Some(script) = self.get_current_script() else {
            tracing::error!("Script not found: {}", self.script);
            return Vec::new();
        };
        script
            .characters
            .iter()
            .filter_map(|id| self.get_character(id))
            .collect()
    }

    fn get_character(&self, id: &str) -> Option<&Character> {
        self.data.characters.iter().find(|&r| r.id() == id)
    }

    fn get_current_script(&self) -> Option<&Script> {
        self.data.scripts.iter().find(|&r| r.name == self.script)
    }

    pub fn randomize_unlocked(&mut self) {
        self.selected.retain(|_, locked| *locked);

        let missing = self.player_count as usize - self.selected.len();

        let locked = {
            let characters = self.characters();
            self.selected
                .iter()
                .filter_map(|(id, _)| characters.iter().find(|c| &c.id() == id).cloned())
                .cloned()
                .collect::<Vec<_>>()
        };
        let characters: Vec<Character> = {
            let characters = self.characters();
            characters
                .into_iter()
                .filter(|c| !self.selected.contains_key(&c.id()))
                .cloned()
                .collect()
        };

        for i in 0..crate::consts::MAX_GENERATION_ITERATIONS {
            let mut new_characters = locked.clone();
            new_characters.extend(
                characters
                    .choose_multiple(&mut rand::rng(), missing)
                    .cloned(),
            );

            let mut type_counts = HashMap::new();
            type_counts.insert(Type::Outsider, vec![self.outsider_count as i8]);
            type_counts.insert(Type::Minion, vec![self.minion_count as i8]);
            type_counts.insert(Type::Demon, vec![self.demon_count as i8]);

            if validate_character_list(&new_characters, type_counts) {
                let old_selected = std::mem::take(&mut self.selected);
                self.selected
                    .extend(new_characters.into_iter().map(|r| r.id()).map(|id| {
                        let value = old_selected.get(&id).copied().unwrap_or_default();
                        (id, value)
                    }));

                tracing::info!("Randomized successfully after {} iterations", i + 1);
                return;
            }
        }

        let err = format!(
            "Failed to randomize after {} iterations",
            crate::consts::MAX_GENERATION_ITERATIONS
        );
        tracing::error!(err);
        gloo_dialogs::alert(&err);
    }
}

pub fn validate_character_list(
    characters: &[Character],
    mut type_counts: HashMap<Type, Vec<i8>>,
) -> bool {
    let conditions: Vec<_> = characters
        .iter()
        .filter_map(|c| c.conditions.clone())
        .flatten()
        .collect();

    let mut type_is_any_count: HashMap<Type, bool> = HashMap::new();

    for condition in conditions {
        match condition {
            Condition::Character { character } => {
                if !characters.iter().any(|c| c.id() == character) {
                    return false;
                }
            }
            Condition::Type {
                r#type,
                amount: TypeCond::None,
            } => {
                if characters.iter().any(|c| c.r#type == r#type) {
                    return false;
                }
                type_is_any_count.insert(r#type, true);
            }
            Condition::Type {
                r#type,
                amount: TypeCond::Add(amounts),
            } => {
                let counts = type_counts.entry(r#type).or_default();
                let mut new_counts = Vec::new();
                for amount in amounts {
                    for &count in counts.iter() {
                        new_counts.push(amount + count);
                    }
                }
                new_counts.dedup();
                *counts = new_counts;
            }
            Condition::Type {
                r#type,
                amount: TypeCond::Any,
            } => {
                type_is_any_count.insert(r#type, true);
            }
        }
    }

    for (r#type, counts) in type_counts {
        if *type_is_any_count.get(&r#type).unwrap_or(&false) {
            continue;
        }
        let actual_count = characters.iter().filter(|c| c.r#type == r#type).count() as i8;
        if !counts.contains(&actual_count) {
            return false;
        }
    }

    true
}

pub fn group_characters_by_type<'a>(
    characters: &Vec<&'a Character>,
) -> BTreeMap<Type, Vec<&'a Character>> {
    let mut grouped = BTreeMap::new();
    for &character in characters {
        grouped
            .entry(character.r#type)
            .or_insert_with(Vec::new)
            .push(character);
    }
    grouped
}
