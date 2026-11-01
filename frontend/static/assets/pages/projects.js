/**
 * PIEAS Materials Society (PMS)
 * Student FYPs & Projects Page Logic
 * frontend/static/assets/pages/projects.js
 */

(async () => {
  const grid = document.getElementById('projects-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const projects = await PMS.apiFetch('/api/projects');
    grid.innerHTML = projects.length ? projects.map((p, idx) => `
      <article class="card fade-up" style="transition-delay:${idx * 60}ms">
        ${p.image_url ? `<img src="${PMS.safeUrl(p.image_url)}" alt="${PMS.escHtml(p.title)}" style="width:100%;height:160px;object-fit:cover;border-radius:8px;margin-bottom:1rem">` : ''}
        <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
          <span class="badge">${PMS.escHtml(p.project_type)}</span>
          <span class="badge badge-accent">Batch ${PMS.escHtml(p.batch)}</span>
        </div>
        <h2 style="font-size:1.15rem;margin-bottom:.5rem">${PMS.escHtml(p.title)}</h2>
        ${p.students ? `<p style="color:var(--primary-light);font-size:.85rem;margin-bottom:.35rem">👥 Investigators: ${PMS.escHtml(p.students)}</p>` : ''}
        ${p.supervisors ? `<p style="color:var(--text-dim);font-size:.8rem;margin-bottom:.5rem">🔬 Supervisors: ${PMS.escHtml(p.supervisors)}</p>` : ''}
        <p style="color:var(--text-muted);font-size:.875rem;line-height:1.6;margin-bottom:.75rem">${PMS.escHtml(p.abstract || '')}</p>
        ${p.tags ? `<div style="display:flex;gap:.35rem;flex-wrap:wrap;margin:.6rem 0">${p.tags.split(',').map(t => `<span class="tag">${PMS.escHtml(t.trim())}</span>`).join('')}</div>` : ''}
        ${p.paper_or_repo_url ? `<div style="margin-top:1rem"><a href="${PMS.safeUrl(p.paper_or_repo_url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm">View Paper / Repo ↗</a></div>` : ''}
      </article>`).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No student projects listed.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load projects.</p>';
  }
})();

