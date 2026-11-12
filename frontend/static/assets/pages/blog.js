/**
 * PIEAS Materials Society (PMS)
 * Blog & Technical Publications Page Logic
 * frontend/static/assets/pages/blog.js
 */

(async () => {
  'use strict';

  const grid = document.getElementById('posts-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    const posts = await PMS.apiFetch('/api/posts');
    grid.innerHTML = posts.length
      ? posts.map((p, idx) => PMS.postCard(p, idx)).join('')
      : PMS.emptyState('No articles published yet.');
    PMS.observeFadeElements(grid);
  } catch (err) {
    PMS.reportError('posts', err);
    grid.innerHTML = PMS.errorState('Could not load posts.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
