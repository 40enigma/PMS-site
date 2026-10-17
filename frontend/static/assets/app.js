/**
 * PMS – PIEAS Materials Society
 * Shared client-side JS utilities (no bundler required)
 * frontend/static/assets/app.js
 */

/* ── API base ── */
const API = '';  // Same-origin: requests go to /api/*

/**
 * Generic API fetch wrapper.
 * Returns parsed JSON or throws an Error with a safe message.
 */
async function apiFetch(path, options = {}) {
  const res = await fetch(`${API}${path}`, {
    credentials: 'include',   // send cookies for authenticated requests
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) },
    ...options,
  });

  if (!res.ok) {
    let msg = `HTTP ${res.status}`;
    try {
      const body = await res.json();
      msg = body.error || msg;
    } catch (_) {}
    throw new Error(msg);
  }

  return res.json();
}

/** POST JSON helper */
async function apiPost(path, data) {
  return apiFetch(path, { method: 'POST', body: JSON.stringify(data) });
}

/** PUT JSON helper */
async function apiPut(path, data) {
  return apiFetch(path, { method: 'PUT', body: JSON.stringify(data) });
}

/** DELETE helper */
async function apiDelete(path) {
  return apiFetch(path, { method: 'DELETE' });
}

/* ── DOM utilities ── */
function qs(sel, ctx = document) { return ctx.querySelector(sel); }
function qsa(sel, ctx = document) { return [...ctx.querySelectorAll(sel)]; }

function showAlert(container, message, type = 'error', isHtml = false) {
  container.innerHTML = `<div class="alert alert-${type}" role="alert">${isHtml ? message : escHtml(message)}</div>`;
}

