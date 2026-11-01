/**
 * PIEAS Materials Society (PMS)
 * Alumni Directory Page Logic
 * frontend/static/assets/pages/alumni.js
 */

(async () => {
  const grid = document.getElementById('alumni-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const alumni = await PMS.apiFetch('/api/alumni');
    grid.innerHTML = alumni.length ? alumni.map((a, idx) => `
      <article class="card fade-up" style="display:flex;gap:1.25rem;align-items:flex-start;transition-delay:${idx * 60}ms">
        ${a.photo_url
          ? `<img src="${PMS.safeUrl(a.photo_url)}" alt="${PMS.escHtml(a.name)}" style="width:72px;height:72px;border-radius:50%;object-fit:cover;flex-shrink:0;border:2px solid var(--primary)">`
          : `<div style="width:72px;height:72px;border-radius:50%;background:var(--gradient-card);flex-shrink:0;display:flex;align-items:center;justify-content:center;font-size:1.75rem">🎓</div>`}
        <div style="flex:1">
          <div style="display:flex;justify-content:space-between;align-items:flex-start;flex-wrap:wrap;gap:.5rem">
            <h2 style="font-size:1.05rem;margin:0">${PMS.escHtml(a.name)}</h2>
            <span class="badge" style="font-size:.72rem">Batch ${PMS.escHtml(a.graduating_batch)}</span>
          </div>
          <p style="color:var(--primary-light);font-size:.85rem;font-weight:600;margin:.25rem 0">${PMS.escHtml(a.current_role || '')} ${a.current_organization ? '· ' + PMS.escHtml(a.current_organization) : ''}</p>
          ${a.industry_sector ? `<p style="color:var(--text-dim);font-size:.78rem;margin-bottom:.35rem">🏭 Sector: ${PMS.escHtml(a.industry_sector)}</p>` : ''}
          ${a.location ? `<p style="color:var(--text-dim);font-size:.78rem;margin-bottom:.5rem">📍 ${PMS.escHtml(a.location)}</p>` : ''}
          ${a.quote_testimonial ? `<blockquote style="font-size:.825rem;color:var(--text-muted);font-style:italic;margin:.6rem 0;border-left:2px solid var(--primary);padding-left:.6rem;line-height:1.5">“${PMS.escHtml(a.quote_testimonial)}”</blockquote>` : ''}
          ${a.linkedin_url ? `<a href="${PMS.safeUrl(a.linkedin_url)}" target="_blank" rel="noopener" style="font-size:.8rem;color:var(--secondary);font-weight:500;display:inline-block;margin-top:.25rem">LinkedIn Profile →</a>` : ''}
        </div>
      </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No alumni profiles listed.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load alumni.</p>';
  }
})();

