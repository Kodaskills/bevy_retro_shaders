# 🚀 Quick Setup Guide

## Prerequisites

- [GitHub CLI (gh)](https://cli.github.com/) installed
- Authenticated with GitHub: `gh auth login`
- Git installed and configured

## One-Command Setup

Run the automated setup script:

```bash
./scripts/setup-repo.sh
```

This will:
1. ✅ Create the GitHub repository
2. ✅ Configure repository settings and topics
3. ✅ Push all code and create initial release
4. ✅ Set up branch protection
5. ⚠️ Guide you through setting up CARGO_REGISTRY_TOKEN
6. ⚠️ Guide you through enabling GitHub Pages

## Manual Steps (After Script Completes)

### 1. Set crates.io Token (for automated publishing)

```bash
# Create token at: https://crates.io/settings/tokens
# Then set it as a repository secret:
gh secret set CARGO_REGISTRY_TOKEN --repo kodaskills/bevy_retro_shaders
```

### 2. Enable GitHub Pages

After the `pages` workflow completes (5-10 minutes):

1. Go to: https://github.com/kodaskills/bevy_retro_shaders/settings/pages
2. Under **Source**, select **GitHub Actions**
3. Your site will be available at: https://kodaskills.github.io/bevy_retro_shaders

### 3. Verify Workflows

Check that workflows are running:
- **CI**: https://github.com/kodaskills/bevy_retro_shaders/actions/workflows/ci.yml
- **Pages**: https://github.com/kodaskills/bevy_retro_shaders/actions/workflows/pages.yml

## Creating Releases

### Via CLI

```bash
# Update version in Cargo.toml
# Commit changes
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git push

# Create and push tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Or use gh
gh release create v0.2.0 --generate-notes
```

This will:
- ✅ Create a GitHub Release
- ✅ Build cross-platform artifacts
- ✅ Publish to crates.io (if CARGO_REGISTRY_TOKEN is set)

## Local Development

### Test Locally

```bash
# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt --all

# Run examples
cargo run --example crt_example --features "jpeg,hot_reload"
cargo run --example crt_3d_example
```

### Test Web Build Locally

```bash
# Install trunk
cargo install --locked trunk

# Build and serve locally
trunk serve --open
```

## Troubleshooting

### CI Fails on Linux

Linux needs dependencies for Bevy:
```yaml
sudo apt-get update
sudo apt-get install -y pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

This is already configured in the workflows.

### Web Build Fails

Make sure:
1. Examples are WASM-compatible (no native-only features)
2. Assets are properly referenced
3. `fit_canvas_to_parent` is set for windows

### crates.io Publish Fails

- Ensure CARGO_REGISTRY_TOKEN is set correctly
- Version in Cargo.toml must match the tag
- All dependencies must be valid

## Repository Structure

```
.github/
├── ISSUE_TEMPLATE/          # Bug reports, feature requests
├── PULL_REQUEST_TEMPLATE.md # PR checklist
├── dependabot.yml           # Automated dependency updates
├── FUNDING.yml              # Sponsorship config
└── workflows/
    ├── ci.yml               # Test, lint, build on push/PR
    ├── release.yml          # Create releases & publish to crates.io
    └── pages.yml            # Build & deploy web demos

scripts/
└── setup-repo.sh            # Automated repository setup

web/
├── index.html               # GitHub Pages landing page
└── Trunk.toml               # WASM build configuration
```

## Useful Commands

```bash
# Check workflow status
gh run list

# View workflow logs
gh run view <run-id> --log

# Create issue
gh issue create --title "Bug: ..." --body "Description..."

# Create PR
gh pr create --title "feat: ..." --body "Description..."

# Check secrets
gh secret list
```

## Support

- 📚 [Documentation](https://github.com/kodaskills/bevy_retro_shaders#readme)
- 🐛 [Report Bug](https://github.com/kodaskills/bevy_retro_shaders/issues/new?template=bug_report.md)
- 💡 [Request Feature](https://github.com/kodaskills/bevy_retro_shaders/issues/new?template=feature_request.md)

---

Happy coding! 🎉
