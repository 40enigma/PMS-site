/**
 * PIEAS Materials Society (PMS)
 * About Page Logic
 * frontend/static/assets/pages/about.js
 */

(async () => {
  // 1. Load Dynamic About Sections
  const container = document.getElementById('sections-content');
  if (container) {
    container.innerHTML = PMS.skeletonGrid(3);
    try {
      const secs = await PMS.apiFetch('/api/sections');
      const aboutSecs = secs.filter(s => s.section_key.startsWith('about'));
      container.innerHTML = aboutSecs.length
        ? aboutSecs.map((s, idx) => `
          <article class="card fade-up" style="transition-delay:${Math.min(idx, 8) * 60}ms">
            <span class="badge" style="margin-bottom:.75rem">${PMS.escHtml(s.badge || s.section_key)}</span>
            <h2 style="font-size:1.3rem;margin-bottom:.5rem">${PMS.escHtml(s.title)}</h2>
            ${s.subtitle ? `<p style="color:var(--accent);font-weight:600;margin-bottom:.75rem">${PMS.escHtml(s.subtitle)}</p>` : ''}
            <p style="color:var(--text-muted);line-height:1.7">${PMS.escHtml(s.content)}</p>
          </article>`).join('')
        : '<p style="color:var(--text-muted);grid-column:1/-1">About information not available.</p>';
      PMS.observeFadeElements(container);
    } catch (e) {
      container.innerHTML = '<p style="color:var(--danger);grid-column:1/-1">Could not load about sections.</p>';
    }
  }

  // 2. Load FAQs Section
  const faqsEl = document.getElementById('faqs-container');
  if (faqsEl) {
    try {
      const faqs = await PMS.apiFetch('/api/faqs');
      faqsEl.innerHTML = faqs.length
        ? faqs.map(f => `
          <details class="card" style="margin-bottom:1rem;cursor:pointer">
            <summary style="font-weight:500;font-size:15px;color:var(--text);display:flex;justify-content:space-between;align-items:center">
              <span>${PMS.escHtml(f.question)}</span>
              <span class="badge" style="font-size:12px">${PMS.escHtml(f.category || 'General')}</span>
            </summary>
            <div style="padding-top:12px;margin-top:10px;border-top:1px solid var(--border-subtle);color:var(--text-secondary);line-height:1.6">
              ${PMS.escHtml(f.answer)}
            </div>
          </details>
        `).join('')
        : '<p style="color:var(--text-muted);text-align:center">No frequently asked questions published yet.</p>';
    } catch (e) {
      faqsEl.innerHTML = '<p style="color:var(--danger);text-align:center">Failed to load FAQs.</p>';
    }
  }
})();

