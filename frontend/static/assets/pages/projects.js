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
      ? projects.map((p, idx) => PMS.projectCard(p, idx)).join('')
      : PMS.emptyState('No student projects listed.');
    PMS.observeFadeElements(grid);
  } catch (err) {
    PMS.reportError('projects', err);
    grid.innerHTML = PMS.errorState('Could not load projects.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
