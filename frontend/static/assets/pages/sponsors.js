/**
 * PIEAS Materials Society (PMS)
 * Industry Partners & Sponsors Page Logic
 * frontend/static/assets/pages/sponsors.js
 */

(async () => {
  // 1. Load active sponsors
  const grid = document.getElementById('sponsors-grid');
  if (grid) {
    grid.innerHTML = PMS.skeletonGrid(3);
    const tierColor = { Platinum: '#e5e7eb', Gold: '#fbbf24', Silver: '#94a3b8', Bronze: '#d97706', Partner: 'var(--primary-light)' };
    try {
      const sponsors = await PMS.apiFetch('/api/sponsors');
      grid.innerHTML = sponsors.length ? sponsors.map((s, idx) => `
        <article class="card fade-up" style="text-align:center;transition-delay:${idx * 60}ms">
          ${s.logo_url
            ? `<img src="${PMS.safeUrl(s.logo_url)}" alt="${PMS.escHtml(s.name)}" style="max-height:80px;max-width:180px;margin:0 auto 1.25rem;object-fit:contain">`
            : `<div style="font-size:2.5rem;margin-bottom:.75rem">🤝</div>`}
          <h2 style="font-size:1.1rem;margin-bottom:.35rem">${PMS.escHtml(s.name)}</h2>
          <span class="badge" style="color:${tierColor[s.tier] || 'var(--text-muted)'}">${PMS.escHtml(s.tier)} Partner</span>
          ${s.since_year ? `<p style="font-size:.8rem;color:var(--text-dim);margin-top:.35rem">Partner Since ${PMS.escHtml(s.since_year)}</p>` : ''}
          ${s.website_url ? `<div style="margin-top:1rem"><a href="${PMS.safeUrl(s.website_url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm">Visit Website ↗</a></div>` : ''}
        </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No active sponsors listed.</p>';
      PMS.observeFadeElements(grid);
    } catch(e) {
      grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load sponsors.</p>';
    }
  }

  // 2. Load sponsors_pitch cards from site_sections
  const pitchGrid = document.getElementById('pitch-grid');
  if (pitchGrid) {
    try {
      const sections = await PMS.apiFetch('/api/sections');
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
            pitchGrid.innerHTML = cards.map(c => `
              <article class="card fade-up">
                <h3 style="font-size:1.1rem;margin-bottom:.5rem;color:var(--primary-light)">✨ ${PMS.escHtml(c.title)}</h3>
                <p style="color:var(--text-muted);font-size:.875rem;line-height:1.6">${PMS.escHtml(c.desc)}</p>
              </article>
            `).join('');
            PMS.observeFadeElements(pitchGrid);
            return;
          } catch (_) {}
        }
      }
      pitchGrid.innerHTML = '';
    } catch(_) {
      pitchGrid.innerHTML = '';
    }
  }
})();

