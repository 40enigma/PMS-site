/**
 * PMS – PIEAS Materials Society
 * Shared Modern Client-Side JS Architecture
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

  const text = await res.text();
  if (!text || !text.trim()) {
    return {};
  }
  try {
    return JSON.parse(text);
  } catch (_) {
    return {};
  }
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
  if (!container) return;
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
  if (!el) return;
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
  const d = new Date(String(iso).replace(' ', 'T'));
  if (isNaN(d.getTime())) return String(iso);
  return d.toLocaleDateString('en-PK', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Robust, safe Markdown parser for technical articles.
 * Extracts fenced code blocks & inline code first using unique placeholders
 * so regexes for headings/bold/italics/lists never corrupt Python comments (#)
 * or C pointers (*ptr). Wraps lists in <ul> without orphan <li> elements.
 */
function renderMarkdown(md) {
  if (!md) return '';

  const codeBlocks = [];
  const inlineCodes = [];

  // 1. Protect fenced code blocks (```lang ... ```)
  let text = md.replace(/```([a-zA-Z0-9_-]*)\n?([\s\S]*?)```/g, (_, lang, code) => {
    const idx = codeBlocks.length;
    codeBlocks.push({ lang, code: code.trim() });
    return `\n\n__PMS_CODE_BLOCK_${idx}__\n\n`;
  });

  // 2. Protect inline code (`code`)
  text = text.replace(/`([^`\n]+)`/g, (_, code) => {
    const idx = inlineCodes.length;
    inlineCodes.push(code);
    return `__PMS_INLINE_CODE_${idx}__`;
  });

  // Escape HTML characters in markdown body to prevent XSS
  text = escHtml(text);

  // 3. Headings
  text = text.replace(/^### (.*$)/gim, '<h3 style="font-size:1.15rem;margin:1.35rem 0 .5rem;color:var(--text)">$1</h3>');
  text = text.replace(/^## (.*$)/gim, '<h2 style="font-size:1.35rem;margin:1.6rem 0 .6rem;color:var(--text)">$1</h2>');
  text = text.replace(/^# (.*$)/gim, '<h1 style="font-size:1.65rem;margin:1.85rem 0 .75rem;color:var(--primary-light)">$1</h1>');

  // 4. Bold & Italics
  text = text.replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>');
  text = text.replace(/\*([^*\n]+)\*/g, '<em>$1</em>');

  // 5. Links [text](url)
  text = text.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, linkText, url) => {
    return `<a href="${safeUrl(url.replace(/&amp;/g, '&'))}" target="_blank" rel="noopener" style="color:var(--primary-light);text-decoration:underline">${linkText}</a>`;
  });

  // 6. Blockquotes
  text = text.replace(/^> (.*$)/gim, '<blockquote style="border-left:3px solid var(--accent);padding-left:1rem;margin:1.2rem 0;color:var(--text-muted);font-style:italic">$1</blockquote>');

  // 7. Group consecutive list items into <ul> without orphan <li>
  text = text.replace(/(?:^\s*[-*]\s+.*(?:\r?\n|$))+/gm, match => {
    const items = match
      .trim()
      .split(/\r?\n/)
      .map(line => {
        const itemContent = line.replace(/^\s*[-*]\s+/, '');
        return `<li style="margin-bottom:.4rem">${itemContent}</li>`;
      })
      .join('');
    return `<ul style="margin:1rem 0 1rem 1.75rem;color:var(--text);line-height:1.6">${items}</ul>`;
  });

  // 8. Paragraphs
  const blocks = text.split(/\n\s*\n/);
  text = blocks
    .map(b => {
      b = b.trim();
      if (!b) return '';
      if (
        b.startsWith('<h') ||
        b.startsWith('__PMS_CODE_BLOCK_') ||
        b.startsWith('<blockquote') ||
        b.startsWith('<ul')
      ) {
        return b;
      }
      return `<p style="margin-bottom:1.1rem;line-height:1.75;color:var(--text)">${b}</p>`;
    })
    .join('');

  // 9. Restore inline code
  text = text.replace(/__PMS_INLINE_CODE_(\d+)__/g, (_, idx) => {
    return `<code style="background:rgba(255,255,255,.08);padding:.2rem .45rem;border-radius:4px;font-size:.875em;color:#93c5fd">${escHtml(inlineCodes[idx])}</code>`;
  });

  // 10. Restore code blocks
  text = text.replace(/__PMS_CODE_BLOCK_(\d+)__/g, (_, idx) => {
    const item = codeBlocks[idx];
    return `<pre class="code-block" style="background:#050811;padding:1.1rem;border-radius:10px;overflow-x:auto;margin:1.25rem 0;border:1px solid var(--border);box-shadow:inset 0 2px 4px rgba(0,0,0,.5)"><code style="font-family:monospace;font-size:.875rem;line-height:1.55;color:#e2e8f0">${escHtml(item.code)}</code></pre>`;
  });

  return text;
}

/* ── Scroll-Triggered Reveal via IntersectionObserver ── */
let scrollObserver = null;
function initScrollReveal() {
  if ('IntersectionObserver' in window) {
    scrollObserver = new IntersectionObserver(
      (entries, obs) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.classList.add('in-view');
            obs.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.1, rootMargin: '0px 0px -40px 0px' }
    );
    observeFadeElements();
  } else {
    qsa('.fade-up').forEach(el => el.classList.add('in-view'));
  }
}

function observeFadeElements(ctx = document) {
  if (!scrollObserver) return;
  qsa('.fade-up:not(.in-view)', ctx).forEach(el => scrollObserver.observe(el));
}

/* ── Navbar ── */
function initNavbar() {
  const hamburger = qs('.hamburger');
  const navLinks = qs('.nav-links');
  if (!hamburger || !navLinks) return;

  hamburger.addEventListener('click', () => {
    const isOpen = navLinks.classList.toggle('open');
    hamburger.classList.toggle('active', isOpen);
    hamburger.setAttribute('aria-expanded', isOpen);
  });

  // Close mobile nav on click outside
  document.addEventListener('click', e => {
    if (!navLinks.contains(e.target) && !hamburger.contains(e.target) && navLinks.classList.contains('open')) {
      navLinks.classList.remove('open');
      hamburger.classList.remove('active');
      hamburger.setAttribute('aria-expanded', 'false');
    }
  });

  // Mark current page as active
  const path = window.location.pathname.replace(/\/$/, '') || '/';
  qsa('.nav-links a').forEach(a => {
    const href = a.getAttribute('href').replace(/\/$/, '') || '/';
    if (href === path) a.classList.add('active');
  });
}

/* ── Announcement Banners with Priority Theming ── */
async function loadAnnouncements(container) {
  if (!container) return;
  try {
    const data = await apiFetch('/api/announcements');
    const active = data.filter(a => a.is_active !== false);
    if (!active.length) {
      container.style.display = 'none';
      return;
    }

    container.innerHTML = active
      .map(a => {
        const priority = (a.priority || 'info').toLowerCase();
        let pClass = 'banner-info';
        let icon = '📢';
        if (priority === 'urgent') {
          pClass = 'banner-urgent';
          icon = '🚨';
        } else if (priority === 'warning') {
          pClass = 'banner-warning';
          icon = '⚠️';
        } else if (priority === 'success') {
          pClass = 'banner-success';
          icon = '✅';
        } else if (priority === 'event') {
          pClass = 'banner-event';
          icon = '📅';
        }

        return `
          <div class="banner ${pClass}">
            <div class="container">
              ${icon} <strong>${escHtml(a.message)}</strong>
              ${a.link_url ? `<a href="${safeUrl(a.link_url)}" style="margin-left:.65rem;text-decoration:underline;font-weight:600;color:inherit">${escHtml(a.link_text || 'More info →')}</a>` : ''}
            </div>
          </div>`;
      })
      .join('');
    container.style.display = 'block';
  } catch (_) {
    /* silently ignore */
  }
}

/* ── Animated Number Counters & Live Site Stats ── */
function animateCounter(el, target, duration = 1200) {
  if (!el || isNaN(target)) return;
  const start = 0;
  const startTime = performance.now();

  function update(currentTime) {
    const elapsed = currentTime - startTime;
    const progress = Math.min(elapsed / duration, 1);
    // Smooth ease-out cubic
    const easeProgress = 1 - Math.pow(1 - progress, 3);
    const currentVal = Math.floor(start + (target - start) * easeProgress);
    el.textContent = currentVal;
    if (progress < 1) {
      requestAnimationFrame(update);
    } else {
      el.textContent = target;
    }
  }
  requestAnimationFrame(update);
}

async function loadStats(container) {
  try {
    const s = await apiFetch('/api/stats');
    if (container) {
      const elM = container.querySelector('[data-stat="members"]');
      const elE = container.querySelector('[data-stat="events"]');
      const elY = container.querySelector('[data-stat="years"]');
      const elP = container.querySelector('[data-stat="partners"]');
      if (elM) animateCounter(elM, s.active_members);
      if (elE) animateCounter(elE, s.events_held);
      if (elY) animateCounter(elY, s.years_active);
      if (elP) animateCounter(elP, s.industrial_partners);
    }

    // Populate contact and constitution details wherever mapped
    qsa('[data-stat="constitution"], .pms-constitution-link').forEach(el => {
      if (s.constitution_pdf_url) {
        el.href = safeUrl(s.constitution_pdf_url);
        el.style.display = 'inline-block';
      }
    });
    qsa('[data-stat="office"], .pms-office-location').forEach(el => {
      if (s.office_location) el.textContent = s.office_location;
    });
    qsa('[data-stat="email"], .pms-contact-email').forEach(el => {
      if (s.contact_email) {
        el.textContent = s.contact_email;
        if (el.tagName === 'A') el.href = `mailto:${escHtml(s.contact_email)}`;
      }
    });
  } catch (_) {
    /* ignore */
  }
}

/* ── Skeleton Placeholder Grid (CLS Protection) ── */
function skeletonGrid(count = 3) {
  let html = '';
  for (let i = 0; i < count; i++) {
    html += '<div class="skeleton-card"></div>';
  }
  return html;
}

/* ── Card Renderers with Staggered Cascading Animation ── */
function eventCard(e, idx = 0) {
  const delay = idx * 60;
  return `
    <article class="card fade-up ${e.is_featured ? 'card-featured' : ''}" style="animation-delay:${delay}ms">
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

