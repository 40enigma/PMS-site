/**
 * PIEAS Materials Society (PMS)
 * Opportunities & Careers Page Logic
 * frontend/static/assets/pages/opportunities.js
 */

(async () => {
  'use strict';

  const grid = document.getElementById('opps-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(2);
  try {
    const opps = await PMS.apiFetch('/api/opportunities');
    grid.innerHTML = opps.length
      ? opps.map((o, idx) => PMS.opportunityCard(o, idx)).join('')
      : PMS.emptyState('No active opportunities listed.');
    PMS.observeFadeElements(grid);
  } catch (err) {
    PMS.reportError('opportunities', err);
    grid.innerHTML = PMS.errorState('Could not load opportunities.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
