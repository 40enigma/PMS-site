/**
 * PIEAS Materials Society (PMS)
 * Micrograph & Photo Gallery Page Logic
 * frontend/static/assets/pages/gallery.js
 */

(async () => {
  'use strict';

  const grid = document.getElementById('gallery-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    const images = await PMS.apiFetch('/api/gallery');
    grid.innerHTML = images.length
      ? images
          .map((img, idx) => {
            const title = img.caption && img.caption !== 'null' ? img.caption : 'Micrograph';
            const captionDetail = img.event_name
              ? `${img.event_name}${img.year ? ' (' + img.year + ')' : ''}`
              : '';
            return `
        <figure class="card fade-up micrograph-card" style="${PMS.stagger(idx)}">
          <button type="button"
            class="micrograph-lightbox-trigger"
            aria-label="View ${PMS.escHtml(title)} in full size"
            data-img-url="${PMS.safeUrl(img.image_url)}"
            data-img-title="${PMS.escHtml(title)}"
            data-img-caption="${PMS.escHtml(captionDetail)}">
            <img src="${PMS.safeUrl(img.image_url)}" alt="${PMS.escHtml(title)}" width="400" height="300" loading="lazy" decoding="async">
          </button>
          <figcaption style="position:absolute;bottom:0;left:0;right:0;background:linear-gradient(transparent,var(--overlay-scrim));padding:.75rem 1rem;font-size:var(--fs-sm);color:var(--overlay-text);pointer-events:none">
            <strong style="color:var(--overlay-text)">${PMS.escHtml(title)}</strong>
            ${captionDetail ? `<br><small style="color:var(--overlay-text-muted)">${PMS.escHtml(captionDetail)}</small>` : ''}
            <div style="font-size:var(--fs-xs);color:var(--overlay-text-muted);margin-top:4px;display:flex;align-items:center;gap:4px">${PMS.icon('search', 12)} Click to inspect</div>
          </figcaption>
        </figure>`;
          })
          .join('')
      : PMS.emptyState('No images in gallery.');
    PMS.observeFadeElements(grid);
  } catch (err) {
    PMS.reportError('gallery', err);
    grid.innerHTML = PMS.errorState('Could not load gallery.');
  } finally {
    grid.setAttribute('aria-busy', 'false');
  }
})();
