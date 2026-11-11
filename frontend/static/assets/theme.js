/**
 * PIEAS Materials Society (PMS) — Theme Initialization
 * Executes synchronously in <head> to prevent Flash of Unstyled Content (FOUC).
 * Strict CSP compliant (loaded via script-src 'self').
 */
(function () {
  document.documentElement.classList.add('js');
  try {
    var saved = localStorage.getItem('pms_theme');
    var theme = saved || (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark');
    document.documentElement.setAttribute('data-theme', theme);
  } catch (_) {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
})();

