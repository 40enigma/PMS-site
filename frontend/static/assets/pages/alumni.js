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
          ? `<img src="${PMS.safeUrl(a.photo_url)}" alt="${PMS.escHtml(a.name)}" style="width:72px;height:72px;border-radius:50%;object-fit:cover;flex-shrink:0;border:1px solid var(--border)">`
          : `<div style="width:72px;height:72px;border-radius:50%;background:var(--bg-raised);border:1px solid var(--border);flex-shrink:0;display:flex;align-items:center;justify-content:center;color:var(--text-muted)">${PMS.icon('graduation-cap', 26)}</div>`}
        <div style="flex:1">
          <div style="display:flex;justify-content:space-between;align-items:flex-start;flex-wrap:wrap;gap:.5rem">
            <h2 style="font-size:1.05rem;margin:0">${PMS.escHtml(a.name)}</h2>
            <span class="badge" style="font-size:12px">Batch ${PMS.escHtml(a.graduating_batch)}</span>
          </div>
          <p style="color:var(--text);font-size:.875rem;font-weight:500;margin:.25rem 0">${PMS.escHtml(a.current_role || '')} ${a.current_organization ? '· ' + PMS.escHtml(a.current_organization) : ''}</p>
          ${a.industry_sector ? `<p style="color:var(--text-muted);font-size:12.5px;margin-bottom:4px;display:flex;align-items:center;gap:6px">${PMS.icon('building', 13)} Sector: ${PMS.escHtml(a.industry_sector)}</p>` : ''}
          ${a.location ? `<p style="color:var(--text-muted);font-size:12.5px;margin-bottom:6px;display:flex;align-items:center;gap:6px">${PMS.icon('map-pin', 13)} ${PMS.escHtml(a.location)}</p>` : ''}
          ${a.quote_testimonial ? `<blockquote style="font-size:13px;color:var(--text-secondary);font-style:italic;margin:8px 0;border-left:2px solid var(--border-hover);padding-left:10px;line-height:1.5">“${PMS.escHtml(a.quote_testimonial)}”</blockquote>` : ''}
          ${a.linkedin_url ? `<a href="${PMS.safeUrl(a.linkedin_url)}" target="_blank" rel="noopener" style="font-size:13px;color:var(--text-secondary);display:inline-flex;align-items:center;gap:5px;margin-top:6px">${PMS.icon('external', 13)} LinkedIn Profile</a>` : ''}
        </div>
      </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No alumni profiles listed.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load alumni.</p>';
  }
})();

