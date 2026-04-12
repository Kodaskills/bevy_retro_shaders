// bevy_retro_shaders - Retro post-processing shaders for Bevy
//
// Provides CRT-style effects including barrel distortion, chromatic aberration,
// scanlines, vignette, and random glitch effects.

pub mod crt;

pub use crt::{CrtGlitch, CrtLabel, CrtPlugin, CrtSettings};
