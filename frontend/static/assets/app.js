/**
 * PMS – PIEAS Materials Society
 * Shared client-side JS utilities (no bundler required)
 * frontend/static/assets/app.js
 */

/* ── API base ── */
const API = ''; // Same-origin: requests go to /api/*

/**
 * Generic API fetch wrapper.
 * Returns parsed JSON or throws an Error with a safe message.
 */
async function apiFetch(path, options = {}) {
  const res = await fetch(`${API}${path}`, {
    credentials: 'include', // send cookies for authenticated requests
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
function qs(sel, ctx = document) {
  return ctx.querySelector(sel);
}
function qsa(sel, ctx = document) {
  return [...ctx.querySelectorAll(sel)];
}

function showAlert(container, message, type = 'error', isHtml = false) {
  container.innerHTML = `<div class="alert alert-${type}" role="alert">${isHtml ? message : escHtml(message)}</div>`;
}

function escHtml(str) {
  if (str === null || str === undefined) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/**
 * Sanitize URL to protect against javascript: and malicious data: XSS vectors.
 * Strictly permits http://, https://, mailto:, relative /, or #.
 */
function safeUrl(url) {
  if (!url || typeof url !== 'string') return '#';
  const trimmed = url.trim();
  if (
    trimmed.startsWith('https://') ||
    trimmed.startsWith('http://') ||
    trimmed.startsWith('mailto:') ||
    trimmed.startsWith('/') ||
    trimmed.startsWith('#')
  ) {
    return escHtml(trimmed);
  }
  return '#';
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
  return new Date(iso).toLocaleDateString('en-PK', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/** Lightweight Markdown parser for technical articles */
function renderMarkdown(md) {
  if (!md) return '';
  let html = escHtml(md);

  // Fenced code blocks
  html = html.replace(/```([\s\S]*?)```/g, (_, code) => `<pre class="code-block" style="background:rgba(0,0,0,.4);padding:1rem;border-radius:8px;overflow-x:auto;margin:1rem 0;border:1px solid var(--border)"><code>${code.trim()}</code></pre>`);

  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code style="background:rgba(255,255,255,.08);padding:.2rem .4rem;border-radius:4px;font-size:.875em">$1</code>');

  // Headings
  html = html.replace(/^### (.*$)/gim, '<h3 style="font-size:1.15rem;margin:1.25rem 0 .5rem;color:var(--text)">$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2 style="font-size:1.35rem;margin:1.5rem 0 .6rem;color:var(--text)">$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1 style="font-size:1.6rem;margin:1.75rem 0 .75rem;color:var(--primary-light)">$1</h1>');

  // Bold & Italics
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>');

  // Blockquotes
  html = html.replace(/^> (.*$)/gim, '<blockquote style="border-left:3px solid var(--primary);padding-left:1rem;margin:1rem 0;color:var(--text-muted);font-style:italic">$1</blockquote>');

  // Unordered list items
  html = html.replace(/^\s*[-*]\s+(.*$)/gim, '<li style="margin-left:1.5rem;margin-bottom:.35rem">$1</li>');

  // Paragraphs
  const paragraphs = html.split(/\n\s*\n/);
  return paragraphs
    .map(p => {
      p = p.trim();
      if (!p) return '';
      if (p.startsWith('<h') || p.startsWith('<pre') || p.startsWith('<blockquote') || p.startsWith('<li')) {
        return p;
      }
      return `<p style="margin-bottom:1rem;line-height:1.7;color:var(--text)">${p}</p>`;
    })
    .join('');
}

/* ── Navbar ── */
function initNavbar() {
  const hamburger = qs('.hamburger');
  const navLinks = qs('.nav-links');
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
        ${a.link_url ? `<a href="${safeUrl(a.link_url)}" style="margin-left:.5rem;color:#93c5fd;font-weight:600;">${escHtml(a.link_text || 'More info →')}</a>` : ''}
      </div>`;
    container.style.display = 'block';
  } catch (_) {
    /* silently ignore */
  }
}

/* ── Stats ── */
async function loadStats(container) {
  if (!container) return;
  try {
    const s = await apiFetch('/api/stats');
    const elM = container.querySelector('[data-stat="members"]');
    const elE = container.querySelector('[data-stat="events"]');
    const elY = container.querySelector('[data-stat="years"]');
    const elP = container.querySelector('[data-stat="partners"]');
    if (elM) elM.textContent = s.active_members;
    if (elE) elE.textContent = s.events_held;
    if (elY) elY.textContent = s.years_active;
    if (elP) elP.textContent = s.industrial_partners;
  } catch (_) {
    /* ignore */
  }
}

/* ── Generic card renderer helpers ── */
function eventCard(e) {
  return `
    <article class="card fade-up">
      ${e.poster_url ? `<img src="${safeUrl(e.poster_url)}" alt="${escHtml(e.title)}" style="width:100%;height:180px;object-fit:cover;border-radius:10px;margin-bottom:1rem;">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem;">
        <span class="badge">${escHtml(e.category)}</span>
        ${e.is_past ? '<span class="badge" style="background:rgba(100,116,139,.2);color:#94a3b8">Past</span>' : '<span class="badge badge-success">Upcoming</span>'}
        ${e.is_featured ? '<span class="badge badge-accent">Featured</span>' : ''}
      </div>
      <h3 style="margin-bottom:.5rem">${escHtml(e.title)}</h3>
      ${e.speaker_or_organizer ? `<p style="color:var(--secondary);font-size:.825rem;font-weight:600;margin-bottom:.5rem">🎙️ ${escHtml(e.speaker_or_organizer)}</p>` : ''}
      <p style="color:var(--text-muted);font-size:.85rem;margin-bottom:.5rem">📅 ${escHtml(e.date)}${e.time_range ? ' · ' + escHtml(e.time_range) : ''}</p>
      ${e.venue ? `<p style="color:var(--text-dim);font-size:.8rem;margin-bottom:.75rem">📍 ${escHtml(e.venue)}</p>` : ''}
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:.75rem">${escHtml(e.short_description || '')}</p>
      ${e.full_summary ? `
        <details style="margin:.75rem 0;font-size:.85rem;color:var(--text-muted);cursor:pointer">
          <summary style="color:var(--primary-light);font-weight:500">View Full Agenda &amp; Details</summary>
          <div style="padding-top:.5rem;line-height:1.6">${escHtml(e.full_summary)}</div>
        </details>` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-top:1rem">
        ${e.registration_link ? `<a href="${safeUrl(e.registration_link)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm">External Link ↗</a>` : ''}
        ${!e.is_past ? `<button class="btn btn-primary btn-sm btn-open-reg" data-event-id="${e.id}" data-event-title="${escHtml(e.title)}">Register Now →</button>` : ''}
      </div>
    </article>`;
}

function teamCard(m) {
  return `
    <article class="card fade-up" style="text-align:center">
      ${m.photo_url
        ? `<img src="${safeUrl(m.photo_url)}" alt="${escHtml(m.name)}" style="width:80px;height:80px;border-radius:50%;object-fit:cover;margin:0 auto .75rem;border:3px solid var(--primary)">`
        : `<div style="width:80px;height:80px;border-radius:50%;background:var(--gradient-card);display:flex;align-items:center;justify-content:center;font-size:2rem;margin:0 auto .75rem">👤</div>`}
      <h3 style="font-size:1rem;margin-bottom:.25rem">${escHtml(m.name)}</h3>
      <p style="color:var(--primary-light);font-size:.85rem;font-weight:600;margin-bottom:.5rem">${escHtml(m.role)}</p>
      ${m.batch ? `<p style="color:var(--text-dim);font-size:.78rem">Batch ${escHtml(m.batch)}</p>` : ''}
      ${m.bio ? `<p style="color:var(--text-muted);font-size:.8rem;margin-top:.5rem">${escHtml(m.bio)}</p>` : ''}
      ${m.linkedin_url ? `<a href="${safeUrl(m.linkedin_url)}" target="_blank" rel="noopener" style="font-size:.8rem;margin-top:.5rem;display:inline-block">LinkedIn →</a>` : ''}
    </article>`;
}

function postCard(p) {
  return `
    <article class="card fade-up">
      ${p.cover_image_url ? `<img src="${safeUrl(p.cover_image_url)}" alt="${escHtml(p.title)}" style="width:100%;height:160px;object-fit:cover;border-radius:8px;margin-bottom:1rem">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
        <span class="badge">${escHtml(p.category)}</span>
        ${p.material_formula ? `<span class="tag">🧪 ${escHtml(p.material_formula)}</span>` : ''}
      </div>
      <h3 style="margin-bottom:.5rem">${escHtml(p.title)}</h3>
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:.75rem">${escHtml(p.excerpt || '')}</p>
      ${p.crystal_structure ? `<p style="color:var(--text-dim);font-size:.78rem;margin-bottom:.5rem">Structure: ${escHtml(p.crystal_structure)}</p>` : ''}
      <div style="display:flex;justify-content:space-between;align-items:center;font-size:.78rem;color:var(--text-dim);margin-bottom:.75rem">
        <span>${escHtml(p.author_name || 'PMS')}</span>
        <span>${fmtDate(p.published_at)}</span>
      </div>
      <button class="btn btn-outline btn-sm btn-read-article" data-slug="${escHtml(p.slug)}" style="width:100%">Read Article →</button>
    </article>`;
}

function resourceCard(r) {
  return `
    <article class="card fade-up">
      ${r.is_pinned ? '<span class="badge badge-accent" style="margin-bottom:.75rem">📌 Pinned</span>' : ''}
      <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:.5rem">
        <span class="badge">${escHtml(r.category)}</span>
        ${r.software_tag ? `<span class="tag">${escHtml(r.software_tag)}</span>` : ''}
      </div>
      <h3 style="font-size:1rem;margin-bottom:.5rem">${escHtml(r.title)}</h3>
      <p style="color:var(--text-muted);font-size:.85rem;margin:.5rem 0">${escHtml(r.description || '')}</p>
      <a href="${safeUrl(r.url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm" style="margin-top:.5rem">Open Resource ↗</a>
    </article>`;
}

/** Open Article Modal and load full article */
async function openArticleModal(slug) {
  let modal = document.getElementById('article-reader-modal');
  if (!modal) {
    modal = document.createElement('dialog');
    modal.id = 'article-reader-modal';
    modal.style.cssText = 'background:var(--bg-card);border:1px solid var(--border);border-radius:var(--radius-lg);color:var(--text);max-width:760px;width:95%;padding:2rem;max-height:85vh;overflow-y:auto;box-shadow:0 25px 50px -12px rgba(0,0,0,.7);';
    document.body.appendChild(modal);
  }

  modal.innerHTML = `
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.5rem">
      <span class="badge">Technical Article</span>
      <button id="close-article-modal" class="btn btn-outline btn-sm" style="padding:.25rem .6rem">✕ Close</button>
    </div>
    <div id="article-reader-content"><div class="spinner"></div></div>
  `;
  modal.showModal();
  document.getElementById('close-article-modal').onclick = () => modal.close();

  try {
    const post = await apiFetch(`/api/posts/${encodeURIComponent(slug)}`);
    const content = document.getElementById('article-reader-content');
    content.innerHTML = `
      ${post.cover_image_url ? `<img src="${safeUrl(post.cover_image_url)}" alt="${escHtml(post.title)}" style="width:100%;max-height:300px;object-fit:cover;border-radius:10px;margin-bottom:1.5rem">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
        <span class="badge">${escHtml(post.category)}</span>
        ${post.material_formula ? `<span class="tag">🧪 ${escHtml(post.material_formula)}</span>` : ''}
        ${post.crystal_structure ? `<span class="tag">💎 ${escHtml(post.crystal_structure)}</span>` : ''}
      </div>
      <h1 style="font-size:1.8rem;margin-bottom:.75rem;color:var(--text)">${escHtml(post.title)}</h1>
      <div style="display:flex;gap:1.5rem;font-size:.85rem;color:var(--text-muted);border-bottom:1px solid var(--border);padding-bottom:1rem;margin-bottom:1.5rem">
        <span>✍️ <strong>${escHtml(post.author_name || 'PMS Technical Council')}</strong> ${post.author_role ? '· ' + escHtml(post.author_role) : ''}</span>
        <span>📅 ${fmtDate(post.published_at)}</span>
        ${post.read_time_minutes ? `<span>⏱️ ${post.read_time_minutes} min read</span>` : ''}
      </div>
      <div class="article-body" style="font-size:1rem;color:var(--text);line-height:1.75">
        ${renderMarkdown(post.body_markdown)}
      </div>
    `;
  } catch (err) {
    document.getElementById('article-reader-content').innerHTML = `
      <div class="alert alert-error">Failed to load article: ${escHtml(err.message)}</div>
    `;
  }
}

/* Event delegation for Article Reader buttons across any page */
document.addEventListener('click', ev => {
  const btn = ev.target.closest('.btn-read-article');
  if (btn && btn.dataset.slug) {
    ev.preventDefault();
    openArticleModal(btn.dataset.slug);
  }
});

/* ── On DOM ready ── */
document.addEventListener('DOMContentLoaded', () => {
  initNavbar();
  loadAnnouncements(qs('#announcement-banner'));
  loadStats(qs('#stats-strip'));
});

/* Export for use in page-specific scripts */
window.PMS = {
  apiFetch,
  apiPost,
  apiPut,
  apiDelete,
  escHtml,
  safeUrl,
  fmtDate,
  renderMarkdown,
  openArticleModal,
  showAlert,
  setLoading,
  eventCard,
  teamCard,
  postCard,
  resourceCard,
};
