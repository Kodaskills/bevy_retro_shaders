// CRT post-processing effect for Bevy
// Adds barrel distortion, chromatic aberration, vignette, scanlines, and occasional glitches.
// Add `CrtSettings` + `CrtGlitch` to a Camera2d entity to enable.

use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        core_3d::graph::{Core3d, Node3d},
        FullscreenShader,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};
use rand::Rng;

// ─── GPU settings (mirrored exactly in crt.wgsl) ────────────────────────────

/// Add to a Camera2d entity to enable the CRT effect.
/// Glitch fields are updated automatically by `update_crt_glitch` if `CrtGlitch` is also present.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct CrtSettings {
    /// Screen curvature: 0.0 = flat, ~0.1 = subtle CRT curve
    pub curvature: f32,
    /// Chromatic aberration: 0.0 = none, ~0.015 = visible
    pub chromatic_aberration: f32,
    /// Vignette at edges: 0.0 = none, ~1.0 = very dark
    pub vignette_strength: f32,
    /// Scanline darkness: 0.0 = off, 0.95 = near-black gaps
    pub scanline_strength: f32,
    /// Noise / film grain: 0.0 = clean, ~0.15 = heavy grain
    pub noise_strength: f32,
    // ── glitch state — written each frame by update_crt_glitch ──
    pub glitch_intensity: f32,
    pub glitch_seed: f32,
    /// Bitmask: bit0=horizontal_shift, bit1=rgb_split, bit2=noise, bit3=freeze
    pub glitch_flags: f32,
    // 4 × f32 padding → 12 × f32 = 48 bytes (multiple of 16, required by WebGL2)
    pub _padding: f32,
    pub _padding2: f32,
    pub _padding3: f32,
    pub _padding4: f32,
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            curvature: 0.0,
            chromatic_aberration: 0.01,
            vignette_strength: 0.0,
            scanline_strength: 0.95,
            noise_strength: 0.0,
            glitch_intensity: 0.0,
            glitch_seed: 0.0,
            glitch_flags: 0.0,
            _padding: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            _padding4: 0.0,
        }
    }
}

// ─── Glitch config (CPU only) ────────────────────────────────────────────────

/// Controls glitch timing and which effects fire. Attach alongside `CrtSettings`.
#[derive(Component)]
pub struct CrtGlitch {
    /// Minimum seconds between glitches
    pub interval_min: f32,
    /// Maximum seconds between glitches
    pub interval_max: f32,
    /// Duration of each burst in seconds
    pub duration: f32,
    /// Peak intensity multiplier (0..1)
    pub intensity: f32,
    /// Enable horizontal band displacement
    pub horizontal_shift: bool,
    /// Enable brutal per-line RGB channel split
    pub rgb_split: bool,
    /// Enable pixel noise / grain
    pub noise: bool,
    /// Enable freeze + jump effect
    pub freeze: bool,
    // internal state
    next_glitch: f32,
    glitch_end: f32,
}

impl Default for CrtGlitch {
    fn default() -> Self {
        Self {
            interval_min: 10.0,
            interval_max: 20.0,
            duration: 0.15,
            intensity: 0.65,
            horizontal_shift: true,
            rgb_split: true,
            noise: true,
            freeze: true,
            next_glitch: 10.0,
            glitch_end: 0.0,
        }
    }
}

impl CrtGlitch {
    pub fn new(intensity: f32, interval_min: f32, interval_max: f32, duration: f32) -> Self {
        Self {
            intensity,
            interval_min,
            interval_max,
            duration,
            horizontal_shift: false,
            rgb_split: false,
            noise: false,
            freeze: false,
            next_glitch: interval_min,
            glitch_end: 0.0,
        }
    }
}

pub struct CrtPlugin;

impl Default for CrtPlugin {
    fn default() -> Self {
        Self
    }
}

