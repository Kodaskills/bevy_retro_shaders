use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::render::{render_graph::RenderGraphExt, RenderApp};
use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};
use bevy_egui::{egui, render::graph::NodeEgui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_retro_shaders::{CrtGlitch, CrtLabel, CrtPlugin, CrtSettings};
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
#[path = "common/analytics.rs"]
mod analytics;

/// Tracks clicks, keypresses, and checkbox toggles. No-op on desktop.
#[cfg(target_arch = "wasm32")]
fn track_interaction(interaction_type: &str, element_name: &str, element_location: &str) {
    analytics::emit_ui_interaction(interaction_type, element_name, element_location);
}
#[cfg(not(target_arch = "wasm32"))]
fn track_interaction(_: &str, _: &str, _: &str) {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ── Resources ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct DemoImages {
    names: Vec<String>,
    handles: Vec<Handle<Image>>,
    current_entity: Option<Entity>,
}

#[derive(Resource, Default)]
struct DemoTextState {
    entity: Option<Entity>,
}

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
struct ColorBox;

#[derive(Component)]
struct ImageSprite;

#[derive(Component)]
struct DemoTextMarker;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Serialize)]
struct CrtState {
    // CRT
    #[serde(rename = "crt_toggle")]
    enabled: bool,
    #[serde(rename = "crt_curvature")]
    curvature: f32,
    #[serde(rename = "crt_chromatic")]
    chromatic: f32,
    #[serde(rename = "crt_vignette")]
    vignette: f32,
    #[serde(rename = "crt_scanlines")]
    scanlines: f32,
    #[serde(rename = "crt_noise")]
    noise: f32,
    // Glitch
    #[serde(rename = "glitch_toggle")]
    glitch_enabled: bool,
    #[serde(rename = "glitch_intensity")]
    glitch_intensity: f32,
    #[serde(rename = "glitch_interval_min")]
    glitch_interval_min: f32,
    #[serde(rename = "glitch_interval_max")]
    glitch_interval_max: f32,
    #[serde(rename = "glitch_duration")]
    glitch_duration: f32,
    #[serde(rename = "glitch_horizontal_shift")]
    glitch_horizontal_shift: bool,
    #[serde(rename = "glitch_rgb_split")]
    glitch_rgb_split: bool,
    #[serde(rename = "glitch_noise")]
    glitch_noise: bool,
    #[serde(rename = "glitch_freeze")]
    glitch_freeze: bool,
    // Demo scene
    #[serde(rename = "scene_content")]
    selected_demo: usize,
    #[serde(rename = "scene_show")]
    show_sprite: bool,
    #[serde(rename = "scene_image")]
    selected_image: usize,
    #[serde(rename = "scene_image_mode")]
    image_mode: usize, // 0=sprite 1=background
    // Text
    #[serde(rename = "text_type")]
    text_demo: usize, // 0=none 1=title 2=paragraph
    #[serde(rename = "text_title")]
    title_input: String,
    #[serde(rename = "text_paragraph")]
    paragraph_input: String,
    // Bloom
    #[serde(rename = "bloom_toggle")]
    bloom_enabled: bool,
    #[serde(rename = "bloom_preset")]
    bloom_preset: usize, // 0=Natural 1=OldSchool 2=ScreenBlur 3=Anamorphic 4=Custom
    #[serde(rename = "bloom_intensity")]
    bloom_intensity: f32,
    #[serde(rename = "bloom_low_freq_boost")]
    bloom_low_freq_boost: f32,
    #[serde(rename = "bloom_low_freq_boost_curve")]
    bloom_low_freq_boost_curve: f32,
    #[serde(rename = "bloom_high_pass")]
    bloom_high_pass: f32,
    #[serde(rename = "bloom_threshold")]
    bloom_threshold: f32,
    #[serde(rename = "bloom_threshold_softness")]
    bloom_threshold_softness: f32,
    // Tonemapping
    #[serde(rename = "tonemapping_toggle")]
    tonemapping_enabled: bool,
    #[serde(rename = "tonemapping_preset")]
    tonemapping: usize,
    // UI
    #[serde(rename = "ui_panels")]
    panels_visible: bool,
}

