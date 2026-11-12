/**
 * PIEAS Materials Society (PMS)
 * Academic Resources & Toolkit Page Logic
 * frontend/static/assets/pages/resources.js
 */

(() => {
  'use strict';

  let allResources = [];

  const ALIASES = {
    software: ['software', 'software & tools', 'software & tool guides', 'tools'],
    books: ['books', 'books & handbooks', 'course notes', 'handbooks', 'guides'],
    standards: ['standards', 'standard specifications', 'astm', 'specifications'],
    safety: ['safety', 'safety guidelines'],
    databases: ['databases', 'materials databases'],
  };

  async function loadResources() {
    const grid = document.getElementById('resources-grid');
    if (!grid) return;
    grid.setAttribute('aria-busy', 'true');
    grid.innerHTML = PMS.skeletonGrid(3);
    try {
      allResources = await PMS.apiFetch('/api/resources');
      renderFilteredResources('');
    } catch (err) {
      PMS.reportError('resources', err);
      grid.innerHTML = PMS.errorState('Could not load resources.');
    } finally {
      grid.setAttribute('aria-busy', 'false');
    }
  }

  function renderFilteredResources(category) {
    const grid = document.getElementById('resources-grid');
    if (!grid) return;
    const filtered = category
      ? allResources.filter(r => {
          if (!r.category) return false;
          const rCat = r.category.toLowerCase().trim();
          const target = category.toLowerCase().trim();
          if (ALIASES[target]) {
            return ALIASES[target].some(alias => rCat === alias || rCat.includes(alias));
          }
          return rCat === target || rCat.includes(target);
        })
      : allResources;

    grid.innerHTML = filtered.length
      ? filtered.map((r, idx) => PMS.resourceCard(r, idx)).join('')
      : PMS.emptyState('No resources found in this category.');
    PMS.observeFadeElements(grid);
  }

  // Category filter buttons
  document.querySelectorAll('.res-filter').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.res-filter').forEach(b => {
        const isActive = b === btn;
        b.classList.toggle('active', isActive);
        b.setAttribute('aria-pressed', String(isActive));
      });
      renderFilteredResources(btn.dataset.cat);
    });
  });

  // Load ASTM standards strip from site_sections
  async function loadAstmStrip() {
    const strip = document.getElementById('astm-strip');
    if (!strip) return;
    strip.setAttribute('aria-busy', 'true');
    try {
      const sections = await PMS.apiFetchCached('/api/sections');
      const sec = sections.find(s => s.section_key === 'resources_standards');
      if (sec) {
        if (sec.badge) {
          const b = document.getElementById('astm-badge');
          if (b) b.textContent = sec.badge;
        }
        if (sec.title) {
          const t = document.getElementById('astm-title');
          if (t) t.textContent = sec.title;
        }
        if (sec.content || sec.subtitle) {
          const d = document.getElementById('astm-desc');
          if (d) d.textContent = sec.content || sec.subtitle;
        }

        if (sec.extra_json) {
          try {
            const protocols = JSON.parse(sec.extra_json);
            strip.innerHTML = protocols
              .map(
                p => `
              <div class="card fade-up protocol-card">
                <span class="tag">${PMS.escHtml(p.code)}</span>
                <h3 class="card__title card__title--xs">${PMS.escHtml(p.title)}</h3>
                <p class="card__text card__text--sm">${PMS.escHtml(p.detail)}</p>
              </div>
            `
              )
              .join('');
            PMS.observeFadeElements(strip);
            return;
          } catch (err) {
            PMS.reportError('standards-json', err);
          }
        }
      }
      strip.innerHTML = PMS.emptyState('Testing protocols will be published soon.');
    } catch (err) {
      PMS.reportError('standards', err);
      strip.innerHTML = PMS.errorState('Could not load testing protocols.');
    } finally {
      strip.setAttribute('aria-busy', 'false');
    }
  }

  loadResources();
  loadAstmStrip();
})();