impl CrtPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for CrtPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/crt.wgsl");

        app.add_plugins((
            ExtractComponentPlugin::<CrtSettings>::default(),
            UniformComponentPlugin::<CrtSettings>::default(),
        ))
        .add_systems(Update, update_crt_glitch);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // Single system that loads shader and creates pipeline
        render_app.add_systems(
            RenderStartup,
            move |mut commands: Commands,
                  render_device: Res<RenderDevice>,
                  asset_server: Res<AssetServer>,
                  fullscreen_shader: Res<FullscreenShader>| {
                let layout = BindGroupLayoutDescriptor::new(
                    "crt_bind_group_layout",
                    &BindGroupLayoutEntries::sequential(
                        ShaderStages::FRAGMENT,
                        (
                            texture_2d(TextureSampleType::Float { filterable: true }),
                            sampler(SamplerBindingType::Filtering),
                            uniform_buffer::<CrtSettings>(true),
                        ),
                    ),
                );

                let sampler = render_device.create_sampler(&SamplerDescriptor::default());
                let shader = load_embedded_asset!(asset_server.as_ref(), "shaders/crt.wgsl");
                let vertex_state = fullscreen_shader.to_vertex_state();

                commands.init_resource::<SpecializedRenderPipelines<CrtPipeline>>();
                commands.insert_resource(CrtPipeline {
                    layout,
                    sampler,
                    shader,
                    vertex_state,
                });
            },
        );

        render_app.add_systems(Render, prepare_crt_pipelines.in_set(RenderSystems::Prepare));

        render_app
            .add_render_graph_node::<ViewNodeRunner<CrtNode>>(Core2d, CrtLabel)
            .add_render_graph_edge(Core2d, Node2d::EndMainPassPostProcessing, CrtLabel)
            .add_render_graph_node::<ViewNodeRunner<CrtNode>>(Core3d, CrtLabel)
            .add_render_graph_edge(Core3d, Node3d::EndMainPassPostProcessing, CrtLabel)
            .add_render_graph_edge(Core3d, CrtLabel, Node3d::Upscaling);
    }
}

// ─── Glitch system ───────────────────────────────────────────────────────────

fn update_crt_glitch(time: Res<Time>, mut query: Query<(&mut CrtSettings, &mut CrtGlitch)>) {
    let t = time.elapsed_secs();

    for (mut settings, mut glitch) in query.iter_mut() {
        // Build flag bitmask from the bool fields
        let flags = (glitch.horizontal_shift as u32)
            | ((glitch.rgb_split as u32) << 1)
            | ((glitch.noise as u32) << 2)
            | ((glitch.freeze as u32) << 3);

        if t < glitch.glitch_end {
            // ── Active burst: bell-curve envelope (0→1→0 over duration) ──
            let burst_start = glitch.glitch_end - glitch.duration;
            let progress = ((t - burst_start) / glitch.duration).clamp(0.0, 1.0);
            let envelope = (progress * std::f32::consts::PI).sin();
            settings.glitch_intensity = glitch.intensity * envelope;
            settings.glitch_seed = t * 73.1;
            settings.glitch_flags = flags as f32;
        } else if t >= glitch.next_glitch {
            // ── Start a new burst ──
            let interval: f32 =
                rand::thread_rng().gen_range(glitch.interval_min..glitch.interval_max);
            glitch.glitch_end = t + glitch.duration;
            glitch.next_glitch = glitch.glitch_end + interval;
            settings.glitch_intensity = 0.01;
            settings.glitch_seed = t;
            settings.glitch_flags = flags as f32;
        } else {
            // ── Idle: no glitch ──
            settings.glitch_intensity = 0.0;
            settings.glitch_flags = 0.0;
        }
    }
}

// ─── Render graph ────────────────────────────────────────────────────────────

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CrtLabel;

#[derive(Default)]
struct CrtNode;

impl ViewNode for CrtNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewCrtPipeline,
        &'static DynamicUniformIndex<CrtSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, view_pipeline, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let crt_pipeline = world.resource::<CrtPipeline>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(view_pipeline.0) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<CrtSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "crt_bind_group",
            &pipeline_cache.get_bind_group_layout(&crt_pipeline.layout),
            &BindGroupEntries::sequential((
                post_process.source,
                &crt_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("crt_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

// ─── Pipeline ───────────────────────────────────────────────────────────────

#[derive(Component)]
struct ViewCrtPipeline(CachedRenderPipelineId);

#[derive(Resource)]
struct CrtPipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    shader: Handle<Shader>,
    vertex_state: VertexState,
}

impl SpecializedRenderPipeline for CrtPipeline {
    type Key = TextureFormat;

    fn specialize(&self, format: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("crt_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: self.vertex_state.clone(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

fn prepare_crt_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<CrtPipeline>>,
    crt_pipeline: Res<CrtPipeline>,
    views: Query<(Entity, &ExtractedView), With<CrtSettings>>,
) {
    for (entity, view) in views.iter() {
        let format = if view.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };
        let pipeline_id = pipelines.specialize(&pipeline_cache, &crt_pipeline, format);
        commands.entity(entity).insert(ViewCrtPipeline(pipeline_id));
    }
}
