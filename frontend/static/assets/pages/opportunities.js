/**
 * PIEAS Materials Society (PMS)
 * Opportunities & Careers Page Logic
 * frontend/static/assets/pages/opportunities.js
 */

(async () => {
  const grid = document.getElementById('opps-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const opps = await PMS.apiFetch('/api/opportunities');
    grid.innerHTML = opps.length ? opps.map((o, idx) => `
      <article class="card fade-up" style="transition-delay:${idx * 60}ms">
        <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
          <span class="badge">${PMS.escHtml(o.opportunity_type)}</span>
          ${o.deadline ? `<span class="badge badge-accent">⏰ Deadline: ${PMS.escHtml(o.deadline)}</span>` : ''}
          ${o.posted_by_alumnus ? `<span class="badge" style="background:rgba(14,165,233,.15);color:var(--secondary);border-color:rgba(14,165,233,.3)">🎓 Shared by ${PMS.escHtml(o.posted_by_alumnus)}</span>` : ''}
        </div>
        <h2 style="font-size:1.15rem;margin-bottom:.35rem">${PMS.escHtml(o.title)}</h2>
        <p style="color:var(--primary-light);font-size:.875rem;font-weight:600;margin-bottom:.5rem">${PMS.escHtml(o.organization)}</p>
        ${o.location ? `<p style="color:var(--text-dim);font-size:.8rem;margin-bottom:.5rem">📍 ${PMS.escHtml(o.location)}</p>` : ''}
        <p style="color:var(--text-muted);font-size:.875rem;line-height:1.6;margin-bottom:.75rem">${PMS.escHtml(o.description || '')}</p>
        ${o.apply_url ? `<div style="margin-top:1rem"><a href="${PMS.safeUrl(o.apply_url)}" target="_blank" rel="noopener" class="btn btn-primary btn-sm">Apply Now ↗</a></div>` : ''}
      </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No active opportunities listed.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load opportunities.</p>';
  }
})();

