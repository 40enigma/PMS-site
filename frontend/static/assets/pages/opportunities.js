/**
 * PIEAS Materials Society (PMS)
 * Opportunities & Careers Page Logic
 * frontend/static/assets/pages/opportunities.js
 */

(async () => {
  const grid = document.getElementById('opps-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const opps = await PMS.apiFetch('/api/opportunities');
    grid.innerHTML = opps.length ? opps.map((o, idx) => `
      <article class="card fade-up" style="transition-delay:${Math.min(idx, 8) * 60}ms">
        <div style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:12px">
          <span class="badge">${PMS.escHtml(o.opportunity_type)}</span>
          ${o.deadline ? `<span class="badge badge-accent">Deadline: ${PMS.escHtml(o.deadline)}</span>` : ''}
          ${o.posted_by_alumnus ? `<span class="badge">Shared by ${PMS.escHtml(o.posted_by_alumnus)}</span>` : ''}
        </div>
        <h3 style="font-size:1.15rem;margin-bottom:6px">${PMS.escHtml(o.title)}</h3>
        <p style="color:var(--text);font-size:14px;font-weight:500;margin-bottom:8px">${PMS.escHtml(o.organization)}</p>
        ${o.location ? `<p style="color:var(--text-muted);font-size:13px;margin-bottom:8px;display:flex;align-items:center;gap:6px">${PMS.icon('map-pin', 13)} ${PMS.escHtml(o.location)}</p>` : ''}
        <p style="color:var(--text-secondary);font-size:14px;line-height:1.55;margin-bottom:12px">${PMS.escHtml(o.description || '')}</p>
        ${o.apply_url ? `<div style="margin-top:auto;padding-top:12px"><a href="${PMS.safeUrl(o.apply_url)}" target="_blank" rel="noopener" class="btn btn-primary btn-sm" style="display:inline-flex;align-items:center;gap:6px">${PMS.icon('external', 13)} Apply</a></div>` : ''}
      </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No active opportunities listed.</p>';
    PMS.observeFadeElements(grid);
  } catch (e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load opportunities.</p>';
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();

