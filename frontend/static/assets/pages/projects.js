/**
 * PIEAS Materials Society (PMS)
 * Student FYPs & Projects Page Logic
 * frontend/static/assets/pages/projects.js
 */

(async () => {
  'use strict';

  const grid = document.getElementById('projects-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const projects = await PMS.apiFetch('/api/projects');
    grid.innerHTML = projects.length
      ? projects
          .map(
            (p, idx) => `
      <article class="card fade-up" style="transition-delay:${Math.min(idx, 8) * 60}ms">
        ${p.image_url ? `<img src="${PMS.safeUrl(p.image_url)}" alt="" width="400" height="160" loading="lazy" decoding="async" style="width:100%;height:160px;object-fit:cover;border-radius:8px;margin-bottom:1rem">` : ''}
        <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
          <span class="badge">${PMS.escHtml(p.project_type)}</span>
          <span class="badge badge-accent">Batch ${PMS.escHtml(p.batch)}</span>
        </div>
        <h3 style="font-size:1.15rem;margin-bottom:.5rem">${PMS.escHtml(p.title)}</h3>
        ${p.students ? `<p style="color:var(--text);font-size:13px;margin-bottom:4px;display:flex;align-items:center;gap:6px">${PMS.icon('users', 14)} Investigators: ${PMS.escHtml(p.students)}</p>` : ''}
        ${p.supervisors ? `<p style="color:var(--text-muted);font-size:13px;margin-bottom:8px;display:flex;align-items:center;gap:6px">${PMS.icon('microscope', 14)} Supervisors: ${PMS.escHtml(p.supervisors)}</p>` : ''}
        <p style="color:var(--text-secondary);font-size:14px;line-height:1.55;margin-bottom:12px">${PMS.escHtml(p.abstract || '')}</p>
        ${
          p.tags
            ? `<div style="display:flex;gap:4px;flex-wrap:wrap;margin:8px 0">${p.tags
                .split(',')
                .map(t => `<span class="tag">${PMS.escHtml(t.trim())}</span>`)
                .join('')}</div>`
            : ''
        }
        ${p.paper_or_repo_url ? `<div style="margin-top:auto;padding-top:12px"><a href="${PMS.safeUrl(p.paper_or_repo_url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm" style="display:inline-flex;align-items:center;gap:6px">${PMS.icon('external', 13)} View Paper / Repository</a></div>` : ''}
      </article>`
          )
          .join('')
      : PMS.emptyState('No student projects listed.');
    PMS.observeFadeElements(grid);
  } catch (_) {
    grid.innerHTML = PMS.errorState('Could not load projects.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
