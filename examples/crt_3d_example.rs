// 3D camera example for bevy_retro_shaders
// Demonstrates CRT post-processing on a Camera3d with full controls.

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter},
    prelude::*,
    render::{render_graph::RenderGraphExt, view::Hdr, RenderApp},
};
use bevy_egui::{egui, render::graph::NodeEgui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_retro_shaders::{CrtGlitch, CrtLabel, CrtPlugin, CrtSettings};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ── Marker ────────────────────────────────────────────────────────────────────

#[derive(Component)]
struct Rotating {
    speed: f32,
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Resource)]
struct CrtState {
    // CRT
    enabled: bool,
    curvature: f32,
    chromatic: f32,
    vignette: f32,
    scanlines: f32,
    noise: f32,
    // Glitch
    glitch_enabled: bool,
    glitch_intensity: f32,
    glitch_interval_min: f32,
    glitch_interval_max: f32,
    glitch_duration: f32,
    glitch_horizontal_shift: bool,
    glitch_rgb_split: bool,
    glitch_noise: bool,
    glitch_freeze: bool,
    // Bloom
    bloom_enabled: bool,
    bloom_preset: usize, // 0=Natural 1=OldSchool 2=ScreenBlur 3=Anamorphic 4=Custom
    bloom_intensity: f32,
    bloom_low_freq_boost: f32,
    bloom_low_freq_boost_curve: f32,
    bloom_high_pass: f32,
    bloom_threshold: f32,
    bloom_threshold_softness: f32,
    // Tonemapping
    tonemapping_enabled: bool,
    tonemapping: usize,
    // UI
    panels_visible: bool,
}

