/**
 * PIEAS Materials Society (PMS)
 * Industry Partners & Sponsors Page Logic
 * frontend/static/assets/pages/sponsors.js
 */

(async () => {
  'use strict';

  // 1. Load active sponsors
  const grid = document.getElementById('sponsors-grid');
  if (grid) {
    grid.setAttribute('aria-busy', 'true');
    grid.innerHTML = PMS.skeletonGrid(3);
    try {
      const sponsors = await PMS.apiFetch('/api/sponsors');
      grid.innerHTML = sponsors.length
        ? sponsors.map((s, idx) => PMS.sponsorCard(s, idx)).join('')
        : PMS.emptyState('No active sponsors listed.');
      PMS.observeFadeElements(grid);
    } catch (err) {
      PMS.reportError('sponsors', err);
      grid.innerHTML = PMS.errorState('Could not load sponsors.');
    } finally {
      grid.setAttribute('aria-busy', 'false');
    }
  }

  // 2. Load sponsors_pitch cards from site_sections
  const pitchGrid = document.getElementById('pitch-grid');
  if (pitchGrid) {
    pitchGrid.setAttribute('aria-busy', 'true');
    try {
      const sections = await PMS.apiFetchCached('/api/sections');
      const sec = sections.find(s => s.section_key === 'sponsors_pitch');
      if (sec) {
        if (sec.badge) {
          const b = document.getElementById('pitch-badge');
          if (b) b.textContent = sec.badge;
        }
        if (sec.title) {
          const t = document.getElementById('pitch-title');
          if (t) t.textContent = sec.title;
        }
        if (sec.subtitle) {
          const subEl = document.getElementById('pitch-subtitle');
          if (subEl) {
            subEl.textContent = sec.subtitle;
            subEl.style.display = 'block';
          }
        }
        if (sec.content) {
          const d = document.getElementById('pitch-desc');
          if (d) d.textContent = sec.content;
        }

        if (sec.extra_json) {
          try {
            const cards = JSON.parse(sec.extra_json);
            pitchGrid.innerHTML = cards
              .map(
                c => `
              <article class="card fade-up">
                <h3 class="card__title card__title--sm">${PMS.escHtml(c.title)}</h3>
                <p class="card__text card__text--sm">${PMS.escHtml(c.desc)}</p>
              </article>
            `
              )
              .join('');
            PMS.observeFadeElements(pitchGrid);
            return;
          } catch (err) {
            PMS.reportError('sponsors-pitch-json', err);
          }
        }
      }
      pitchGrid.innerHTML = PMS.emptyState('Partnership details will be published soon.');
    } catch (err) {
      PMS.reportError('sponsors-pitch', err);
      pitchGrid.innerHTML = PMS.emptyState('Partnership details will be published soon.');
    } finally {
      pitchGrid.setAttribute('aria-busy', 'false');
    }
  }
})();
