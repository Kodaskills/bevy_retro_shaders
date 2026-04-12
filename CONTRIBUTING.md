# Contributing to bevy_retro_shaders

Thank you for your interest in contributing! This guide will help you get started.

## 🎯 Quick Start

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy -- -D warnings` and `cargo fmt --check`
6. Commit with [conventional commits](https://www.conventionalcommits.org/): `git commit -m "feat: add new effect"`
7. Push and submit a Pull Request

## 📋 Pull Request Guidelines

### Before Submitting
- [ ] Tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] Documentation is updated (if applicable)
- [ ] Examples still compile and run

### PR Description
- Clearly describe what changed and why
- Reference related issues with `Fixes #123`
- Include screenshots/GIFs for visual changes
- List any breaking changes

## 🐛 Reporting Bugs

Use the [bug report template](https://github.com/kodaskills/bevy_retro_shaders/issues/new?template=bug_report.md) and include:

- **Bevy version**: Which version are you using?
- **OS**: Windows, macOS, Linux?
- **Minimal reproduction**: Code snippet or example
- **Expected vs actual behavior**

## 💡 Feature Requests

Use the [feature request template](https://github.com/kodaskills/bevy_retro_shaders/issues/new?template=feature_request.md) to suggest improvements.

## 🎨 Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Write doc comments for public APIs
- Keep functions focused and small

## 📝 Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new chromatic aberration mode
fix: correct barrel distortion calculation
docs: update README with 3D camera example
test: add integration tests for CRT settings
chore: update dependencies
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test crt_settings

# Run with all features
cargo test --all-features
```

## 📦 Adding New Effects

1. Add shader code in `src/shaders/`
2. Create Rust configuration in `src/crt.rs`
3. Update `CrtSettings` struct
4. Add to examples and documentation
5. Write tests for new configuration options

## 🤝 Community

- Be respectful and inclusive
- Help others when you can
- Ask questions if unsure

## ❓ Questions?

Open an issue with the "question" label or reach out via discussions.

---

Thank you for contributing! 🎉
