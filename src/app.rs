use std::collections::BTreeMap;

use gloo_storage::{LocalStorage, Storage as _};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{html::Scope, prelude::*};

use crate::logic::{
    character::{Character, Type},
    state::{Selected, State, group_characters_by_type},
};

pub enum Msg {
    Toggle(String),
    ToggleLock(String),
    SetLockForAll(bool),
    Randomize,
    ClearAll,
    SetPlayerCount(u8),
    SetTypeCountsLocked(bool),
    SetOutsiderCount(u8),
    SetMinionCount(u8),
    SetDemonCount(u8),
    SetScript(String),
    ToggleScriptMenu,
    DeleteScript,
    UpdateScriptInput(String),
    ImportScript,
}

pub struct App {
    state: State,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let included_data = serde_json::from_str(include_str!("data.json")).unwrap();
        let user_data = LocalStorage::get(crate::consts::STORAGE_KEY).unwrap_or_default();
        let mut state = State {
            script: crate::consts::DEFAULT_SCRIPT.to_string(),
            selected: BTreeMap::new(),
            player_count: 10,
            type_counts_locked: true,
            outsider_count: 0,
            minion_count: 2,
            demon_count: 1,
            included_data,
            user_data,
            expanded_script_menu: false,
            script_input: String::new(),
        };
        state.randomize_unlocked();
        Self { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let redraw = match msg {
            Msg::Toggle(character) => {
                if self.state.selected.contains_key(&character) {
                    self.state.selected.remove(&character);
                } else {
                    self.state
                        .selected
                        .insert(character, Selected { locked: true });
                }
                true
            }
            Msg::ToggleLock(character) => {
                if let Some(selected) = self.state.selected.get_mut(&character) {
                    selected.locked = !selected.locked;
                }
                true
            }
            Msg::SetLockForAll(value) => {
                for (_, selected) in self.state.selected.iter_mut() {
                    selected.locked = value;
                }
                true
            }
            Msg::Randomize => {
                self.state.randomize_unlocked();
                true
            }
            Msg::SetPlayerCount(count) => {
                self.state.player_count = count;
                self.state.update_type_counts();
                true
            }
            Msg::SetTypeCountsLocked(value) => {
                self.state.type_counts_locked = value;
                self.state.update_type_counts();
                true
            }
            Msg::SetOutsiderCount(count) => {
                self.state.outsider_count = count;
                true
            }
            Msg::SetMinionCount(count) => {
                self.state.minion_count = count;
                true
            }
            Msg::SetDemonCount(count) => {
                self.state.demon_count = count;
                true
            }
            Msg::SetScript(script) => {
                self.state.script = script;
                self.state.selected.clear();
                true
            }
            Msg::ClearAll => {
                self.state.selected.clear();
                true
            }
            Msg::ToggleScriptMenu => {
                self.state.expanded_script_menu = !self.state.expanded_script_menu;
                true
            }
            Msg::DeleteScript => {
                self.state
                    .user_data
                    .scripts
                    .retain(|s| s.name != self.state.script);
                self.state.script = crate::consts::DEFAULT_SCRIPT.to_string();
                self.state.expanded_script_menu = false;
                true
            }
            Msg::UpdateScriptInput(input) => {
                self.state.script_input = input;
                false
            }
            Msg::ImportScript => {
                self.state.import_script();
                self.state.script_input.clear();
                self.state.expanded_script_menu = false;
                true
            }
        };
        LocalStorage::set(crate::consts::STORAGE_KEY, &self.state.user_data).unwrap();
        redraw
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let set_type_counts_locked = ctx.link().callback(|e: Event| {
            Msg::SetTypeCountsLocked(e.target_unchecked_into::<HtmlInputElement>().checked())
        });

        html! {
            <main>
                <div class="sidebar">
                    <div class="box">
                        <div class="row">
                            {self.view_script_dropdown(ctx.link())}
                            <button onclick={ctx.link().callback(|_| Msg::ToggleScriptMenu)}>{"⚙️"}</button>
                        </div>
                    </div>
                    {if self.state.expanded_script_menu {self.view_script_menu(ctx.link())} else {html! {}}}
                    <div class="box">
                        <div class="row">
                            <label>{"Player Count: "}</label>
                            <input type="number" min="1" max="50"
                                value={self.state.player_count.to_string()}
                                onchange={clamped(ctx, 5, 15, Msg::SetPlayerCount)}
                            />
                        </div>
                        <div class="row">
                            <label>{"Default Type Counts: "}</label>
                            <input type="checkbox"
                                checked={self.state.type_counts_locked}
                                onchange={set_type_counts_locked}
                            />
                        </div>
                        <div class="row">
                            <label>{format!("{}: ", Type::Townsfolk.plural_str())}</label>
                            <input type="number" min="0" max="50"
                                disabled=true
                                value={self.state.townsfolk_count().to_string()}
                            />
                        </div>
                        <div class="row">
                            <label>{format!("{}: ", Type::Outsider.plural_str())}</label>
                            <input type="number" min="0" max="50"
                                disabled={self.state.type_counts_locked}
                                value={self.state.outsider_count.to_string()}
                                onchange={clamped(ctx, 0, 15, Msg::SetOutsiderCount)}
                            />
                        </div>
                        <div class="row">
                            <label>{format!("{}: ", Type::Minion.plural_str())}</label>
                            <input type="number" min="0" max="50"
                                disabled={self.state.type_counts_locked}
                                value={self.state.minion_count.to_string()}
                                onchange={clamped(ctx, 0, 15, Msg::SetMinionCount)}
                            />
                        </div>
                        <div class="row">
                            <label>{format!("{}: ", Type::Demon.plural_str())}</label>
                            <input type="number" min="0" max="50"
                                disabled={self.state.type_counts_locked}
                                value={self.state.demon_count.to_string()}
                                onchange={clamped(ctx, 0, 15, Msg::SetDemonCount)}
                            />
                        </div>
                    </div>
                    <div class="box">
                        <div class="row">
                            {"Valid List: "}{if self.state.is_valid_character_list() {"✅"} else {"❌"}}
                        </div>
                    </div>
                    <div class="box">
                        <div class="row">
                            <button onclick={ctx.link().callback(|_| Msg::Randomize)}>{"Randomize Unlocked"}</button>
                        </div>
                        <div class="row">
                            <button onclick={ctx.link().callback(|_| Msg::SetLockForAll(true))}>{"Lock All"}</button>
                            <button onclick={ctx.link().callback(|_| Msg::SetLockForAll(false))}>{"Unlock All"}</button>
                            <button onclick={ctx.link().callback(|_| Msg::ClearAll)}>{"Clear All"}</button>
                        </div>
                    </div>
                    {self.view_character_list(ctx.link())}
                </div>
                <div class="content">
                    <div class="box">
                        {self.view_selected_characters(ctx.link())}
                    </div>
                </div>
            </main>
        }
    }
}

