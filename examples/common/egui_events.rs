use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Message, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct UiInteraction {
    pub widget_type: String,
    pub widget_id: String,
}

#[derive(SystemParam)]
pub struct EguiUi<'w, 's> {
    contexts: EguiContexts<'w, 's>,
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    interaction_writer: MessageWriter<'w, UiInteraction>,
}

impl<'w, 's> EguiUi<'w, 's> {
    pub fn ctx_mut(&mut self) -> egui::Context {
        self.contexts
            .ctx_mut()
            .expect("primary egui context")
            .clone()
    }

    pub fn slider(
        &mut self,
        ui: &mut egui::Ui,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        id: impl Into<String>,
    ) {
        let label = id.into();
        let response = ui.add(egui::Slider::new(value, range).text(&label));
        if response.drag_stopped() {
            self.interaction_writer.write(UiInteraction {
                widget_type: "slider".to_string(),
                widget_id: label,
            });
        }
    }

    pub fn checkbox(&mut self, ui: &mut egui::Ui, value: &mut bool, label: impl Into<String>) {
        let label = label.into();
        let response = ui.checkbox(value, &label);
        if response.changed() {
            self.interaction_writer.write(UiInteraction {
                widget_type: "checkbox".to_string(),
                widget_id: label,
            });
        }
    }

    pub fn radio_value<T: PartialEq>(
        &mut self,
        ui: &mut egui::Ui,
        current: &mut T,
        alternative: T,
        label: impl Into<String>,
    ) {
        let label = label.into();
        let response = ui.radio_value(current, alternative, &label);
        if response.changed() {
            self.interaction_writer.write(UiInteraction {
                widget_type: "radio".to_string(),
                widget_id: label,
            });
        }
    }

    pub fn text_edit_singleline(
        &mut self,
        ui: &mut egui::Ui,
        text: &mut String,
        id: impl Into<String>,
    ) {
        let label = id.into();
        let response = ui.text_edit_singleline(text);
        if response.lost_focus() && response.changed() {
            self.interaction_writer.write(UiInteraction {
                widget_type: "text_input".to_string(),
                widget_id: label,
            });
        }
    }

    pub fn text_edit_multiline(
        &mut self,
        ui: &mut egui::Ui,
        text: &mut String,
        id: impl Into<String>,
    ) {
        let label = id.into();
        let response = ui.text_edit_multiline(text);
        if response.lost_focus() && response.changed() {
            self.interaction_writer.write(UiInteraction {
                widget_type: "text_input".to_string(),
                widget_id: label,
            });
        }
    }
}
