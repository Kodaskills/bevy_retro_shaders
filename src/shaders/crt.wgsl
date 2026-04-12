// CRT post-processing shader
// Effects: barrel distortion, chromatic aberration, vignette, scanlines, glitch

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: CrtSettings;

struct CrtSettings {
    curvature: f32,
    chromatic_aberration: f32,
    vignette_strength: f32,
    scanline_strength: f32,
    noise_strength: f32,
    glitch_intensity: f32,
    glitch_seed: f32,
    glitch_flags: f32,
    // 4 × f32 padding → 12 × f32 = 48 bytes (multiple of 16, required by WebGL2)
    _padding: f32,
    _padding2: f32,
    _padding3: f32,
    _padding4: f32,
}

fn barrel_distort(uv: vec2<f32>, curvature: f32) -> vec2<f32> {
    var p = uv * 2.0 - 1.0;
    let r2 = dot(p, p);
    p = p * (1.0 + curvature * r2);
    return p * 0.5 + 0.5;
}

fn in_bounds(uv: vec2<f32>) -> f32 {
    let below = step(vec2(0.0), uv);
    let above = step(uv, vec2(1.0));
    return below.x * below.y * above.x * above.y;
}

fn hash(n: f32) -> f32 {
    let x = sin(n) * 43758.5453;
    return x - floor(x);
}

fn hash2(p: vec2<f32>) -> f32 {
    return hash(dot(p, vec2(127.1, 311.7)));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let gi = settings.glitch_intensity;

    let flags = u32(settings.glitch_flags);
    let flag_shift = f32((flags >> 0u) & 1u);
    let flag_rgb = f32((flags >> 1u) & 1u);
    let flag_noise = f32((flags >> 2u) & 1u);
    let flag_freeze = f32((flags >> 3u) & 1u);

    let distorted_uv = barrel_distort(uv, settings.curvature);
    var sample_uv = distorted_uv;

    let freeze_band = hash(floor(in.position.y / 40.0) + settings.glitch_seed * 17.0);
    let is_frozen = step(1.0 - gi * 0.5, freeze_band) * flag_freeze;
    let frozen_y = fract(settings.glitch_seed * 3.7);
    sample_uv.y = mix(sample_uv.y, frozen_y, is_frozen);

    let band = floor(in.position.y / 6.0);
    let band_hash = hash(band * 7.3 + settings.glitch_seed * 100.0);
    let should_shift = step(1.0 - gi * 0.65, band_hash) * flag_shift;
    let shift_amount = (hash(band + settings.glitch_seed * 31.0) - 0.5) * 0.09 * gi;
    sample_uv.x += shift_amount * should_shift;

    let aberr = settings.chromatic_aberration;
    let uv_center = sample_uv - 0.5;
    let line_hash = hash(floor(in.position.y) * 1.3 + settings.glitch_seed * 200.0);
    let rgb_glitch = step(1.0 - gi * 0.45, line_hash) * flag_rgb;
    let glitch_split = gi * 0.04 * rgb_glitch;
    let uv_r = sample_uv + uv_center * aberr + vec2(glitch_split, 0.0);
    let uv_b = sample_uv - uv_center * aberr - vec2(glitch_split, 0.0);

    let r = textureSample(screen_texture, screen_sampler, uv_r).r;
    let g = textureSample(screen_texture, screen_sampler, sample_uv).g;
    let b = textureSample(screen_texture, screen_sampler, uv_b).b;
    let a = textureSample(screen_texture, screen_sampler, sample_uv).a;

    var color = vec4(r, g, b, a);
    color = color * in_bounds(distorted_uv);

    // Noise / film grain — always active, intensity set by noise_strength
    let pixel_hash = hash2(floor(in.position.xy) + fract(settings.glitch_seed * 500.0) * 999.0);
    let noise_val = (pixel_hash - 0.5) * 2.0; // -1.0 to 1.0
    let glitch_boost = 0.15 * gi * flag_noise;
    let grain_amount = settings.noise_strength + glitch_boost;
    color = vec4(color.rgb + noise_val * grain_amount, color.a);

    let angle = (in.position.y / 4.0) * 2.0 * 3.14159265;
    let wave = cos(angle) * 0.5 + 0.5;
    let scan = 1.0 - settings.scanline_strength * (1.0 - wave * wave * wave);
    color = vec4(color.rgb * clamp(scan, 0.0, 1.0), color.a);

    let vign_uv = uv * 2.0 - 1.0;
    let vignette = 1.0 - dot(vign_uv * vign_uv, vign_uv * vign_uv) * settings.vignette_strength;
    color = vec4(color.rgb * clamp(vignette, 0.0, 1.0), color.a);

    return color;
}