impl App {
    fn view_character_list(&self, link: &Scope<Self>) -> Html {
        let by_type = group_characters_by_type(&self.state.script_characters());
        let mut li = Vec::new();
        for (r#type, cs) in by_type {
            li.push(self.view_type(&r#type));
            li.extend(cs.iter().map(|c| self.view_character_item(link, c)));
        }
        html! { <ul> { for li } </ul> }
    }

    fn view_character_item(&self, link: &Scope<Self>, char: &Character) -> Html {
        let onclick = {
            let id = char.id();
            link.callback(move |_| Msg::Toggle(id.clone()))
        };
        let selected = self
            .state
            .selected
            .contains_key(&char.id())
            .then_some("selected");
        html! {
            <li class="clickable">
                <div class={classes!(selected)} onclick={onclick}>
                    <img src={char.icon.clone()} width="32.5" height="32.5"/>
                    {&char.name}
                </div>
            </li>
        }
    }

    fn view_selected_characters(&self, link: &Scope<Self>) -> Html {
        let by_type = group_characters_by_type(&self.state.selected_characters());
        let mut lists = Vec::new();
        for (r#type, cs) in by_type {
            let li = cs.iter().map(|c| self.view_selected_character(link, c));
            lists.push(html! { <> {self.view_type(&r#type)} <ul> { for li } </ul> </> });
        }
        html! { { for lists } }
    }

    fn view_selected_character(&self, link: &Scope<Self>, char: &Character) -> Html {
        let toggle_lock = {
            let id = char.id();
            link.callback(move |_| Msg::ToggleLock(id.clone()))
        };
        let selected = self
            .state
            .selected
            .get(&char.id())
            .cloned()
            .unwrap_or(Default::default());
        html! {
            <li class={classes!("clickable", if selected.locked {"locked"} else {"unlocked"})} onclick={toggle_lock}>
                <img src={char.icon.clone()}/>
                <div>
                    <h4>{&char.name}</h4>
                    <p>{&char.description}</p>
                </div>
            </li>
        }
    }

    fn view_type(&self, r#type: &Type) -> Html {
        html! {
            <h3>
                {r#type.plural_str()}
                <img src={r#type.icon().to_string()} width="52" height="52"/>
            </h3>
        }
    }

    fn view_script_dropdown(&self, link: &Scope<Self>) -> Html {
        let mut options = Vec::new();
        for script in self.state.scripts() {
            options.push(html! {
                <option
                    selected={script.name == self.state.script}
                    value={script.name.clone()}
                >
                    {&script.name}
                </option>
            });
        }
        let set_script =
            link.callback(move |e: Event| Msg::SetScript(get_text(e.target().unwrap())));
        html! {
            <>
            <label for="script">{"Script: "}</label>
            <select name="script" id="script" onchange={set_script}>{ for options }</select>
            </>
        }
    }

    fn view_script_menu(&self, link: &Scope<Self>) -> Html {
        let update_script_input =
            link.callback(|e: InputEvent| Msg::UpdateScriptInput(get_text(e.target().unwrap())));
        let is_user_script = self
            .state
            .user_data
            .scripts
            .iter()
            .any(|s| s.name == self.state.script);
        html! {
            <div class="box">
                <div class="row">
                    <div>
                        {if is_user_script {
                            html!{<button onclick={link.callback(|_| Msg::DeleteScript)}>{"Delete Current Script"}</button>}
                        } else {
                            html!{}
                        }}
                        <p>
                            {"Use the "}
                            <a href="https://script.bloodontheclocktower.com/" target="_blank">{"BotC Script Builder"}</a>
                            {" and export to clipboard (JSON):"}
                        </p>
                        <input type="text"
                            placeholder="Paste your script here..."
                            oninput={update_script_input}
                            value={self.state.script_input.clone()}
                        />
                        <button onclick={link.callback(|_| Msg::ImportScript)}>{"Import"}</button>
                    </div>
                </div>
            </div>
        }
    }
}

fn get_text(target: EventTarget) -> String {
    target
        .value_of()
        .unchecked_into::<HtmlInputElement>()
        .value()
}

fn clamped<T>(ctx: &Context<App>, min: T, max: T, msg: fn(T) -> Msg) -> Callback<Event>
where
    T: Copy + Ord + std::str::FromStr + 'static,
{
    ctx.link().callback(move |e: Event| {
        let count = get_text(e.target().unwrap()).parse::<T>().unwrap_or(min);
        if count < min {
            msg(min)
        } else if count > max {
            msg(max)
        } else {
            msg(count)
        }
    })
}