function escHtml(str) {
  return String(str)
    .replace(/&/g,'&amp;')
    .replace(/</g,'&lt;')
    .replace(/>/g,'&gt;')
    .replace(/"/g,'&quot;')
    .replace(/'/g,'&#39;');
}

function setLoading(el, loading) {
  if (loading) {
    el.disabled = true;
    el.dataset.orig = el.textContent;
    el.textContent = 'Loading…';
  } else {
    el.disabled = false;
    el.textContent = el.dataset.orig || 'Submit';
  }
}

/** Format ISO date string to human readable */
function fmtDate(iso) {
  if (!iso) return '';
  return new Date(iso).toLocaleDateString('en-PK', { year:'numeric', month:'short', day:'numeric' });
}

/* ── Navbar ── */
function initNavbar() {
  const hamburger = qs('.hamburger');
  const navLinks   = qs('.nav-links');
  if (!hamburger || !navLinks) return;
  hamburger.addEventListener('click', () => {
    navLinks.classList.toggle('open');
    hamburger.setAttribute('aria-expanded', navLinks.classList.contains('open'));
  });

  // Mark current page as active
  const path = window.location.pathname.replace(/\/$/, '') || '/';
  qsa('.nav-links a').forEach(a => {
    const href = a.getAttribute('href').replace(/\/$/, '') || '/';
    if (href === path) a.classList.add('active');
  });
}

/* ── Announcement banner ── */
async function loadAnnouncements(container) {
  if (!container) return;
  try {
    const data = await apiFetch('/api/announcements');
    if (!data.length) return;
    const a = data[0];
    container.innerHTML = `
      <div class="container">
        📢 ${escHtml(a.message)}
        ${a.link_url ? `<a href="${escHtml(a.link_url)}" style="margin-left:.5rem;color:#93c5fd;font-weight:600;">${escHtml(a.link_text || 'More info →')}</a>` : ''}
      </div>`;
    container.style.display = 'block';
  } catch (_) { /* silently ignore */ }
}

/* ── Stats ── */
async function loadStats(container) {
  if (!container) return;
  try {
    const s = await apiFetch('/api/stats');
    container.querySelector('[data-stat="members"]').textContent  = s.active_members;
    container.querySelector('[data-stat="events"]').textContent   = s.events_held;
    container.querySelector('[data-stat="years"]').textContent    = s.years_active;
    container.querySelector('[data-stat="partners"]').textContent = s.industrial_partners;
  } catch (_) { /* ignore */ }
}

/* ── Generic card renderer helpers ── */
function eventCard(e) {
  return `
    <article class="card fade-up">
      ${e.poster_url ? `<img src="${escHtml(e.poster_url)}" alt="${escHtml(e.title)}" style="width:100%;height:180px;object-fit:cover;border-radius:10px;margin-bottom:1rem;">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem;">
        <span class="badge">${escHtml(e.category)}</span>
        ${e.is_past ? '<span class="badge" style="background:rgba(100,116,139,.2);color:#94a3b8">Past</span>' : '<span class="badge badge-success">Upcoming</span>'}
      </div>
      <h3 style="margin-bottom:.5rem">${escHtml(e.title)}</h3>
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:.75rem">${escHtml(e.date)} ${e.time_range ? '• '+escHtml(e.time_range) : ''}</p>
      ${e.venue ? `<p style="color:var(--text-dim);font-size:.8rem;margin-bottom:.75rem">📍 ${escHtml(e.venue)}</p>` : ''}
      <p style="color:var(--text-muted);font-size:.875rem">${escHtml(e.short_description || '')}</p>
    </article>`;
}

function teamCard(m) {
  return `
    <article class="card fade-up" style="text-align:center">
      ${m.photo_url
        ? `<img src="${escHtml(m.photo_url)}" alt="${escHtml(m.name)}" style="width:80px;height:80px;border-radius:50%;object-fit:cover;margin:0 auto .75rem;border:3px solid var(--primary)">`
        : `<div style="width:80px;height:80px;border-radius:50%;background:var(--gradient-card);display:flex;align-items:center;justify-content:center;font-size:2rem;margin:0 auto .75rem">👤</div>`}
      <h3 style="font-size:1rem;margin-bottom:.25rem">${escHtml(m.name)}</h3>
      <p style="color:var(--primary-light);font-size:.85rem;font-weight:600;margin-bottom:.5rem">${escHtml(m.role)}</p>
      ${m.batch ? `<p style="color:var(--text-dim);font-size:.78rem">Batch ${escHtml(m.batch)}</p>` : ''}
      ${m.linkedin_url ? `<a href="${escHtml(m.linkedin_url)}" target="_blank" rel="noopener" style="font-size:.8rem;margin-top:.5rem;display:inline-block">LinkedIn →</a>` : ''}
    </article>`;
}

function postCard(p) {
  return `
    <article class="card fade-up">
      <span class="badge" style="margin-bottom:.75rem">${escHtml(p.category)}</span>
      <h3 style="margin-bottom:.5rem">${escHtml(p.title)}</h3>
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:.75rem">${escHtml(p.excerpt || '')}</p>
      <div style="display:flex;justify-content:space-between;align-items:center;font-size:.78rem;color:var(--text-dim)">
        <span>${escHtml(p.author_name || 'PMS')} ${p.author_batch ? '· Batch '+escHtml(p.author_batch) : ''}</span>
        <span>${fmtDate(p.published_at)}</span>
      </div>
    </article>`;
}

function resourceCard(r) {
  return `
    <article class="card fade-up">
      ${r.is_pinned ? '<span class="badge badge-accent" style="margin-bottom:.75rem">📌 Pinned</span>' : ''}
      <h3 style="font-size:1rem;margin-bottom:.5rem">${escHtml(r.title)}</h3>
      ${r.software_tag ? `<span class="tag">${escHtml(r.software_tag)}</span>` : ''}
      <p style="color:var(--text-muted);font-size:.85rem;margin:.5rem 0">${escHtml(r.description || '')}</p>
      <a href="${escHtml(r.url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm" style="margin-top:.5rem">Open Resource ↗</a>
    </article>`;
}

/* ── On DOM ready ── */
document.addEventListener('DOMContentLoaded', () => {
  initNavbar();
  loadAnnouncements(qs('#announcement-banner'));
  loadStats(qs('#stats-strip'));
});

/* Export for use in page-specific scripts */
window.PMS = {
  apiFetch, apiPost, apiPut, apiDelete,
  escHtml, fmtDate, showAlert, setLoading,
  eventCard, teamCard, postCard, resourceCard,
};
