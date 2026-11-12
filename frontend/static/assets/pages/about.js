/**
 * PIEAS Materials Society (PMS)
 * About Page Logic
 * frontend/static/assets/pages/about.js
 */

(async () => {
  'use strict';

  // 1. Load Dynamic About Sections
  const container = document.getElementById('sections-content');
  if (container) {
    container.setAttribute('aria-busy', 'true');
    container.innerHTML = PMS.skeletonGrid(3);
    try {
      const secs = await PMS.apiFetchCached('/api/sections');
      const aboutSecs = secs.filter(s => s.section_key.startsWith('about'));
      container.innerHTML = aboutSecs.length
        ? aboutSecs
            .map(
              (s, idx) => `
          <article class="card fade-up" style="${PMS.stagger(idx)}">
            <span class="badge" style="margin-bottom:var(--sp-3)">${PMS.escHtml(s.badge || s.section_key)}</span>
            <h2 class="card__title card__title--sm">${PMS.escHtml(s.title)}</h2>
            ${s.subtitle ? `<p class="card__meta card__meta--strong" style="color:var(--accent)">${PMS.escHtml(s.subtitle)}</p>` : ''}
            <p class="card__text">${PMS.escHtml(s.content)}</p>
          </article>`
            )
            .join('')
        : PMS.emptyState('About information is not available yet.');
      PMS.observeFadeElements(container);
    } catch (err) {
      PMS.reportError('about-sections', err);
      container.innerHTML = PMS.errorState('Could not load about sections.');
    } finally {
      container.setAttribute('aria-busy', 'false');
    }
  }

  // 2. Load FAQs Section
  const faqsEl = document.getElementById('faqs-container');
  if (faqsEl) {
    faqsEl.setAttribute('aria-busy', 'true');
    try {
      const faqs = await PMS.apiFetch('/api/faqs');
      faqsEl.innerHTML = faqs.length
        ? faqs
            .map(
              f => `
          <details class="card" style="margin-bottom:var(--sp-4);cursor:pointer">
            <summary>
              <h3 class="card__title card__title--xs" style="margin:0">${PMS.escHtml(f.question)}</h3>
              <span class="badge">${PMS.escHtml(f.category || 'General')}</span>
            </summary>
            <div class="card__text" style="padding-top:var(--sp-3);margin-top:var(--sp-2);border-top:1px solid var(--border-subtle);margin-bottom:0">
              ${PMS.escHtml(f.answer)}
            </div>
          </details>
        `
            )
            .join('')
        : PMS.emptyState('No frequently asked questions published yet.');
    } catch (err) {
      PMS.reportError('faqs', err);
      faqsEl.innerHTML = PMS.errorState('Could not load FAQs.');
    } finally {
      faqsEl.setAttribute('aria-busy', 'false');
    }
  }
})();
