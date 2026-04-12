document.addEventListener('DOMContentLoaded', () => {
    // Loader: reads data-bevy-event from <body> to know which WASM ready event to listen for
    window.addEventListener('TrunkApplicationStarted', () => {
        const status = document.getElementById('loader-status');
        if (status) status.textContent = 'Compiling shaders...';
    });

    const bevyEvent = document.body.dataset.bevyEvent;
    if (bevyEvent) {
        window.addEventListener(bevyEvent, () => {
            const loader = document.getElementById('wasm-loader');
            if (loader) {
                loader.classList.add('hidden');
                setTimeout(() => { loader.style.display = 'none'; }, 500);
            }
        });
    }

    // Theme toggle
    const themeToggle = document.getElementById('themeToggle');
    const moonIcon = document.getElementById('moonIcon');
    const sunIcon = document.getElementById('sunIcon');
    const html = document.documentElement;

    function updateIcons(isDark) {
        moonIcon.style.display = isDark ? 'none' : 'block';
        sunIcon.style.display = isDark ? 'block' : 'none';
    }

    function initTheme() {
        const mode = localStorage.getItem('theme-mode') || 'dark';
        const isDark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
        html.className = isDark ? 'dark' : 'light';
        updateIcons(isDark);
    }

    themeToggle.addEventListener('click', () => {
        const isDark = html.classList.contains('dark');
        const newMode = isDark ? 'light' : 'dark';
        html.className = newMode === 'dark' ? 'dark' : 'light';
        localStorage.setItem('theme-mode', newMode);
        updateIcons(!isDark);
    });

    initTheme();

    // Header scroll shadow
    const header = document.querySelector('.header');
    let ticking = false;

    function updateActiveNav() {
        if (!navLinks.length) return;
        const scrollY = window.scrollY + 150;
        navLinks.forEach(link => link.classList.remove('active'));
        sections.forEach(section => {
            const sectionTop = section.offsetTop;
            const sectionId = section.getAttribute('id');
            if (scrollY >= sectionTop && scrollY < sectionTop + section.offsetHeight && sectionId) {
                navLinks.forEach(link => {
                    if (link.getAttribute('href') === `#${sectionId}`) link.classList.add('active');
                });
            }
        });
    }

    window.addEventListener('scroll', () => {
        if (!ticking) {
            window.requestAnimationFrame(() => {
                if (window.scrollY > 20) {
                    header.classList.add('scrolled');
                } else {
                    header.classList.remove('scrolled');
                }
                updateActiveNav();
                ticking = false;
            });
            ticking = true;
        }
    });

    // Active nav (only runs when anchor nav links exist)
    const sections = document.querySelectorAll('section[id]');
    const navLinks = document.querySelectorAll('.nav-link[href^="#"]');

    // Smooth scroll for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        });
    });

    // Hide loading message on load
    window.addEventListener('load', () => {
        setTimeout(() => {
            const loading = document.getElementById('loading-message');
            if (loading) loading.style.display = 'none';
        }, 2000);
    });
});
