use std::collections::BTreeMap;

use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{html::Scope, prelude::*};

use crate::logic::{
    character::{Character, Type},
    data::Data,
    state::{State, group_characters_by_type},
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
}

pub struct App {
    state: State,
    _focus_ref: NodeRef,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let data = serde_json::from_str::<Data>(include_str!("data.json")).unwrap();
        let state = State {
            script: "Bad Moon Rising".to_string(),
            selected: BTreeMap::new(),
            data,
            player_count: 10,
            type_counts_locked: true,
            outsider_count: 0,
            minion_count: 2,
            demon_count: 1,
        };
        let focus_ref = NodeRef::default();
        Self {
            state,
            _focus_ref: focus_ref,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Toggle(character) => {
                if self.state.selected.contains_key(&character) {
                    self.state.selected.remove(&character);
                } else if self.state.selected.len() < self.state.player_count as usize {
                    self.state.selected.insert(character, true);
                } else {
                    return false;
                }
                true
            }
            Msg::ToggleLock(character) => {
                if let Some(locked) = self.state.selected.get_mut(&character) {
                    *locked = !*locked;
                }
                true
            }
            Msg::SetLockForAll(value) => {
                for (_, locked) in self.state.selected.iter_mut() {
                    *locked = value;
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
        }
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
                        </div>
                    </div>
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
        let by_type = group_characters_by_type(&self.state.characters());
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
        let locked = self
            .state
            .selected
            .get(&char.id())
            .copied()
            .unwrap_or(false);
        html! {
            <li class={classes!("clickable", if locked {"locked"} else {"unlocked"})} onclick={toggle_lock}>
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
        for script in &self.state.data.scripts {
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
