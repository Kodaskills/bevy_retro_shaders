// analytics.rs
#![cfg(target_arch = "wasm32")]
use bevy::prelude::*;
use serde::Serialize;
use serde_json::Value;
use web_sys::{window, CustomEvent, CustomEventInit};

/// Returns the current page name by reading the data-bevy-event attribute on <body>.
/// Falls back to "unknown" if not found.
pub fn get_page_name() -> String {
    let Some(win) = window() else {
        return "unknown".to_string();
    };
    let Some(doc) = win.document() else {
        return "unknown".to_string();
    };
    let Some(body) = doc.body() else {
        return "unknown".to_string();
    };

    match body.dataset().get("bevyEvent").as_deref() {
        Some("BevyApp3dReady") => "3d".to_string(),
        Some("BevyAppReady") => "demo".to_string(),
        _ => "unknown".to_string(),
    }
}

// ── Shared Analytics Logic ──────────────────────────────────────────────────

#[derive(Resource)]
pub struct AnalyticsDebounce<T: Resource + Serialize + Clone> {
    pub pending: Option<T>,
    pub timer: f32,
}

impl<T: Resource + Serialize + Clone> Default for AnalyticsDebounce<T> {
    fn default() -> Self {
        Self {
            pending: None,
            timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct PreviousState<T: Resource + Serialize + Clone>(pub T);

pub const DEBOUNCE_SECS: f32 = 2.0;

/// System that tracks changes to a resource and emits analytics after a debounce period.
pub fn track_resource_changes<T: Resource + Serialize + Clone>(
    current: Res<T>,
    mut previous: ResMut<PreviousState<T>>,
    mut debounce: ResMut<AnalyticsDebounce<T>>,
    time: Res<Time>,
) {
    {
        if current.is_changed() {
            debounce.pending = Some((*current).clone());
            debounce.timer = DEBOUNCE_SECS;
        }
        if debounce.pending.is_some() {
            debounce.timer -= time.delta_secs();
            if debounce.timer <= 0.0 {
                if let Some(new_state) = debounce.pending.take() {
                    emit_slider_change(&previous.0, &new_state);
                    previous.0 = new_state;
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (current, previous, debounce, time);
    }
}

// ── Low-level Emitter ───────────────────────────────────────────────────────

/// Dispatches a CustomEvent("analytics_event") on window with the given payload.
fn dispatch_analytics_event(payload: Value) {
    let json_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            web_sys::console::error_1(&format!("analytics: serialization failed: {e}").into());
            return;
        }
    };
    let detail = match js_sys::JSON::parse(&json_str) {
        Ok(v) => v,
        Err(e) => {
            web_sys::console::error_1(&format!("analytics: JSON parse failed: {e:?}").into());
            return;
        }
    };
    let mut init = CustomEventInit::new();
    init.detail(&detail);
    let event = match CustomEvent::new_with_event_init_dict("analytics_event", &init) {
        Ok(e) => e,
        Err(e) => {
            web_sys::console::error_1(&format!("analytics: event creation failed: {e:?}").into());
            return;
        }
    };
    if let Some(win) = window() {
        let _ = win.dispatch_event(&event);
    }
}

/// Emits a ui_interaction event for clicks, keypresses, and checkbox toggles.
pub fn emit_ui_interaction(interaction_type: &str, element_name: &str, element_location: &str) {
    let payload = serde_json::json!({
        "event": "ui_interaction",
        "interaction_type": interaction_type,
        "page_name": get_page_name(),
        "element_type": "wasm_egui",
        "element_name": element_name,
        "element_location": element_location,
    });
    dispatch_analytics_event(payload);
}

/// Compares two states and returns only the fields that changed,
/// keeping only numeric values (sliders).
/// Toggles (booleans) and Presets (indices already tracked via click) are excluded.
fn diff_sliders<T: Serialize>(old: &T, new: &T) -> Value {
    // List of keys to ignore because they are presets/radios already tracked via "click" interaction.
    let ignored_keys = [
        "bloom_preset",
        "tonemapping_preset",
        "scene_content",
        "scene_image",
        "scene_image_mode",
        "text_type",
    ];

    let old_val = match serde_json::to_value(old) {
        Ok(v) => v,
        Err(_) => return Value::Null,
    };
    let new_val = match serde_json::to_value(new) {
        Ok(v) => v,
        Err(_) => return Value::Null,
    };

    match (old_val, new_val) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            let mut diff = serde_json::Map::new();
            for (key, val) in &new_map {
                // Only keep numeric values that are NOT in the ignored list
                if !val.is_number() || ignored_keys.contains(&key.as_str()) {
                    continue;
                }
                if old_map.get(key) != Some(val) {
                    diff.insert(key.clone(), val.clone());
                }
            }
            Value::Object(diff)
        }
        _ => Value::Null,
    }
}

/// Emits a 'state_change' event for debounced slider movements.
pub fn emit_slider_change<T: Serialize>(old: &T, new: &T) {
    let diff = diff_sliders(old, new);
    if diff.is_null() || diff.as_object().map_or(true, |o| o.is_empty()) {
        return;
    }

    let element_name = if let Value::Object(map) = &diff {
        let keys: Vec<&String> = map.keys().collect();
        match keys.len() {
            1 => keys[0].to_string(),
            _ => "multiple_sliders".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    let payload = serde_json::json!({
        "event": "ui_interaction",
        "interaction_type": "state_change",
        "page_name": get_page_name(),
        "element_type": "wasm_egui",
        "element_name": element_name,
        "element_location": "canvas",
        "system_state": diff,
    });
    dispatch_analytics_event(payload);
}
