/**
 * PIEAS Materials Society (PMS)
 * Alumni Directory Page Logic
 * frontend/static/assets/pages/alumni.js
 */

(async () => {
  'use strict';

  const grid = document.getElementById('alumni-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const alumni = await PMS.apiFetch('/api/alumni');
    grid.innerHTML = alumni.length
      ? alumni.map((a, idx) => PMS.alumniCard(a, idx)).join('')
      : PMS.emptyState('No alumni profiles listed.');
    PMS.observeFadeElements(grid);
  } catch (err) {
    PMS.reportError('alumni', err);
    grid.innerHTML = PMS.errorState('Could not load alumni.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
