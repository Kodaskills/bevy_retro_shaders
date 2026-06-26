use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Message, Clone)]
pub struct UiEvent {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub interaction_type: String,
}

/// Registers `UiEvent` so `EguiEventEmitter`'s `MessageWriter` validates on
/// native builds too (analytics, which also registers it, is wasm-only).
/// `add_message` is idempotent, so double-registration is harmless.
pub struct EguiEventsPlugin;

impl Plugin for EguiEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiEvent>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    DragStopped,
    Changed,
    LostFocus,
}

impl InteractionType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::DragStopped => "drag",
            Self::Changed => "click",
            Self::LostFocus => "blur",
        }
    }
}

#[derive(SystemParam)]
pub struct EguiEventEmitter<'w, 's> {
    contexts: EguiContexts<'w, 's>,
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    event_writer: MessageWriter<'w, UiEvent>,
}

impl<'w, 's> EguiEventEmitter<'w, 's> {
    pub fn ctx_mut(&mut self) -> egui::Context {
        self.contexts
            .ctx_mut()
            .expect("primary egui context")
            .clone()
    }

    pub fn emit(&mut self, response: egui::Response, interaction: InteractionType) {
        let should_emit = match interaction {
            InteractionType::DragStopped => response.drag_stopped(),
            InteractionType::Changed => response.changed(),
            InteractionType::LostFocus => response.lost_focus(),
        };
        if should_emit {
            self.event_writer.write(UiEvent {
                interaction_type: interaction.as_str().to_string(),
            });
        }
    }
}
