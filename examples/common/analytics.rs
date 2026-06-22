#![cfg(target_arch = "wasm32")]
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use serde::Serialize;
use serde_json::Value;
use web_sys::{window, CustomEvent, CustomEventInit};

use crate::egui_events::UiEvent;
use crate::CrtState;

const TRACKED_KEYS: [KeyCode; 5] = [
    KeyCode::KeyC,
    KeyCode::KeyG,
    KeyCode::KeyB,
    KeyCode::KeyT,
    KeyCode::KeyH,
];

pub struct AnalyticsPlugin;

impl Plugin for AnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiEvent>()
            .insert_resource(AnalyticsState::default())
            .add_systems(PostUpdate, process_analytics);
    }
}

#[derive(Resource)]
struct AnalyticsState {
    previous: CrtState,
}

impl Default for AnalyticsState {
    fn default() -> Self {
        Self {
            previous: CrtState::default(),
        }
    }
}

#[derive(Serialize)]
struct AnalyticsPayload<'a> {
    event: &'static str,
    page_name: String,
    element_type: &'static str,
    interaction_type: &'a str,
    changes: serde_json::Map<String, Value>,
}

fn get_page_name() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_default()
        .split('/')
        .filter(|s| !s.is_empty())
        .last()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "index".to_string())
}

fn detect_changes(old: &CrtState, current: &CrtState) -> Option<serde_json::Map<String, Value>> {
    let old_v = serde_json::to_value(old).ok()?;
    let new_v = serde_json::to_value(current).ok()?;

    let changes: serde_json::Map<String, Value> = new_v
        .as_object()?
        .iter()
        .filter(|(k, v)| {
            if let Some(obj) = old_v.as_object() {
                obj.get(*k).map_or(true, |o| o != *v)
            } else {
                false
            }
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn process_analytics(
    current: Res<CrtState>,
    mut ui_events: MessageReader<UiEvent>,
    mut analytics_state: ResMut<AnalyticsState>,
    mut key_reader: MessageReader<KeyboardInput>,
    mut egui_ctx: bevy_egui::EguiContexts,
) {
    let mut interaction_type: Option<String> = None;

    // Don't report keyboard presses when user is typing in an egui text field
    let wants_keyboard = egui_ctx
        .ctx_mut()
        .is_ok_and(|ctx| ctx.wants_keyboard_input());
    if !wants_keyboard {
        for ev in key_reader.read() {
            if ev.state == ButtonState::Pressed && TRACKED_KEYS.contains(&ev.key_code) {
                interaction_type = Some(format!("key_press:{:?}", ev.key_code));
            }
        }
    }
    for ev in ui_events.read() {
        interaction_type = Some(ev.interaction_type.clone());
    }

    let Some(interaction) = interaction_type else {
        return;
    };

    let old = analytics_state.previous.clone();

    if let Some(changes) = detect_changes(&old, &current) {
        dispatch_to_js(&interaction, &changes);
    }

    analytics_state.previous = (*current).clone();
}

fn dispatch_to_js(interaction_type: &str, changes: &serde_json::Map<String, Value>) {
    let page_name = get_page_name();

    let payload = AnalyticsPayload {
        event: "ui_interaction",
        page_name,
        element_type: "wasm_egui",
        interaction_type,
        changes: changes.clone(),
    };

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    let js_value = match Serialize::serialize(&payload, &serializer) {
        Ok(v) => v,
        Err(_) => {
            web_sys::console::error_1(&"analytics: serialization failed".into());
            return;
        }
    };

    let init = CustomEventInit::new();
    init.set_detail(&js_value);

    if let Ok(evt) = CustomEvent::new_with_event_init_dict("analytics_event", &init) {
        if let Some(win) = window() {
            let _ = win.dispatch_event(&evt);
        }
    }
}
