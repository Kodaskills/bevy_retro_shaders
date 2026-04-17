#!/bin/bash
# GitHub Repository Setup Script
# Uses gh CLI to create and configure the repository automatically

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_NAME="bevy_retro_shaders"
REPO_OWNER="kodaskills"
REPO_DESCRIPTION="Retro post-processing shaders for Bevy 0.18+ — CRT curvature, scanlines, chromatic aberration, vignette, and glitch effects"
REPO_VISIBILITY="public"
REPO_HOMEPAGE="https://kodaskills.github.io/bevy_retro_shaders"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   GitHub Repository Setup for Bevy Retro Shaders        ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ gh CLI is not installed${NC}"
    echo ""
    echo "Please install it first:"
    echo "  macOS: brew install gh"
    echo "  Linux: See https://github.com/cli/cli#installation"
    echo "  Windows: winget install GitHub.cli"
    exit 1
fi

# Check if already authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}⚠️  Not authenticated with GitHub${NC}"
    echo "Please authenticate..."
    gh auth login
fi

echo -e "${GREEN}✅ gh CLI is installed and authenticated${NC}"
echo ""

# Check if repository already exists
if gh repo view "${REPO_OWNER}/${REPO_NAME}" &> /dev/null; then
    echo -e "${YELLOW}⚠️  Repository ${REPO_OWNER}/${REPO_NAME} already exists!${NC}"
    read -p "Do you want to continue? This may overwrite settings (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Setup cancelled.${NC}"
        exit 0
    fi
else
    # Create the repository
    echo -e "${BLUE}📦 Creating GitHub repository...${NC}"
    gh repo create \
        "${REPO_OWNER}/${REPO_NAME}" \
        --public \
        --description "${REPO_DESCRIPTION}" \
        --homepage "${REPO_HOMEPAGE}" \
        --confirm

    echo -e "${GREEN}✅ Repository created: https://github.com/${REPO_OWNER}/${REPO_NAME}${NC}"
    echo ""
fi

# Set the remote
echo -e "${BLUE}🔗 Setting up git remote...${NC}"
if git remote | grep -q "origin"; then
    echo -e "${YELLOW}⚠️  Remote 'origin' already exists, updating URL...${NC}"
    gh repo set-default "${REPO_OWNER}/${REPO_NAME}"
else
    gh repo set-default "${REPO_OWNER}/${REPO_NAME}"
fi
echo -e "${GREEN}✅ Remote configured${NC}"
echo ""

# Add repository topics
echo -e "${BLUE}🏷️  Adding repository topics...${NC}"
gh repo edit "${REPO_OWNER}/${REPO_NAME}" \
    --add-topic "bevy" \
    --add-topic "rust" \
    --add-topic "shaders" \
    --add-topic "crt" \
    --add-topic "retro" \
    --add-topic "wgsl" \
    --add-topic "post-processing" \
    --add-topic "game-development" \
    --add-topic "graphics" \
    --add-topic "webassembly"

echo -e "${GREEN}✅ Topics added${NC}"
echo ""

# Enable GitHub features
echo -e "${BLUE}⚙️  Configuring repository settings...${NC}"

# Enable issues
gh repo edit "${REPO_OWNER}/${REPO_NAME}" --enable-issues

# Enable wiki (optional, disable if not needed)
gh repo edit "${REPO_OWNER}/${REPO_NAME}" --enable-wiki

echo -e "${GREEN}✅ Repository features enabled${NC}"
echo ""

# Create initial commit and push
echo -e "${BLUE}📤 Pushing to GitHub...${NC}"

# Check if there are changes to commit
if [[ -n "$(git status --porcelain)" ]]; then
    echo -e "${BLUE}📝 Creating initial commit...${NC}"
    git add -A
    git commit -m "chore: initial setup with open-source configuration

- Add LICENSE, CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md
- Add CHANGELOG.md
- Add GitHub Actions workflows (CI, release, pages)
- Add issue and PR templates
- Add dependabot configuration
- Add web build infrastructure
- Configure for GitHub Pages deployment"
fi

# Push to main branch
git branch -M main
git push -u origin main --force

echo -e "${GREEN}✅ Code pushed to GitHub${NC}"
echo ""

# Create initial git tag if version is 0.1.0
echo -e "${BLUE}🏷️  Creating initial release tag...${NC}"
if ! git tag | grep -q "v0.1.0"; then
    git tag -a v0.1.0 -m "Initial release: CRT post-processing shaders for Bevy"
    git push origin v0.1.0
    echo -e "${GREEN}✅ Tag v0.1.0 created and pushed${NC}"
else
    echo -e "${YELLOW}⚠️  Tag v0.1.0 already exists${NC}"
fi
echo ""

