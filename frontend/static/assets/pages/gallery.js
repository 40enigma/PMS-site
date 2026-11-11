/**
 * PIEAS Materials Society (PMS)
 * Micrograph & Photo Gallery Page Logic
 * frontend/static/assets/pages/gallery.js
 */

(async () => {
  const grid = document.getElementById('gallery-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    const images = await PMS.apiFetch('/api/gallery');
    grid.innerHTML = images.length ? images.map((img, idx) => {
      const title = (img.caption && img.caption !== 'null') ? img.caption : 'Micrograph';
      const captionDetail = img.event_name ? `${img.event_name}${img.year ? ' (' + img.year + ')' : ''}` : '';
      return `
        <figure class="card fade-up micrograph-lightbox-trigger"
          tabindex="0"
          role="button"
          aria-label="View micrograph in full size"
          data-img-url="${PMS.safeUrlRaw(img.image_url)}"
          data-img-title="${PMS.escHtml(title)}"
          data-img-caption="${PMS.escHtml(captionDetail)}"
          style="margin:0;padding:0;border-radius:var(--radius-lg);overflow:hidden;position:relative;aspect-ratio:4/3;background:var(--bg-card);border:1px solid var(--border);cursor:pointer;transition-delay:${idx * 60}ms">
          <img src="${PMS.safeUrl(img.image_url)}" alt="${PMS.escHtml(title)}" loading="lazy"
            style="width:100%;height:100%;object-fit:cover;transition:transform .3s ease">
          <figcaption style="position:absolute;bottom:0;left:0;right:0;background:linear-gradient(transparent,rgba(0,0,0,.88));padding:.75rem 1rem;font-size:.8rem;color:#f8fafc;pointer-events:none">
            <strong style="color:#fff">${PMS.escHtml(title)}</strong>
            ${captionDetail ? `<br><small style="color:#94a3b8">${PMS.escHtml(captionDetail)}</small>` : ''}
            <div style="font-size:12px;color:var(--text-muted);margin-top:4px;display:flex;align-items:center;gap:4px">${PMS.icon('search', 12)} Click to inspect</div>
          </figcaption>
        </figure>`;
    }).join('') : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No images in gallery.</p>';
    PMS.observeFadeElements(grid);
  } catch(e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1;text-align:center">Could not load gallery.</p>';
  }
})();