impl Default for CrtState {
    fn default() -> Self {
        Self {
            enabled: false,
            curvature: 0.05,
            chromatic: 0.01,
            vignette: 0.0,
            scanlines: 0.95,
            noise: 0.0,
            glitch_enabled: false,
            glitch_intensity: 0.65,
            glitch_interval_min: 10.0,
            glitch_interval_max: 20.0,
            glitch_duration: 0.15,
            glitch_horizontal_shift: true,
            glitch_rgb_split: true,
            glitch_noise: true,
            glitch_freeze: true,
            selected_demo: 1,
            show_sprite: true,
            selected_image: 0,
            image_mode: 1, // background by default
            text_demo: 0,
            title_input: "Ha! I kill me!".to_string(),
            paragraph_input:
                "On Melmac, we had a saying: if at first you don't succeed, forget it.\nNo sense being a fool about it."
                    .to_string(),
            bloom_enabled: false,
            bloom_preset: 0,
            bloom_intensity: 0.15,
            bloom_low_freq_boost: 0.7,
            bloom_low_freq_boost_curve: 0.95,
            bloom_high_pass: 1.0,
            bloom_threshold: 0.0,
            bloom_threshold_softness: 0.0,
            tonemapping_enabled: false,
            tonemapping: 6, // TonyMcMapface (Bevy default)
            panels_visible: true,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn shortcuts_label() -> egui::text::LayoutJob {
    let sep = egui::text::TextFormat {
        color: egui::Color32::from_rgb(80, 80, 90),
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    append_key(
        &mut job,
        "C",
        " = CRT",
        egui::Color32::from_rgb(255, 140, 40),
    );
    job.append("   |   ", 0.0, sep.clone());
    append_key(
        &mut job,
        "G",
        " = Glitch",
        egui::Color32::from_rgb(60, 220, 110),
    );
    job.append("   |   ", 0.0, sep.clone());
    append_key(
        &mut job,
        "B",
        " = Bloom",
        egui::Color32::from_rgb(80, 160, 255),
    );
    job.append("   |   ", 0.0, sep.clone());
    append_key(
        &mut job,
        "T",
        " = Tonemapping",
        egui::Color32::from_rgb(255, 210, 60),
    );
    job.append("   |   ", 0.0, sep.clone());
    append_key(
        &mut job,
        "H",
        " = Show/Hide panels",
        egui::Color32::from_rgb(180, 130, 255),
    );
    job
}

fn append_key(job: &mut egui::text::LayoutJob, key: &str, label: &str, color: egui::Color32) {
    job.append(
        key,
        0.0,
        egui::text::TextFormat {
            color,
            ..Default::default()
        },
    );
    job.append(
        label,
        0.0,
        egui::text::TextFormat {
            color: egui::Color32::from_rgb(180, 180, 190),
            ..Default::default()
        },
    );
}

// ── Egui ordering ─────────────────────────────────────────────────────────────
// Ensure egui draws AFTER CRT, so panels are not affected by the shader.

struct EguiAfterCrtPlugin;

impl Plugin for EguiAfterCrtPlugin {
    fn build(&self, app: &mut App) {
        use bevy::core_pipeline::core_2d::graph::Core2d;

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        // CrtPlugin already adds EndMainPassPostProcessing → CrtLabel.
        // This edge ensures egui draws AFTER CRT, keeping panels shader-free.
        render_app.add_render_graph_edge(Core2d, CrtLabel, NodeEgui::EguiPass);
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_wasm() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run_app();
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    run_app();
}

fn run_app() {
    #[cfg(not(target_arch = "wasm32"))]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "CRT Effect Controls".into(),
            resolution: (1200u32, 800u32).into(),
            ..default()
        }),
        ..default()
    };

    #[cfg(target_arch = "wasm32")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas".to_string()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    let asset_plugin = AssetPlugin {
        // Images live in examples/assets/ — shader is embedded, unaffected
        file_path: "examples/assets".to_string(),
        ..default()
    };
    #[cfg(target_arch = "wasm32")]
    let asset_plugin = AssetPlugin {
        // Trunk copies examples/assets/ → dist/assets/, so prepend "assets" to all load paths.
        file_path: "assets".to_string(),
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..default()
    };

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(window_plugin).set(asset_plugin));

