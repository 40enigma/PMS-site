/**
 * PIEAS Materials Society (PMS)
 * Academic Resources & Toolkit Page Logic
 * frontend/static/assets/pages/resources.js
 */

let allResources = [];

const ALIASES = {
  software: ['software', 'software & tools', 'software & tool guides', 'tools'],
  books: ['books', 'books & handbooks', 'course notes', 'handbooks', 'guides'],
  standards: ['standards', 'standard specifications', 'astm', 'specifications'],
  safety: ['safety', 'safety guidelines'],
  databases: ['databases', 'materials databases']
};

async function loadResources() {
  const grid = document.getElementById('resources-grid');
  if (!grid) return;
  grid.setAttribute('aria-busy', 'true');
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    allResources = await PMS.apiFetch('/api/resources');
    renderFilteredResources('');
  } catch (e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1">Failed to load resources.</p>';
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
    : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No resources found in this category.</p>';
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
    const sections = await PMS.apiFetch('/api/sections');
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
          strip.innerHTML = protocols.map(p => `
            <div class="card" style="padding:20px;border-left:2px solid var(--accent)">
              <span class="tag" style="margin-bottom:8px;font-family:var(--font-mono);color:var(--text)">${PMS.escHtml(p.code)}</span>
              <h3 style="font-size:14px;margin-bottom:6px;color:var(--text)">${PMS.escHtml(p.title)}</h3>
              <p style="font-size:13px;color:var(--text-secondary);line-height:1.5">${PMS.escHtml(p.detail)}</p>
            </div>
          `).join('');
          return;
        } catch (_) { }
      }
    }
    strip.innerHTML = '';
  } catch (_) {
    strip.innerHTML = '';
  } finally {
    strip.setAttribute('aria-busy', 'false');
  }
}

loadResources();
loadAstmStrip();