function teamCard(m, idx = 0) {
  const delay = idx * 60;
  return `
    <article class="card fade-up" style="text-align:center;animation-delay:${delay}ms">
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

function postCard(p, idx = 0) {
  const delay = idx * 60;
  return `
    <article class="card fade-up ${p.is_featured ? 'card-featured' : ''}" style="animation-delay:${delay}ms">
      ${p.cover_image_url ? `<img src="${safeUrl(p.cover_image_url)}" alt="${escHtml(p.title)}" style="width:100%;height:160px;object-fit:cover;border-radius:8px;margin-bottom:1rem">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
        <span class="badge">${escHtml(p.category)}</span>
        ${p.material_formula ? `<span class="tag">🧪 ${escHtml(p.material_formula)}</span>` : ''}
        ${p.is_featured ? '<span class="badge badge-accent">Featured</span>' : ''}
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

function resourceCard(r, idx = 0) {
  const delay = idx * 60;
  return `
    <article class="card fade-up ${r.is_pinned ? 'card-featured' : ''}" style="animation-delay:${delay}ms">
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

/* ── Global Event Registration Modal (Supported on index.html, events.html, etc.) ── */
function openRegistrationModal(eventId, eventTitle) {
  let modal = document.getElementById('reg-modal');
  if (!modal) {
    modal = document.createElement('dialog');
    modal.id = 'reg-modal';
    modal.style.cssText = 'background:var(--bg-card);border:1px solid var(--border);border-radius:var(--radius-lg);color:var(--text);max-width:540px;width:92%;padding:2rem;box-shadow:0 25px 50px -12px rgba(0,0,0,.7);';
    modal.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.25rem">
        <h2 id="reg-modal-title" style="margin:0;font-size:1.35rem">Register for Event</h2>
        <button type="button" id="reg-close-icon" class="btn btn-outline btn-sm" style="padding:.25rem .5rem">✕</button>
      </div>
      <form id="reg-form" novalidate>
        <input type="hidden" id="reg-event-id">
        <div class="form-group">
          <label class="form-label" for="reg-name">Full Name *</label>
          <input id="reg-name" class="form-control" required placeholder="Your full name">
        </div>
        <div class="grid grid-2" style="gap:.75rem">
          <div class="form-group">
            <label class="form-label" for="reg-roll">Roll Number *</label>
            <input id="reg-roll" class="form-control" required placeholder="e.g. 22-DMME-01">
          </div>
          <div class="form-group">
            <label class="form-label" for="reg-batch">Batch *</label>
            <input id="reg-batch" class="form-control" required placeholder="e.g. 2022-2026">
          </div>
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-dept">Academic Department *</label>
          <input id="reg-dept" class="form-control" required placeholder="e.g. Metallurgy & Materials Engineering (DMME)">
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-email">PIEAS Email *</label>
          <input id="reg-email" type="email" class="form-control" required placeholder="student@pieas.edu.pk">
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-phone">Contact Phone</label>
          <input id="reg-phone" class="form-control" placeholder="Optional for SMS updates">
        </div>
        <div id="reg-msg" style="margin-top:1rem"></div>
        <div style="display:flex;justify-content:flex-end;gap:.75rem;margin-top:1.5rem">
          <button type="button" class="btn btn-outline" id="reg-close">Cancel</button>
          <button type="submit" class="btn btn-primary" id="reg-submit">Confirm Registration</button>
        </div>
      </form>
    `;
    document.body.appendChild(modal);
  }

  // Set fields and reset alert
  const titleEl = modal.querySelector('#reg-modal-title');
  if (titleEl) titleEl.textContent = eventTitle ? `Register: ${eventTitle}` : 'Register for Event';
  const idEl = modal.querySelector('#reg-event-id');
  if (idEl) idEl.value = eventId;
  const msgBox = modal.querySelector('#reg-msg');
  if (msgBox) msgBox.innerHTML = '';

  // Ensure submit button is re-enabled and visible on opening modal
  const submitBtn = modal.querySelector('#reg-submit');
  if (submitBtn) {
    submitBtn.disabled = false;
    submitBtn.style.display = '';
  }

  // Bind close buttons and form submit once
  if (!modal.dataset.initialized) {
    modal.dataset.initialized = 'true';
    const closeBtns = modal.querySelectorAll('#reg-close, #reg-close-icon');
    closeBtns.forEach(b => {
      b.onclick = () => modal.close();
    });

    const form = modal.querySelector('#reg-form');
    if (form) {
      form.addEventListener('submit', async ev => {
        ev.preventDefault();
        const curSubmitBtn = modal.querySelector('#reg-submit');
        const curMsgBox = modal.querySelector('#reg-msg');
        const curEventId = (modal.querySelector('#reg-event-id') || {}).value;

        const nameInput = modal.querySelector('#reg-name');
        const rollInput = modal.querySelector('#reg-roll');
        const batchInput = modal.querySelector('#reg-batch');
        const deptInput = modal.querySelector('#reg-dept');
        const emailInput = modal.querySelector('#reg-email');
        const phoneInput = modal.querySelector('#reg-phone');

        const name = (nameInput?.value || '').trim();
        const roll = (rollInput?.value || '').trim();
        const batch = (batchInput?.value || '').trim();
        const dept = (deptInput?.value || '').trim();
        const email = (emailInput?.value || '').trim();
        const phone = (phoneInput?.value || '').trim();

        if (!name || !roll || !batch || !dept || !email) {
          showAlert(curMsgBox, 'Please fill in all required fields.', 'error');
          return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
          showAlert(curMsgBox, 'Please enter a valid email address.', 'error');
          return;
        }

        let isSuccess = false;
        setLoading(curSubmitBtn, true);
        try {
          const result = await apiPost(`/api/events/${curEventId}/register`, {
            student_name: name,
            roll_number: roll,
            batch,
            department: dept,
            email,
            phone: phone || null,
          });
          showAlert(
            curMsgBox,
            `✅ Registration confirmed! Your digital participation verification code is: <br><strong style="font-size:1.15rem;letter-spacing:1px;color:#6ee7b7">${escHtml(result.certificate_code)}</strong><br><small>Save this code to verify your certificate on this portal.</small>`,
            'success',
            true
          );
          form.reset();
          isSuccess = true;
          // Hide and disable submit button to prevent accidental double-registration
          curSubmitBtn.disabled = true;
          curSubmitBtn.style.display = 'none';
        } catch (err) {
          showAlert(curMsgBox, err.message || 'Registration failed.', 'error');
        } finally {
          setLoading(curSubmitBtn, false);
          if (isSuccess) {
            curSubmitBtn.disabled = true;
            curSubmitBtn.style.display = 'none';
          }
        }
      });
    }
  }

  if (!modal.open) {
    modal.showModal();
  }
}

/* ── Lightbox for SEM & Optical Micrographs (gallery.html) ── */
function openLightbox(imageUrl, title, caption = '') {
  let modal = document.getElementById('lightbox-modal');
  if (!modal) {
    modal = document.createElement('dialog');
    modal.id = 'lightbox-modal';
    modal.className = 'lightbox-modal';
    modal.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem">
        <h3 id="lightbox-title" style="font-size:1.15rem;color:var(--text)">Micrograph</h3>
        <button id="lightbox-close-btn" class="btn btn-outline btn-sm" style="padding:.25rem .6rem">✕ Close</button>
      </div>
      <img id="lightbox-img" src="" alt="Micrograph">
      <p id="lightbox-caption" style="color:var(--text-muted);font-size:.9rem;line-height:1.6"></p>
    `;
    document.body.appendChild(modal);
    modal.querySelector('#lightbox-close-btn').onclick = () => modal.close();
  }

  modal.querySelector('#lightbox-img').src = safeUrl(imageUrl);
  modal.querySelector('#lightbox-title').textContent = title || 'Micrograph Detail';
  modal.querySelector('#lightbox-caption').textContent = caption;
  modal.showModal();
}

/* ── Open Article Modal and load full article (with URL Deep Linking) ── */
async function openArticleModal(slug) {
  if (!slug) return;
  let modal = document.getElementById('article-reader-modal');
  if (!modal) {
    modal = document.createElement('dialog');
    modal.id = 'article-reader-modal';
    modal.style.cssText = 'background:var(--bg-card);border:1px solid var(--border);border-radius:var(--radius-lg);color:var(--text);max-width:780px;width:95%;padding:2rem;max-height:85vh;overflow-y:auto;box-shadow:0 25px 60px rgba(0,0,0,.8);';
    document.body.appendChild(modal);
  }

  // Deep linking: update URL query parameter ?article=<slug>
  try {
    const url = new URL(window.location.href);
    if (url.searchParams.get('article') !== slug) {
      url.searchParams.set('article', slug);
      history.pushState(null, '', url.pathname + url.search + url.hash);
    }
  } catch (_) {}

  // Clear query parameter on modal close
  if (!modal.dataset.hasCloseHandler) {
    modal.dataset.hasCloseHandler = 'true';
    modal.addEventListener('close', () => {
      try {
        const url = new URL(window.location.href);
        if (url.searchParams.has('article')) {
          url.searchParams.delete('article');
          const cleanUrl = url.pathname + (url.search ? url.search : '') + url.hash;
          history.pushState(null, '', cleanUrl);
        }
      } catch (_) {}
    });
  }

  modal.innerHTML = `
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.5rem">
      <span class="badge">Technical Article</span>
      <button id="close-article-modal" class="btn btn-outline btn-sm" style="padding:.25rem .6rem">✕ Close</button>
    </div>
    <div id="article-reader-content"><div class="spinner"></div></div>
  `;
  if (!modal.open) {
    modal.showModal();
  }
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
      <div style="display:flex;gap:1.5rem;font-size:.85rem;color:var(--text-muted);border-bottom:1px solid var(--border);padding-bottom:1rem;margin-bottom:1.5rem;flex-wrap:wrap">
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

/* ── Event delegation for Global Actions ── */
document.addEventListener('click', ev => {
  // Read article modal
  const readBtn = ev.target.closest('.btn-read-article');
  if (readBtn && readBtn.dataset.slug) {
    ev.preventDefault();
    openArticleModal(readBtn.dataset.slug);
    return;
  }

  // Register for event modal (works on homepage featured cards and events.html)
  const regBtn = ev.target.closest('.btn-open-reg');
  if (regBtn && regBtn.dataset.eventId) {
    ev.preventDefault();
    openRegistrationModal(regBtn.dataset.eventId, regBtn.dataset.eventTitle);
    return;
  }

  // Micrograph lightbox
  const lbTrigger = ev.target.closest('.micrograph-lightbox-trigger');
  if (lbTrigger) {
    ev.preventDefault();
    openLightbox(
      lbTrigger.dataset.imgUrl,
      lbTrigger.dataset.imgTitle,
      lbTrigger.dataset.imgCaption
    );
  }
});

/* ── Popstate handler for back/forward navigation with article deep links ── */
window.addEventListener('popstate', () => {
  const articleSlug = new URLSearchParams(window.location.search).get('article');
  const modal = document.getElementById('article-reader-modal');
  if (articleSlug) {
    openArticleModal(articleSlug);
  } else if (modal && modal.open) {
    modal.close();
  }
});

/* ── On DOM ready ── */
document.addEventListener('DOMContentLoaded', () => {
  initNavbar();
  initScrollReveal();
  loadAnnouncements(qs('#announcement-banner'));
  loadStats(qs('#stats-strip'));

  // Auto-open article modal if ?article=<slug> is in URL
  const articleSlug = new URLSearchParams(window.location.search).get('article');
  if (articleSlug) {
    openArticleModal(articleSlug);
  }
});

/* Export for use across all page scripts */
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
  openRegistrationModal,
  openLightbox,
  skeletonGrid,
  showAlert,
  setLoading,
  observeFadeElements,
  eventCard,
  teamCard,
  postCard,
  resourceCard,
};
