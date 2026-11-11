/**
 * PIEAS Materials Society (PMS)
 * Blog & Technical Publications Page Logic
 * frontend/static/assets/pages/blog.js
 */

(async () => {
  const grid = document.getElementById('posts-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    const posts = await PMS.apiFetch('/api/posts');
    grid.innerHTML = posts.length
      ? posts.map((p, idx) => PMS.postCard(p, idx)).join('')
      : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No articles published yet.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load posts.</p>';
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();

