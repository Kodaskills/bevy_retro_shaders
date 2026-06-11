#![cfg(target_arch = "wasm32")]
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Serialize;
use serde_json::Value;
use web_sys::{window, CustomEvent, CustomEventInit};

use crate::egui_events::UiInteraction;
use crate::CrtState;

// ── Resources ─────────────────────────────────────────────────────────────────

pub struct AnalyticsPlugin;

impl Plugin for AnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<crate::egui_events::UiInteraction>()
            .insert_resource(EventState::default())
            .add_systems(Update, process_analytics);
    }
}

#[derive(Resource)]
pub struct EventState {
    pub previous: CrtState,
}

impl Default for EventState {
    fn default() -> Self {
        Self {
            previous: CrtState::default(),
        }
    }
}

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum InteractionType {
    #[allow(dead_code)]
    Click,
    KeyPress(KeyCode),
    Named(String),
}

impl InteractionType {
    pub fn as_str(&self) -> String {
        match self {
            Self::Click => "click".to_string(),
            Self::KeyPress(k) => format!("keypress:{:?}", k),
            Self::Named(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateDiff(serde_json::Map<String, Value>);

impl StateDiff {
    pub fn try_from(old: &CrtState, new: &CrtState) -> Option<Self> {
        let old_v = serde_json::to_value(old).ok()?;
        let new_v = serde_json::to_value(new).ok()?;

        let diff: serde_json::Map<String, Value> = new_v
            .as_object()?
            .iter()
            .filter(|(k, v)| old_v.as_object()?.get(*k).map_or(true, |o| o != *v))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if diff.is_empty() {
            None
        } else {
            Some(Self(diff))
        }
    }

    pub fn as_value(&self) -> Value {
        Value::Object(self.0.clone())
    }
}

// ── Payload Structure ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct AnalyticsPayload<'a> {
    event: &'static str,
    page_name: String,
    element_type: &'static str,
    interaction_type: &'a str,
    state_diff: &'a Value,
}

impl<'a> AnalyticsPayload<'a> {
    fn new(interaction_type: &'a str, state_diff: &'a Value) -> Self {
        Self {
            event: "ui_interaction",
            page_name: get_page_name(),
            element_type: "wasm_egui",
            interaction_type,
            state_diff,
        }
    }
}

fn get_page_name() -> String {
    let path = window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.last() {
        Some(last) => last.to_string(),
        None => "index".to_string(),
    }
}

// ── Events & SystemParam for UI capture ─────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use crate::egui_events::UiInteraction;

// ── SystemParam for analytics processing ────────────────────────────────────

#[derive(SystemParam)]
pub struct AnalyticsInput<'w> {
    pub current: Res<'w, CrtState>,
    pub event_state: ResMut<'w, EventState>,
    pub keyboard_events: EventReader<'w, KeyboardInput>,
    pub ui_events: MessageReader<'w, UiInteraction>,
}

fn detect_interaction(input: &mut AnalyticsInput) -> Option<InteractionType> {
    let mut interaction = None;

    for ev in input.keyboard_events.read() {
        if ev.state == ButtonState::Pressed {
            interaction = Some(InteractionType::KeyPress(ev.key_code));
        }
    }
    for ui_ev in input.ui_events.read() {
        interaction = Some(InteractionType::Named(ui_ev.widget_id.clone()));
    }

    interaction
}

// ── Proxy ─────────────────────────────────────────────────────────────────────

pub fn process_analytics(mut input: AnalyticsInput) {
    let diff = StateDiff::try_from(&input.event_state.previous, &input.current);

    let interaction = detect_interaction(&mut input);

    if let Some(diff) = &diff {
        input.event_state.previous = input.current.clone();

        if let Some(interaction) = interaction {
            dispatch_to_js(&interaction, diff);
        }
    }
}

// ── Dispatch JS ───────────────────────────────────────────────────────────────

fn dispatch_to_js(interaction: &InteractionType, diff: &StateDiff) {
    let payload = AnalyticsPayload::new(&interaction.as_str(), &diff.as_value());

    let js_value = match serde_wasm_bindgen::to_value(&payload) {
        Ok(v) => v,
        Err(_) => {
            web_sys::console::error_1(&"analytics: serialization failed".into());
            return;
        }
    };

    let mut init = CustomEventInit::new();
    init.detail(&js_value);

    if let Ok(evt) = CustomEvent::new_with_event_init_dict("analytics_event", &init) {
        if let Some(win) = window() {
            let _ = win.dispatch_event(&evt);
        }
    }
}
