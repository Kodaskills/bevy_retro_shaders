// Determine current page (demo or 3d)
// This function is "Head-safe" as it doesn't rely on document.body
function getPage() {
    return window.location.pathname.includes('3d.html') ? '3d' : 'demo';
}

// Initialize data layer
window.dataLayer = window.dataLayer || [];

const page = getPage();

const uiInteractionMap = {
  // Shared (both pages)
  click_changelog_footer:       { element_type: "link",   element_name: "changelog",        interaction_type: "click", element_location: "footer", page_name: page },
  click_contributing_footer:    { element_type: "link",   element_name: "contributing",     interaction_type: "click", element_location: "footer", page_name: page },
  click_cratesio_footer_badge:  { element_type: "link",   element_name: "crates_io",        interaction_type: "click", element_location: "footer", page_name: page },
  click_cratesio_footer_project:{ element_type: "link",   element_name: "crates_io",        interaction_type: "click", element_location: "footer", page_name: page },
  click_cratesio_header:        { element_type: "button", element_name: "get_crate",        interaction_type: "click", element_location: "header", page_name: page },
  click_cratesio_hero:          { element_type: "button", element_name: "crates_io",        interaction_type: "click", element_location: "hero",   page_name: page },
  click_docs_footer:            { element_type: "link",   element_name: "documentation",    interaction_type: "click", element_location: "footer", page_name: page },
  click_github:                 { element_type: "button", element_name: "github",           interaction_type: "click", element_location: "header", page_name: page },
  click_github_footer_connect:  { element_type: "link",   element_name: "github",           interaction_type: "click", element_location: "footer", page_name: page },
  click_github_footer_project:  { element_type: "link",   element_name: "github",           interaction_type: "click", element_location: "footer", page_name: page },
  click_github_hero:            { element_type: "button", element_name: "view_on_github",   interaction_type: "click", element_location: "hero",   page_name: page },
  click_issues_footer:          { element_type: "link",   element_name: "issues",           interaction_type: "click", element_location: "footer", page_name: page },
  click_license_footer:         { element_type: "link",   element_name: "license",          interaction_type: "click", element_location: "footer", page_name: page },
  click_license_footer_badge:   { element_type: "link",   element_name: "license",          interaction_type: "click", element_location: "footer", page_name: page },
  click_logo_kodaskills:        { element_type: "link",   element_name: "kodaskills_logo",  interaction_type: "click", element_location: "header", page_name: page },
  click_releases_footer:        { element_type: "link",   element_name: "releases",         interaction_type: "click", element_location: "footer", page_name: page },
  click_theme_toggle:           { element_type: "button", element_name: "theme_toggle",     interaction_type: "click", element_location: "header", page_name: page },
  click_website_footer_connect: { element_type: "link",   element_name: "website",          interaction_type: "click", element_location: "footer", page_name: page },

  // Demo‑only
  "3d_section":           { element_type: "section", element_name: "3d_section",       interaction_type: "view", element_location: "body",   page_name: "demo" },
  click_01_features:      { element_type: "link",   element_name: "01_features",       interaction_type: "click", element_location: "header", page_name: "demo" },
  click_02_controls:      { element_type: "link",   element_name: "02_controls",       interaction_type: "click", element_location: "header", page_name: "demo" },
  click_03_demo:          { element_type: "link",   element_name: "03_demo",           interaction_type: "click", element_location: "header", page_name: "demo" },
  click_04_demo_3d:       { element_type: "link",   element_name: "04_demo_3d",        interaction_type: "click", element_location: "header", page_name: "demo" },
  click_github_hero_meta: { element_type: "link",   element_name: "GitHub",            interaction_type: "click", element_location: "hero",   page_name: "demo" },
  click_try_demo:         { element_type: "button", element_name: "try_demo",          interaction_type: "click", element_location: "header", page_name: "demo" },
  click_try_3d_demo:      { element_type: "button", element_name: "try_3d_demo",       interaction_type: "click", element_location: "body",   page_name: "demo" },
  controls_section:       { element_type: "section", element_name: "controls_section", interaction_type: "view", element_location: "body",   page_name: "demo" },
  demo_section:           { element_type: "section", element_name: "demo_section",     interaction_type: "view", element_location: "body",   page_name: "demo" },
  features_section:       { element_type: "section", element_name: "features_section", interaction_type: "view", element_location: "body",   page_name: "demo" },

  // 3D‑only
  click_back_to_home:     { element_type: "link",   element_name: "back_to_home",      interaction_type: "click", element_location: "hero",   page_name: "3d" }
};

function pushUiInteraction(key) {
  const params = uiInteractionMap[key];
  if (!params) return;
  window.dataLayer.push({
    event: 'ui_interaction',
    ...params
  });
}

function getThemeState() {
  const isDark = document.documentElement.classList.contains('dark');
  if (isDark) return 'dark';
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  return prefersDark ? 'dark' : 'light';
}

function pushSystemEvent(systemAction, systemState) {
  window.dataLayer.push({
    event: 'system_event',
    system_action: systemAction,
    page_name: page,
    system_state: systemState
  });
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  // On page load
  const currentTheme = getThemeState();
  if (page === '3d') {
    pushSystemEvent('page_load', currentTheme === 'dark' ? 'init_dark' : 'init_light');
  } else {
    pushSystemEvent('page_load', currentTheme);
  }

  // Click tracking
  document.addEventListener('click', (e) => {
    const trackedEl = e.target.closest('[data-analytics]');
    if (!trackedEl) return;
    pushUiInteraction(trackedEl.getAttribute('data-analytics'));
  });

  // Section view tracking (demo only)
  if (page === 'demo') {
    const viewedSections = new Set();
    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        const key = entry.target.dataset.analyticsSection;
        if (!key || viewedSections.has(key)) return;
        viewedSections.add(key);
        pushUiInteraction(key);
      });
    }, { threshold: 0.3 });

    document.querySelectorAll('[data-analytics-section]').forEach((el) => observer.observe(el));
  }
});

// Listen for analytics CustomEvents from WASM (CrtState changes via egui)
// Always active, even before DOMContentLoaded
window.addEventListener("analytics_event", (e) => {
  if (!e.detail) return;
  window.dataLayer.push({ ...e.detail, _clear: true });
  if (typeof __DEV__ !== 'undefined') {
    console.debug('[analytics]', e.detail);
  }
});