# Setup CARGO_REGISTRY_TOKEN for automated publishing
echo -e "${YELLOW}⚠️  Action Required: Set up CARGO_REGISTRY_TOKEN${NC}"
echo ""
echo "To enable automated crates.io publishing, you need to:"
echo "1. Go to https://crates.io/settings/tokens"
echo "2. Create a new token with 'publish-new' scope"
echo "3. Run: gh secret set CARGO_REGISTRY_TOKEN --repo ${REPO_OWNER}/${REPO_NAME}"
echo ""
read -p "Do you want to set CARGO_REGISTRY_TOKEN now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Enter your crates.io token:"
    read -s CARGO_TOKEN
    echo
    echo "$CARGO_TOKEN" | gh secret set CARGO_REGISTRY_TOKEN --repo "${REPO_OWNER}/${REPO_NAME}"
    echo -e "${GREEN}✅ CARGO_REGISTRY_TOKEN set${NC}"
else
    echo -e "${YELLOW}⏭️  Skipping CARGO_REGISTRY_TOKEN setup${NC}"
    echo "You can set it later with:"
    echo "  gh secret set CARGO_REGISTRY_TOKEN --repo ${REPO_OWNER}/${REPO_NAME}"
fi
echo ""

# Enable GitHub Pages
echo -e "${BLUE}🌐 Configuring GitHub Pages...${NC}"
echo ""
echo -e "${YELLOW}⚠️  Manual Step Required${NC}"
echo ""
echo "GitHub Pages will be automatically deployed by the CI workflow."
echo "After the pages workflow completes (5-10 minutes):"
echo ""
echo "1. Go to: https://github.com/${REPO_OWNER}/${REPO_NAME}/settings/pages"
echo "2. Under 'Source', select 'GitHub Actions'"
echo "3. Your site will be available at: https://${REPO_OWNER}.github.io/${REPO_NAME}"
echo ""

# Add branch protection rules via GitHub API
echo -e "${BLUE}🔒 Setting up branch protection...${NC}"
gh api \
    --method PUT \
    "/repos/${REPO_OWNER}/${REPO_NAME}/branches/main/protection" \
    -f required_status_checks='{"strict":true,"contexts":["Tests (ubuntu-latest)","Tests (macos-latest)","Tests (windows-latest)","Clippy Lint","Code Formatting"]}' \
    -f enforce_admins=true \
    -f required_pull_request_reviews='{"required_approving_review_count":0,"dismiss_stale_reviews":true}' \
    -f restrictions=null 2>/dev/null || {
    echo -e "${YELLOW}⚠️  Branch protection requires GitHub Pro or Organization${NC}"
    echo "You can set it manually in: Repository Settings → Branches → Add rule"
}
echo ""

# Summary
echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          Repository Setup Complete! 🎉                  ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📋 Summary:${NC}"
echo ""
echo -e "  Repository:   ${GREEN}https://github.com/${REPO_OWNER}/${REPO_NAME}${NC}"
echo -e "  Issues:       ${GREEN}https://github.com/${REPO_OWNER}/${REPO_NAME}/issues${NC}"
echo -e "  Actions:      ${GREEN}https://github.com/${REPO_OWNER}/${REPO_NAME}/actions${NC}"
echo -e "  Releases:     ${GREEN}https://github.com/${REPO_OWNER}/${REPO_NAME}/releases${NC}"
echo ""
echo -e "${BLUE}🚀 Next Steps:${NC}"
echo ""
echo "  1. ${YELLOW}Watch the CI/CD workflows:${NC}"
echo "     https://github.com/${REPO_OWNER}/${REPO_NAME}/actions"
echo ""
echo "  2. ${YELLOW}Set up CARGO_REGISTRY_TOKEN${NC} (if not done)"
echo "     gh secret set CARGO_REGISTRY_TOKEN --repo ${REPO_OWNER}/${REPO_NAME}"
echo ""
echo "  3. ${YELLOW}Enable GitHub Pages${NC} (after pages workflow completes)"
echo "     https://github.com/${REPO_OWNER}/${REPO_NAME}/settings/pages"
echo ""
echo "  4. ${YELLOW}Test the web demo${NC} (after pages deployment)"
echo "     https://${REPO_OWNER}.github.io/${REPO_NAME}"
echo ""
echo "  5. ${YELLOW}Create a release${NC} to trigger crates.io publishing"
echo "     gh release create v0.2.0 --generate-notes"
echo ""
echo -e "${BLUE}📚 Documentation:${NC}"
echo ""
echo "  - README.md: Project overview"
echo "  - CONTRIBUTING.md: Contribution guidelines"
echo "  - CODE_OF_CONDUCT.md: Community standards"
echo "  - SECURITY.md: Security policy"
echo "  - CHANGELOG.md: Version history"
echo ""
echo -e "${GREEN}✅ All done! Your repository is ready for open-source development! 🎉${NC}"
echo ""
