use bevy_retro_shaders::{CrtGlitch, CrtSettings};

#[test]
fn test_crt_settings_default_values() {
    let settings = CrtSettings::default();

    assert_eq!(settings.curvature, 0.0, "curvature should be 0.0");
    assert_eq!(
        settings.chromatic_aberration, 0.01,
        "chromatic_aberration should be 0.01"
    );
    assert_eq!(
        settings.vignette_strength, 0.0,
        "vignette_strength should be 0.0"
    );
    assert_eq!(
        settings.scanline_strength, 0.95,
        "scanline_strength should be 0.95"
    );
    assert_eq!(
        settings.glitch_intensity, 0.0,
        "glitch_intensity should be 0.0"
    );
    assert_eq!(settings.glitch_seed, 0.0, "glitch_seed should be 0.0");
    assert_eq!(settings.glitch_flags, 0.0, "glitch_flags should be 0.0");
    assert_eq!(settings._padding, 0.0, "_padding should be 0.0");
}

#[test]
fn test_crt_settings_default_is_noop_effect() {
    let settings = CrtSettings::default();

    assert!(
        settings.curvature == 0.0,
        "default should have no curvature"
    );
    assert!(
        settings.vignette_strength == 0.0,
        "default should have no vignette"
    );
    assert!(
        settings.glitch_intensity == 0.0,
        "default should have no glitch"
    );
    assert!(
        settings.glitch_flags == 0.0,
        "default should have no glitch flags"
    );
}

#[test]
fn test_crt_glitch_default_values() {
    let glitch = CrtGlitch::default();

    assert_eq!(glitch.interval_min, 10.0, "interval_min should be 10.0");
    assert_eq!(glitch.interval_max, 20.0, "interval_max should be 20.0");
    assert_eq!(glitch.duration, 0.15, "duration should be 0.15");
    assert_eq!(glitch.intensity, 0.65, "intensity should be 0.65");
    assert!(glitch.horizontal_shift, "horizontal_shift should be true");
    assert!(glitch.rgb_split, "rgb_split should be true");
    assert!(glitch.noise, "noise should be true");
    assert!(glitch.freeze, "freeze should be true");
}

#[test]
fn test_crt_glitch_default_all_effects_enabled() {
    let glitch = CrtGlitch::default();

    assert!(
        glitch.horizontal_shift,
        "default should enable horizontal_shift"
    );
    assert!(glitch.rgb_split, "default should enable rgb_split");
    assert!(glitch.noise, "default should enable noise");
    assert!(glitch.freeze, "default should enable freeze");
}

#[test]
fn test_crt_glitch_default_initial_timing() {
    let glitch = CrtGlitch::default();

    assert!(
        glitch.interval_min <= glitch.interval_max,
        "interval_min should be <= interval_max"
    );
}