    app.add_plugins(EguiPlugin::default())
        .add_plugins(CrtPlugin)
        .add_plugins(EguiAfterCrtPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(EguiPrimaryContextPass, ui_controls)
        .add_systems(
            Update,
            (
                apply_crt_settings,
                apply_bloom,
                apply_tonemapping,
                update_demo_display,
                update_text_display,
                #[cfg(target_arch = "wasm32")]
                analytics::track_resource_changes::<CrtState>,
                #[cfg(target_arch = "wasm32")]
                signal_ready,
            ),
        )
        .insert_resource(CrtState::default());
    #[cfg(target_arch = "wasm32")]
    app.insert_resource(analytics::PreviousState(CrtState::default()))
        .insert_resource(analytics::AnalyticsDebounce::<CrtState>::default());
    app.insert_resource(DemoImages::default())
        .insert_resource(DemoTextState::default())
        .run();
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut demo_images: ResMut<DemoImages>,
) {
    // Native: scan the images directory at runtime.
    #[cfg(not(target_arch = "wasm32"))]
    {
        let images_dir = std::path::Path::new("./examples/assets/images");
        if let Ok(entries) = std::fs::read_dir(images_dir) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| matches!(ext.to_str(), Some("jpg" | "jpeg" | "png" | "webp")))
                        .unwrap_or(false)
                })
                .collect();
            paths.sort_by_key(|e| e.file_name());
            for entry in paths {
                let filename = entry.file_name().to_string_lossy().to_string();
                demo_images
                    .handles
                    .push(asset_server.load(format!("images/{filename}")));
                demo_images.names.push(filename);
            }
        }
    }

    // WASM: std::fs is unavailable — use the known asset list from examples/assets/images/.
    #[cfg(target_arch = "wasm32")]
    {
        for filename in ["melmac.jpg", "family-photo.jpg", "retro-gaming.jpg"] {
            demo_images
                .handles
                .push(asset_server.load(format!("images/{filename}")));
            demo_images.names.push(filename.to_string());
        }
    }

    // Native: HDR + Bloom + TonyMcMapface tonemapping.
    // WASM/WebGL2: Hdr requires Rgba16Float which is not universally available — skip it.
    // Bloom also requires HDR textures, so it is omitted on WASM.
    // The CRT shader already handles both paths via `if view.hdr` in crt.rs.
    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        Camera2d,
        bevy::render::view::Hdr,
        CrtSettings::default(),
        CrtGlitch::new(0.0, 10.0, 20.0, 0.15),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
    ));
    #[cfg(target_arch = "wasm32")]
    commands.spawn((
        Camera2d,
        bevy::render::view::Hdr,
        CrtSettings::default(),
        CrtGlitch::new(0.0, 10.0, 20.0, 0.15),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Msaa::Off, // WebGL2 does not support MSAA with post-processing
    ));

    let colors = [
        Color::srgb(1.0, 0.2, 0.2),
        Color::srgb(0.2, 1.0, 0.2),
        Color::srgb(0.2, 0.2, 1.0),
        Color::srgb(1.0, 1.0, 0.2),
        Color::srgb(0.2, 1.0, 1.0),
        Color::srgb(1.0, 0.2, 1.0),
    ];

    for (i, color) in colors.iter().enumerate() {
        commands.spawn((
            ColorBox,
            Sprite {
                color: *color,
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            Transform::from_xyz(-200.0 + (i as f32) * 80.0, 100.0, 0.0),
        ));
    }

    for (i, color) in colors.iter().enumerate() {
        commands.spawn((
            ColorBox,
            Sprite {
                color: *color,
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            Transform::from_xyz(-150.0 + (i as f32) * 60.0, -100.0, 0.0),
        ));
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

fn ui_controls(
    mut contexts: EguiContexts,
    mut state: ResMut<CrtState>,
    keys: Res<ButtonInput<KeyCode>>,
    demo_images: Res<DemoImages>,
) {
    let prev_enabled = state.enabled;
    let prev_glitch = state.glitch_enabled;
    let prev_bloom = state.bloom_enabled;
    let prev_tonemapping = state.tonemapping_enabled;
    // Glitch sub-options
    let prev_horiz_shift = state.glitch_horizontal_shift;
    let prev_rgb_split = state.glitch_rgb_split;
    let prev_glitch_noise = state.glitch_noise;
    let prev_glitch_freeze = state.glitch_freeze;
    // Bloom
    let prev_bloom_preset = state.bloom_preset;
    // Tonemapping
    let prev_tonemapping_preset = state.tonemapping;
    // Scene
    let prev_show_sprite = state.show_sprite;
    let prev_selected_demo = state.selected_demo;
    let prev_selected_image = state.selected_image;
    let prev_image_mode = state.image_mode;
    // Text
    let prev_text_demo = state.text_demo;

    // ── Keyboard shortcuts ───────────────────────────────────────────────────
    // Memorizes which toggles were changed by keypresses, so that the post-UI click detection can ignore those changes and
    // avoid emitting duplicate analytics events (keypress + click) for the same change.
    let key_crt = keys.just_pressed(KeyCode::KeyC);
    let key_glitch = keys.just_pressed(KeyCode::KeyG);
    let key_bloom = keys.just_pressed(KeyCode::KeyB);
    let key_tonemapping = keys.just_pressed(KeyCode::KeyT);

    if key_crt {
        state.enabled = !state.enabled;
        track_interaction("keypress", "crt_toggle", "canvas");
    }
    if key_glitch {
        state.glitch_enabled = !state.glitch_enabled;
        track_interaction("keypress", "glitch_toggle", "canvas");
    }
    if key_bloom {
        state.bloom_enabled = !state.bloom_enabled;
        track_interaction("keypress", "bloom_toggle", "canvas");
    }
    if key_tonemapping {
        state.tonemapping_enabled = !state.tonemapping_enabled;
        track_interaction("keypress", "tonemapping_toggle", "canvas");
    }

    // ── Render EGUI  ───────────────────────────────────────────────────────────
    let ctx = contexts.ctx_mut().expect("failed to get egui context");

    if !state.panels_visible {
        return;
    }

    // Dark visuals
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgba_premultiplied(12, 12, 16, 230);
    visuals.panel_fill = egui::Color32::from_rgba_premultiplied(8, 8, 12, 245);
    ctx.set_visuals(visuals);

    // ── Left column ──────────────────────────────────────────────────────────

    egui::Window::new("CRT Effect")
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("▣ CRT Effect");
            ui.separator();
            ui.checkbox(&mut state.enabled, "Enabled");

            if state.enabled {
                ui.separator();
                ui.add(egui::Slider::new(&mut state.curvature, 0.0..=0.2).text("Curvature"));
                ui.add(egui::Slider::new(&mut state.chromatic, 0.0..=0.05).text("Chromatic"));
                ui.add(egui::Slider::new(&mut state.vignette, 0.0..=1.0).text("Vignette"));
                ui.add(egui::Slider::new(&mut state.scanlines, 0.0..=1.0).text("Scanlines"));
                ui.add(egui::Slider::new(&mut state.noise, 0.0..=0.25).text("Noise / Grain"));
            }
        });

    egui::Window::new("Glitch")
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("⚡ Glitch");
            ui.separator();
            ui.checkbox(&mut state.glitch_enabled, "Enabled");

            if state.glitch_enabled {
                ui.separator();
                ui.add(egui::Slider::new(&mut state.glitch_intensity, 0.0..=1.0).text("Intensity"));
                ui.add(
                    egui::Slider::new(&mut state.glitch_interval_min, 0.5..=30.0)
                        .text("Interval Min"),
                );
                ui.add(
                    egui::Slider::new(&mut state.glitch_interval_max, 1.0..=60.0)
                        .text("Interval Max"),
                );
                ui.add(egui::Slider::new(&mut state.glitch_duration, 0.05..=1.0).text("Duration"));
                ui.separator();
                ui.checkbox(&mut state.glitch_horizontal_shift, "Horizontal Shift");
                ui.checkbox(&mut state.glitch_rgb_split, "RGB Split");
                ui.checkbox(&mut state.glitch_noise, "Noise");
                ui.checkbox(&mut state.glitch_freeze, "Freeze");
                ui.separator();
                ui.label("ENTER - Toggle Glitch");
            }
        });

    egui::Window::new("Scene").resizable(false).show(ctx, |ui| {
        ui.heading("Scene");
        ui.separator();
        ui.label("Content:");
        ui.radio_value(&mut state.selected_demo, 0, "Colors");
        ui.radio_value(&mut state.selected_demo, 1, "Images");
        ui.separator();
        ui.checkbox(&mut state.show_sprite, "Show");

        if state.selected_demo == 1 {
            ui.separator();
            ui.label("Image:");
            for (i, name) in demo_images.names.iter().enumerate() {
                ui.radio_value(&mut state.selected_image, i, name.as_str());
            }
            ui.separator();
            ui.label("Display mode:");
            ui.radio_value(&mut state.image_mode, 0, "Sprite");
            ui.radio_value(&mut state.image_mode, 1, "Background");
        }

        ui.separator();
        ui.label("Text:");
        ui.radio_value(&mut state.text_demo, 0, "None");
        ui.radio_value(&mut state.text_demo, 1, "Title");
        ui.radio_value(&mut state.text_demo, 2, "Paragraph");

        if state.text_demo == 1 {
            ui.separator();
            ui.label("Title:");
            ui.text_edit_singleline(&mut state.title_input);
        }
        if state.text_demo == 2 {
            ui.separator();
            ui.label("Paragraph:");
            ui.text_edit_multiline(&mut state.paragraph_input);
        }
    });

    // ── Right column ─────────────────────────────────────────────────────────

    egui::Window::new("Bloom")
        .default_pos([990.0, 10.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("✨ Bloom");
            ui.separator();
            ui.checkbox(&mut state.bloom_enabled, "Enabled");

            if state.bloom_enabled {
                ui.separator();
                ui.label("Preset:");
                ui.radio_value(&mut state.bloom_preset, 0, "Natural");
                ui.radio_value(&mut state.bloom_preset, 1, "Old School");
                ui.radio_value(&mut state.bloom_preset, 2, "Screen Blur");
                ui.radio_value(&mut state.bloom_preset, 3, "Anamorphic");
                ui.radio_value(&mut state.bloom_preset, 4, "Custom");

                if state.bloom_preset == 4 {
                    ui.separator();
                    ui.label("Custom:");
                    ui.add(
                        egui::Slider::new(&mut state.bloom_intensity, 0.0..=1.0).text("Intensity"),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.bloom_low_freq_boost, 0.0..=1.0)
                            .text("Low Freq Boost"),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.bloom_low_freq_boost_curve, 0.0..=1.0)
                            .text("Boost Curve"),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.bloom_high_pass, 0.0..=1.0).text("High Pass"),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.bloom_threshold, 0.0..=1.0).text("Threshold"),
                    );
                    ui.add(
                        egui::Slider::new(&mut state.bloom_threshold_softness, 0.0..=1.0)
                            .text("Threshold Softness"),
                    );
                }
            }
        });

    egui::Window::new("Tonemapping")
        .default_pos([990.0, 380.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("🎨 Tonemapping");
            ui.separator();
            ui.checkbox(&mut state.tonemapping_enabled, "Enabled");

            if state.tonemapping_enabled {
                ui.separator();
                ui.label("Preset:");
                ui.radio_value(&mut state.tonemapping, 1, "Reinhard");
                ui.radio_value(&mut state.tonemapping, 2, "Reinhard Luminance");
                ui.radio_value(&mut state.tonemapping, 3, "ACES Fitted");
                ui.radio_value(&mut state.tonemapping, 4, "AgX");
                ui.radio_value(&mut state.tonemapping, 5, "Somewhat Boring");
                ui.radio_value(&mut state.tonemapping, 6, "Tony McMapface ✓");
                ui.radio_value(&mut state.tonemapping, 7, "Blender Filmic");
            }
        });

    egui::TopBottomPanel::bottom("shortcuts").show(ctx, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label(shortcuts_label());
        });
    });

    // ── Click Detection (Post-UI Render) ─────────────────────────────
    // Detects changes to toggles that were NOT caused by keypresses
    if !key_crt && state.enabled != prev_enabled {
        track_interaction("click", "crt_toggle", "canvas");
    }
    if !key_glitch && state.glitch_enabled != prev_glitch {
        track_interaction("click", "glitch_toggle", "canvas");
    }
    if !key_bloom && state.bloom_enabled != prev_bloom {
        track_interaction("click", "bloom_toggle", "canvas");
    }
    if !key_tonemapping && state.tonemapping_enabled != prev_tonemapping {
        track_interaction("click", "tonemapping_toggle", "canvas");
    }

    // Glitch sub-options
    if state.glitch_horizontal_shift != prev_horiz_shift {
        track_interaction("click", "glitch_horizontal_shift", "canvas");
    }
    if state.glitch_rgb_split != prev_rgb_split {
        track_interaction("click", "glitch_rgb_split", "canvas");
    }
    if state.glitch_noise != prev_glitch_noise {
        track_interaction("click", "glitch_noise", "canvas");
    }
    if state.glitch_freeze != prev_glitch_freeze {
        track_interaction("click", "glitch_freeze", "canvas");
    }

    // Bloom preset (radio buttons)
    if state.bloom_preset != prev_bloom_preset {
        track_interaction("click", "bloom_preset", "canvas");
    }

    // Tonemapping preset (radio buttons)
    if state.tonemapping != prev_tonemapping_preset {
        track_interaction("click", "tonemapping_preset", "canvas");
    }

    // Scene options
    if state.show_sprite != prev_show_sprite {
        track_interaction("click", "scene_show", "canvas");
    }
    if state.selected_demo != prev_selected_demo {
        track_interaction("click", "scene_content", "canvas");
    }
    if state.selected_image != prev_selected_image {
        track_interaction("click", "scene_image", "canvas");
    }
    if state.image_mode != prev_image_mode {
        track_interaction("click", "scene_image_mode", "canvas");
    }

    // Text options
    if state.text_demo != prev_text_demo {
        track_interaction("click", "text_type", "canvas");
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn apply_crt_settings(
    state: Res<CrtState>,
    mut settings_query: Query<&mut CrtSettings, With<Camera>>,
    mut glitch_query: Query<&mut CrtGlitch, With<Camera>>,
) {
    if !state.is_changed() {
        return;
    }
    for mut settings in settings_query.iter_mut() {
        if state.enabled {
            settings.curvature = state.curvature;
            settings.chromatic_aberration = state.chromatic;
            settings.vignette_strength = state.vignette;
            settings.scanline_strength = state.scanlines;
            settings.noise_strength = state.noise;
        } else {
            settings.curvature = 0.0;
            settings.chromatic_aberration = 0.0;
            settings.vignette_strength = state.vignette;
            settings.scanline_strength = 0.0;
            settings.noise_strength = 0.0;
        }
    }
    for mut glitch in glitch_query.iter_mut() {
        if state.glitch_enabled {
            glitch.intensity = state.glitch_intensity;
            glitch.interval_min = state.glitch_interval_min;
            glitch.interval_max = state.glitch_interval_max;
            glitch.duration = state.glitch_duration;
            glitch.horizontal_shift = state.glitch_horizontal_shift;
            glitch.rgb_split = state.glitch_rgb_split;
            glitch.noise = state.glitch_noise;
            glitch.freeze = state.glitch_freeze;
        } else {
            glitch.intensity = 0.0;
            glitch.horizontal_shift = false;
            glitch.rgb_split = false;
            glitch.noise = false;
            glitch.freeze = false;
        }
    }
}

fn apply_bloom(mut commands: Commands, state: Res<CrtState>, cameras: Query<Entity, With<Camera>>) {
    if !state.is_changed() {
        return;
    }
    for entity in cameras.iter() {
        let bloom = if !state.bloom_enabled {
            Bloom {
                intensity: 0.0,
                ..Bloom::NATURAL
            }
        } else {
            match state.bloom_preset {
                1 => Bloom::OLD_SCHOOL,
                2 => Bloom::SCREEN_BLUR,
                3 => Bloom::ANAMORPHIC,
                4 => Bloom {
                    intensity: state.bloom_intensity,
                    low_frequency_boost: state.bloom_low_freq_boost,
                    low_frequency_boost_curvature: state.bloom_low_freq_boost_curve,
                    high_pass_frequency: state.bloom_high_pass,
                    prefilter: BloomPrefilter {
                        threshold: state.bloom_threshold,
                        threshold_softness: state.bloom_threshold_softness,
                    },
                    composite_mode: if state.bloom_threshold > 0.0 {
                        BloomCompositeMode::Additive
                    } else {
                        BloomCompositeMode::EnergyConserving
                    },
                    ..default()
                },
                _ => Bloom::NATURAL,
            }
        };
        commands.entity(entity).insert(bloom);
    }
}

fn apply_tonemapping(
    mut commands: Commands,
    state: Res<CrtState>,
    cameras: Query<Entity, With<Camera>>,
) {
    if !state.is_changed() {
        return;
    }
    for entity in cameras.iter() {
        let tm = if !state.tonemapping_enabled {
            Tonemapping::None
        } else {
            match state.tonemapping {
                1 => Tonemapping::Reinhard,
                2 => Tonemapping::ReinhardLuminance,
                3 => Tonemapping::AcesFitted,
                4 => Tonemapping::AgX,
                5 => Tonemapping::SomewhatBoringDisplayTransform,
                7 => Tonemapping::BlenderFilmic,
                _ => Tonemapping::TonyMcMapface,
            }
        };
        commands.entity(entity).insert(tm);
    }
}

fn update_demo_display(
    mut commands: Commands,
    state: Res<CrtState>,
    mut demo_images: ResMut<DemoImages>,
    mut color_boxes: Query<&mut Visibility, With<ColorBox>>,
    image_sprites: Query<Entity, With<ImageSprite>>,
    windows: Query<&Window>,
) {
    if !state.is_changed() {
        return;
    }

    let show_colors = state.show_sprite && state.selected_demo == 0;
    let show_image = state.show_sprite && state.selected_demo == 1;

    for mut vis in color_boxes.iter_mut() {
        *vis = if show_colors {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !show_image {
        for e in image_sprites.iter() {
            commands.entity(e).despawn();
        }
        demo_images.current_entity = None;
        return;
    }

    let Some(handle) = demo_images.handles.get(state.selected_image).cloned() else {
        return;
    };

    for e in image_sprites.iter() {
        commands.entity(e).despawn();
    }
    demo_images.current_entity = None;

    let (size, z) = match state.image_mode {
        1 => {
            let win_size = windows
                .single()
                .map(|w| Vec2::new(w.width(), w.height()))
                .unwrap_or(Vec2::new(1200.0, 800.0));
            (win_size, -100.0)
        }
        _ => (Vec2::new(600.0, 400.0), 0.0),
    };

    let entity = commands
        .spawn((
            ImageSprite,
            Sprite {
                image: handle,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, z),
        ))
        .id();

    demo_images.current_entity = Some(entity);
}

fn update_text_display(
    mut commands: Commands,
    state: Res<CrtState>,
    mut text_state: ResMut<DemoTextState>,
) {
    if !state.is_changed() {
        return;
    }

    if let Some(e) = text_state.entity.take() {
        commands.entity(e).despawn();
    }

    let entity = match state.text_demo {
        1 => commands
            .spawn((
                DemoTextMarker,
                Text2d::new(state.title_input.clone()),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.8)),
                Transform::from_xyz(0.0, 250.0, 10.0),
            ))
            .id(),
        2 => commands
            .spawn((
                DemoTextMarker,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Percent(50.0),
                        ..default()
                    },
                    Text::new(state.paragraph_input.clone()),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.9, 0.75)),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            })
            .id(),
        _ => return,
    };

    text_state.entity = Some(entity);
}

// After enough frames have rendered, all shader pipelines are compiled.
// Dispatch BevyAppReady so the JS loader can hide itself.
#[cfg(target_arch = "wasm32")]
fn signal_ready(mut frame: Local<u32>) {
    *frame += 1;
    if *frame == 10 {
        if let Some(window) = web_sys::window() {
            if let Ok(event) = web_sys::CustomEvent::new("BevyAppReady") {
                let _ = window.dispatch_event(&event);
            }
        }
    }
}
