use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use rand::prelude::IndexedRandom as _;

use super::{
    character::{Character, Type},
    condition::{Condition, ExtraCharacters, TypeCond},
    data::{IncludedData, Script, UserData},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub script: String,
    pub selected: BTreeMap<String, Selected>,
    pub player_count: u8,
    pub type_counts_locked: bool,
    pub outsider_count: u8,
    pub minion_count: u8,
    pub demon_count: u8,
    pub included_data: IncludedData,
    pub user_data: UserData,
    pub expanded_script_menu: bool,
    pub script_rename_input: String,
    pub script_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Selected {
    pub locked: bool,
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

    pub fn script_characters(&self) -> Vec<&Character> {
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

    fn characters(&self) -> impl Iterator<Item = &Character> {
        self.included_data
            .characters
            .iter()
            .chain(self.user_data.characters.iter())
    }

    fn get_character(&self, id: &str) -> Option<&Character> {
        self.characters().find(|&c| c.id() == id)
    }

    pub fn scripts(&self) -> impl Iterator<Item = Script> {
        self.included_data
            .scripts
            .iter()
            .chain(self.user_data.scripts.iter())
            .map(|s| {
                let mut it = s.clone();
                // Filter out characters that are not in the data
                // and duplicate characters that have the same id.
                it.characters = it
                    .characters
                    .iter()
                    .flat_map(|id| {
                        self.characters()
                            .filter(|&c| (&c.id_no_numeric() == id))
                            .map(|c| c.id())
                            .collect::<Vec<_>>()
                    })
                    .collect();
                it
            })
    }

    fn get_current_script(&self) -> Option<Script> {
        self.scripts().find(|s| s.name == self.script)
    }

    pub fn import_script(&mut self) {
        let Ok(mut new_script) =
            super::data::import_script(&self.script_input).inspect_err(|e| tracing::error!(?e))
        else {
            gloo_dialogs::alert("Invalid script format");
            return;
        };

        // Ensure the script name is unique
        let base_name = new_script.name.clone();
        let mut i = 0;
        while self.scripts().any(|s| s.name == new_script.name) {
            i += 1;
            new_script.name = format!("{base_name} ({i})");
        }

        self.user_data.scripts.push(new_script);
    }

    pub fn is_valid_character_list(&self) -> bool {
        let (valid, extra) = validate_list(&self.selected_characters(), self.type_counts());
        valid && self.selected.len() == (self.player_count + extra) as usize
    }

    pub fn randomize_unlocked(&mut self) {
        let old_unlocked = self
            .selected
            .extract_if(|_, selected| !selected.locked)
            .map(|(id, _)| id)
            .collect();

        let locked: Vec<_> = self
            .selected
            .iter()
            .filter_map(|(id, _)| self.script_characters().into_iter().find(|c| &c.id() == id))
            .collect();

        match self.get_randomized_characters(&locked, &old_unlocked) {
            Some((i, new_selected)) => {
                tracing::info!("Valid permutation found after {i} iterations");
                self.selected
                    .extend(new_selected.into_iter().map(|id| (id, Default::default())));
            }
            None => {
                let err = format!(
                    "Failed to randomize after {} iterations",
                    crate::consts::MAX_GENERATION_ITERATIONS
                );
                tracing::error!(err);
                gloo_dialogs::alert(&err);
            }
        }
    }

    fn get_randomized_characters(
        &self,
        locked: &[&Character],
        old_unlocked: &BTreeSet<String>,
    ) -> Option<(usize, BTreeSet<String>)> {
        let missing = self.player_count as usize - locked.len();

        let character_pool: Vec<&Character> = self
            .script_characters()
            .into_iter()
            .filter(|c| !locked.iter().any(|locked_c| locked_c.id() == c.id()))
            .collect();

        let mut extras_set = HashSet::from([0]);
        for i in 0..crate::consts::MAX_GENERATION_ITERATIONS {
            for current_extras in extras_set.clone() {
                let new = character_pool
                    .choose_multiple(&mut rand::rng(), missing + current_extras as usize)
                    .copied()
                    .collect::<Vec<_>>();

                let (valid, extras) = validate_list(&[locked, &new].concat(), self.type_counts());
                extras_set.insert(extras);

                if valid && extras == current_extras {
                    let new_unlocked = new.into_iter().map(Character::id).collect::<BTreeSet<_>>();
                    if &new_unlocked != old_unlocked {
                        return Some((i + 1, new_unlocked));
                    }
                }
            }
        }

        None
    }

    fn type_counts(&self) -> HashMap<Type, BTreeSet<i8>> {
        let mut type_counts = HashMap::new();
        type_counts.insert(Type::Outsider, BTreeSet::from([self.outsider_count as i8]));
        type_counts.insert(Type::Minion, BTreeSet::from([self.minion_count as i8]));
        type_counts.insert(Type::Demon, BTreeSet::from([self.demon_count as i8]));
        type_counts
    }
}

fn validate_list(
    characters: &[&Character],
    mut type_counts: HashMap<Type, BTreeSet<i8>>,
) -> (bool, u8) {
    let conditions: Vec<_> = characters
        .iter()
        .filter_map(|c| c.conditions.clone())
        .flatten()
        .collect();

    let mut type_is_any_count: HashMap<Type, bool> = HashMap::new();
    let mut saturating_subs: HashMap<Type, BTreeSet<u8>> = HashMap::new();

    let mut valid = true;
    let mut extras = 0;
    let mut extras_by_type = HashMap::new();
    for condition in conditions {
        match condition {
            Condition::Character { character } => {
                if !characters.iter().any(|c| c.id() == character) {
                    valid = false;
                }
            }
            Condition::Type {
                r#type,
                amount: TypeCond::None,
            } => {
                if characters.iter().any(|c| c.r#type == r#type) {
                    valid = false;
                }
                type_is_any_count.insert(r#type, true);
            }
            Condition::Type {
                r#type,
                amount: TypeCond::Any,
            } => {
                type_is_any_count.insert(r#type, true);
            }
            Condition::Type {
                r#type,
                amount: TypeCond::Add(amounts),
            } => {
                let counts = type_counts.entry(r#type).or_default();
                let mut new_counts = BTreeSet::new();
                for amount in amounts {
                    for &count in counts.iter() {
                        new_counts.insert(amount + count);
                    }
                }
                *counts = new_counts;
            }
            Condition::Type {
                r#type,
                amount: TypeCond::SaturatingSub(amounts),
            } => {
                saturating_subs.entry(r#type).or_default().extend(amounts);
            }
            Condition::ExtraCharacters {
                extra_characters: ExtraCharacters::Const(extra),
            } => extras += extra,
            Condition::ExtraCharacters {
                extra_characters: ExtraCharacters::Type(r#type, sub),
            } => {
                let type_count = characters.iter().filter(|c| c.r#type == r#type).count() as u8;
                let extra = type_count.saturating_sub(sub);
                extras_by_type
                    .entry(r#type)
                    .and_modify(|e: &mut u8| *e = (*e).max(extra))
                    .or_insert(extra);
            }
        }
    }

    for (r#type, amounts) in saturating_subs {
        let counts = type_counts.entry(r#type).or_default();
        let mut new_counts = BTreeSet::new();
        for &amount in amounts.iter() {
            for &count in counts.iter() {
                if count >= 0 {
                    let count = count.cast_unsigned();
                    new_counts.insert(count.saturating_sub(amount).cast_signed());
                }
            }
        }
        *counts = new_counts;
    }

    for (r#type, counts) in type_counts {
        if *type_is_any_count.get(&r#type).unwrap_or(&false) {
            continue;
        }
        let actual_count = characters.iter().filter(|c| c.r#type == r#type).count() as i8;
        if !counts.contains(&actual_count) {
            valid = false;
        }
    }

    (valid, extras + extras_by_type.values().sum::<u8>())
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
    for (_, group) in grouped.iter_mut() {
        group.sort_unstable();
    }
    grouped
}
