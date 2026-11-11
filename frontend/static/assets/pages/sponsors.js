/**
 * PIEAS Materials Society (PMS)
 * Industry Partners & Sponsors Page Logic
 * frontend/static/assets/pages/sponsors.js
 */

(async () => {
  // 1. Load active sponsors
  const grid = document.getElementById('sponsors-grid');
  if (grid) {
    grid.setAttribute('aria-busy', 'true');
    grid.innerHTML = PMS.skeletonGrid(3);
    const tierColor = {
      Platinum: 'var(--tier-platinum, #c9ced6)',
      Gold: 'var(--tier-gold, #fbbf24)',
      Silver: 'var(--tier-silver, #94a3b8)',
      Bronze: 'var(--tier-bronze, #d97706)',
      Partner: 'var(--accent)',
    };
    try {
      const sponsors = await PMS.apiFetch('/api/sponsors');
      grid.innerHTML = sponsors.length ? sponsors.map((s, idx) => `
        <article class="card fade-up" style="text-align:center;transition-delay:${Math.min(idx, 8) * 60}ms">
          ${s.logo_url
            ? `<img src="${PMS.safeUrl(s.logo_url)}" alt="${PMS.escHtml(s.name)}" style="max-height:80px;max-width:180px;margin:0 auto 1.25rem;object-fit:contain">`
            : `<div style="width:64px;height:64px;border-radius:50%;background:var(--bg-raised);border:1px solid var(--border);display:flex;align-items:center;justify-content:center;margin:0 auto 12px;color:var(--text-muted)">${PMS.icon('building', 24)}</div>`}
          <h3 style="font-size:1.1rem;margin-bottom:.35rem">${PMS.escHtml(s.name)}</h3>
          <span class="badge" style="color:${tierColor[s.tier] || 'var(--text-muted)'}">${PMS.escHtml(s.tier === 'Partner' ? 'Partner' : `${s.tier} Partner`)}</span>
          ${s.since_year ? `<p style="font-size:.8rem;color:var(--text-muted);margin-top:.35rem">Partner Since ${PMS.escHtml(s.since_year)}</p>` : ''}
          ${s.website_url ? `<div style="margin-top:1rem"><a href="${PMS.safeUrl(s.website_url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm" style="display:inline-flex;align-items:center;gap:6px">${PMS.icon('external', 13)} Visit Website</a></div>` : ''}
        </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No active sponsors listed.</p>';
      PMS.observeFadeElements(grid);
    } catch(e) {
      grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load sponsors.</p>';
    } finally {
      grid.setAttribute('aria-busy', 'false');
    }
  }

  // 2. Load sponsors_pitch cards from site_sections
  const pitchGrid = document.getElementById('pitch-grid');
  if (pitchGrid) {
    pitchGrid.setAttribute('aria-busy', 'true');
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
                <h3 style="font-size:15px;margin-bottom:8px;color:var(--text)">${PMS.escHtml(c.title)}</h3>
                <p style="color:var(--text-secondary);font-size:13.5px;line-height:1.55">${PMS.escHtml(c.desc)}</p>
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
    } finally {
      pitchGrid.setAttribute('aria-busy', 'false');
    }
  }
})();