impl Default for CrtState {
    fn default() -> Self {
        Self {
            enabled: false,
            curvature: 0.06,
            chromatic: 0.01,
            vignette: 0.3,
            scanlines: 0.9,
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
            bloom_enabled: false,
            bloom_preset: 0,
            bloom_intensity: 0.15,
            bloom_low_freq_boost: 0.7,
            bloom_low_freq_boost_curve: 0.95,
            bloom_high_pass: 1.0,
            bloom_threshold: 0.0,
            bloom_threshold_softness: 0.0,
            tonemapping_enabled: false,
            tonemapping: 6, // TonyMcMapface
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
    job.append("   |   ", 0.0, sep);
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

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    run_app();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_wasm() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run_app();
}

struct EguiAfterCrtPlugin;
impl Plugin for EguiAfterCrtPlugin {
    fn build(&self, app: &mut App) {
        use bevy::core_pipeline::core_3d::graph::Core3d;
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return; };
        render_app.add_render_graph_edge(Core3d, CrtLabel, NodeEgui::EguiPass);
    }
}

fn run_app() {
    #[cfg(not(target_arch = "wasm32"))]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "CRT 3D Example".into(),
            resolution: (1200u32, 800u32).into(),
            ..default()
        }),
        ..default()
    };
    #[cfg(target_arch = "wasm32")]
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy-canvas-3d".to_string()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    };

    #[cfg(target_arch = "wasm32")]
    let asset_plugin = bevy::asset::AssetPlugin {
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..default()
    };

    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(window_plugin));
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(DefaultPlugins.set(window_plugin).set(asset_plugin));

    app.add_plugins(EguiPlugin::default())
        .add_plugins(CrtPlugin)
        .add_plugins(EguiAfterCrtPlugin)
        .insert_resource(CrtState::default())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_controls)
        .add_systems(
            Update,
            (
                rotate_meshes,
                apply_crt_settings,
                apply_bloom,
                apply_tonemapping,
                #[cfg(target_arch = "wasm32")]
                signal_ready,
            ),
        );

    // WebGL2 does not support MSAA with post-processing
    #[cfg(target_arch = "wasm32")]
    app.add_systems(PostStartup, disable_msaa);

    app.run();
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera3d — same API as Camera2d: just add CrtSettings + CrtGlitch
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Hdr,                        // required for HDR pipeline
        Tonemapping::TonyMcMapface, // always present, just changed at runtime
        Bloom::NATURAL,             // always present, just changed at runtime
        CrtSettings {
            curvature: 0.06,
            chromatic_aberration: 0.01,
            vignette_strength: 0.3,
            scanline_strength: 0.9,
            ..default()
        },
        CrtGlitch::default(),
        AmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            ..default()
        },
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Floor
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.12),
            perceptual_roughness: 0.9,
            ..default()
        })),
    ));

    // Central glowing cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 1.6, 1.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.4, 0.1),
            emissive: LinearRgba::new(1.5, 0.4, 0.0, 1.0),
            perceptual_roughness: 0.3,
            metallic: 0.7,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.8, 0.0),
        Rotating { speed: 0.6 },
    ));

    // Orbiting spheres
    let sphere_mesh = meshes.add(Sphere::new(0.45).mesh().uv(32, 16));
    let orbit_data = [
        (
            0.0_f32,
            Color::srgb(0.2, 0.5, 1.0),
            LinearRgba::new(0.0, 0.5, 2.0, 1.0),
            0.4_f32,
        ),
        (
            std::f32::consts::TAU / 3.0,
            Color::srgb(0.2, 1.0, 0.4),
            LinearRgba::new(0.0, 2.0, 0.3, 1.0),
            -0.5,
        ),
        (
            2.0 * std::f32::consts::TAU / 3.0,
            Color::srgb(1.0, 0.8, 0.2),
            LinearRgba::new(2.0, 0.8, 0.0, 1.0),
            0.3,
        ),
    ];
    for (angle, base_color, emissive, speed) in orbit_data {
        let x = angle.cos() * 3.2;
        let z = angle.sin() * 3.2;
        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                emissive,
                perceptual_roughness: 0.15,
                metallic: 0.9,
                ..default()
            })),
            Transform::from_xyz(x, 0.45, z),
            Rotating { speed },
        ));
    }

    // Corner pillars
    let pillar_mesh = meshes.add(Cuboid::new(0.3, 2.5, 0.3));
    let pillar_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.4),
        perceptual_roughness: 0.7,
        metallic: 0.3,
        ..default()
    });
    for (x, z) in [(-4.5_f32, -4.5_f32), (4.5, -4.5), (-4.5, 4.5), (4.5, 4.5)] {
        commands.spawn((
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(pillar_mat.clone()),
            Transform::from_xyz(x, 1.25, z),
        ));
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

fn ui_controls(
    mut contexts: EguiContexts,
    mut state: ResMut<CrtState>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    if keys.just_pressed(KeyCode::KeyH) {
        state.panels_visible = !state.panels_visible;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        state.enabled = !state.enabled;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        state.glitch_enabled = !state.glitch_enabled;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        state.bloom_enabled = !state.bloom_enabled;
    }
    if keys.just_pressed(KeyCode::KeyT) {
        state.tonemapping_enabled = !state.tonemapping_enabled;
    }

    let ctx = contexts.ctx_mut()?;

    if !state.panels_visible {
        return Ok(());
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

    Ok(())
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn rotate_meshes(time: Res<Time>, mut query: Query<(&Rotating, &mut Transform)>) {
    for (rotating, mut transform) in query.iter_mut() {
        transform.rotate_y(rotating.speed * time.delta_secs());
    }
}

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
            settings.vignette_strength = 0.0;
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

#[cfg(target_arch = "wasm32")]
fn disable_msaa(mut commands: Commands, cameras: Query<Entity, With<Camera>>) {
    for entity in cameras.iter() {
        commands.entity(entity).insert(Msaa::Off);
    }
}

#[cfg(target_arch = "wasm32")]
fn signal_ready(mut frame: Local<u32>) {
    *frame += 1;
    if *frame == 10 {
        if let Some(window) = web_sys::window() {
            if let Ok(event) = web_sys::CustomEvent::new("BevyApp3dReady") {
                let _ = window.dispatch_event(&event);
            }
        }
    }
}
