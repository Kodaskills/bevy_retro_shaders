<div align="center">

# bevy_retro_shaders

### Retro post-processing shaders for **Bevy 0.18+** — CRT curvature, scanlines, chromatic aberration, vignette, and glitch effects. Works on **Camera2d** and **Camera3d**.

[![Crates.io](https://img.shields.io/crates/v/bevy_retro_shaders?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/bevy_retro_shaders)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/Kodaskills/bevy_retro_shaders/main?style=for-the-badge)](https://github.com/Kodaskills/bevy_retro_shaders/commits/main)

### Built with:
[![Rust](https://img.shields.io/badge/Rust-2021-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-232326?style=for-the-badge)](https://bevyengine.org)
[![WGSL](https://img.shields.io/badge/WGSL-Shader-6A0DAD?style=for-the-badge)](https://gpuweb.github.io/gpuweb/wgsl/)
[![egui](https://img.shields.io/badge/egui-0.31-5B8FB9?style=for-the-badge)](https://github.com/emilk/egui)

</div>

---

## ✨ Features

| Effect | Description |
|--------|-------------|
| **Barrel Distortion** | Curved screen simulation — from flat to strong CRT bulge |
| **Scanlines** | Cosine-based horizontal line pattern with adjustable intensity |
| **Chromatic Aberration** | RGB channel offset for authentic retro color fringing |
| **Vignette** | Smooth edge darkening |
| **Glitch Bursts** | Randomized bursts with horizontal shift, RGB split, noise, and freeze |
| **Camera2d + Camera3d** | Works on both 2D and 3D pipelines — same API |
| **Embedded Shader** | No asset file required in the user project — shader is embedded in the lib binary |

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Bevy 0.18](https://bevyengine.org)

### Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_retro_shaders = "0.1"
```

Add the plugin and attach `CrtSettings` to your camera:

```rust
use bevy::prelude::*;
use bevy_retro_shaders::{CrtPlugin, CrtSettings, CrtGlitch};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CrtPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera2d
    commands.spawn((
        Camera2d,
        CrtSettings::default(),
        CrtGlitch::default(), // optional: enables random glitch bursts
    ));
}
```

Works identically on **Camera3d**:

```rust
commands.spawn((
    Camera3d::default(),
    CrtSettings { curvature: 0.06, scanline_strength: 0.9, ..default() },
    CrtGlitch::default(),
));
```

---

## 📁 Project Structure

```
bevy_retro_shaders/
├── src/
│   ├── lib.rs              # Public API re-exports
│   ├── crt.rs              # Plugin, CrtSettings, CrtGlitch, render graph
│   └── shaders/
│       └── crt.wgsl        # WGSL fragment shader (embedded in binary)
└── examples/
    ├── assets
    │   ├── images          # Images hot reloaded to test effect on sprites or background (only 2D demo)
    ├── crt_example.rs      # 2D interactive demo (images, text, bloom, tonemapping)
    └── crt_3d_example.rs   # 3D interactive demo (PBR scene, bloom, tonemapping)
```

---

## ⚙️ Configuration

### `CrtSettings`

Add to any `Camera2d` or `Camera3d` entity to enable the effect.

| Field | Default | Description |
|-------|---------|-------------|
| `curvature` | `0.0` | Barrel distortion — `0.0` = flat, `0.1` = subtle CRT curve |
| `chromatic_aberration` | `0.01` | RGB channel offset amount |
| `vignette_strength` | `0.0` | Edge darkening — `0.0` = none, `1.0` = very dark |
| `scanline_strength` | `0.95` | Scanline gap darkness — `0.0` = off, `0.95` = near-black |
| `noise_strength` | `0.0` | Film grain — `0.0` = clean, `0.15` = heavy noise |
| `glitch_intensity` | `0.0` | Auto-updated at runtime by `update_crt_glitch` |
| `glitch_seed` | `0.0` | Auto-updated at runtime |
| `glitch_flags` | `0.0` | Auto-updated at runtime (bitmask) |

### `CrtGlitch`

Attach alongside `CrtSettings` to enable automatic randomized glitch bursts.
The `update_crt_glitch` system updates `CrtSettings` glitch fields every frame.

| Field | Default | Description |
|-------|---------|-------------|
| `interval_min` | `10.0` | Minimum seconds between bursts |
| `interval_max` | `20.0` | Maximum seconds between bursts |
| `duration` | `0.15` | Duration of each burst in seconds |
| `intensity` | `0.65` | Peak intensity multiplier `(0..1)` |
| `horizontal_shift` | `true` | Horizontal band displacement |
| `rgb_split` | `true` | Brutal per-line RGB channel split |
| `noise` | `true` | Pixel noise / grain |
| `freeze` | `true` | Freeze + jump effect |

Use `CrtGlitch::new(intensity, interval_min, interval_max, duration)` to build with all effects disabled by default, then toggle each flag:

```rust
let glitch = CrtGlitch::new(0.8, 5.0, 15.0, 0.2);
// then: glitch.horizontal_shift = true; etc.
```

---

## 💡 Usage Examples

### Minimal — scanlines only

```rust
commands.spawn((
    Camera2d,
    CrtSettings {
        scanline_strength: 0.85,
        ..default()
    },
));
```

### Full retro CRT with glitches

```rust
use bevy::render::view::Hdr;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;

commands.spawn((
    Camera2d,
    Hdr,
    CrtSettings {
        curvature: 0.08,
        chromatic_aberration: 0.015,
        vignette_strength: 0.4,
        scanline_strength: 0.9,
        ..default()
    },
    CrtGlitch {
        intensity: 0.8,
        interval_min: 5.0,
        interval_max: 15.0,
        duration: 0.2,
        horizontal_shift: true,
        rgb_split: true,
        noise: true,
        freeze: false,
        ..CrtGlitch::new(0.8, 5.0, 15.0, 0.2)
    },
    Bloom::NATURAL,
    Tonemapping::TonyMcMapface,
));
```

### Pairing with Bloom

`Bloom` requires `Hdr` on the camera. The CRT effect runs **after** bloom and tonemapping, so it applies to the final composited image.

---

## 🎮 Running the Examples

```bash
# 2D interactive demo — images, text, colors, bloom, tonemapping
cargo run --example crt_example --features "jpeg,hot_reload"

# 3D interactive demo — PBR scene with rotating meshes
cargo run --example crt_3d_example
```

### Example controls

| Key | Action |
|-----|--------|
| `SPACE` | Toggle CRT effect |
| `ENTER` | Toggle Glitch |
| `B` | Toggle Bloom |
| `T` | Toggle Tonemapping |

---

## 🔧 Render Graph Position

The CRT node is inserted in both pipelines:

| Pipeline | Position |
|----------|----------|
| `Core2d` | After `EndMainPassPostProcessing`, before `NodeUi::UiPass` |
| `Core3d` | After `EndMainPassPostProcessing`, before `Upscaling` |

UI (egui, Bevy UI) is rendered **after** the CRT pass — the overlay is never affected by the effect.

---

## 📦 Features

```toml
[features]
default = ["crt"]
crt     = []                       # CRT effect (always included)
jpeg    = ["bevy/jpeg"]            # JPEG image support
hot_reload = ["bevy/file_watcher"] # Asset hot-reloading
```

---

## 🔄 Bevy Version Compatibility

| `bevy` | `bevy_retro_shaders` |
| ------ | -------------------- |
| 0.18   | 0.1                  |

---

## 📄 License

**MIT License** — See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

Built with gratitude to the [Bevy](https://bevyengine.org) community and the open-source ecosystem that makes game development in Rust possible.

---

<div align="center">

**Maintained with 🔥 by the [Kodaskills](https://github.com/Kodaskills) team**

[![Bevy](https://img.shields.io/badge/Made%20for-Bevy-232326?style=for-the-badge)](https://bevyengine.org)

</div>
