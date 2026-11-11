// @ts-check
/**
 * PMS – PIEAS Materials Society
 * Shared Modern Client-Side JS Architecture
 * frontend/static/assets/app.js
 */

/**
 * @typedef {HTMLDialogElement & { _returnFocusEl?: HTMLElement | null, _closingFromPopstate?: boolean }} PmsDialog
 * @typedef {HTMLButtonElement & { dataset: DOMStringMap }} PmsButton
 */

(() => {
  'use strict';

  document.documentElement.classList.add('js');
  document.documentElement.classList.remove('no-js');

  /* ── API base ── */
  const API = ''; // Same-origin: requests go to /api/*

  /**
   * Generic API fetch wrapper.
   * Returns parsed JSON or throws an Error with an HTTP `status` property.
   * @param {string} path
   * @param {RequestInit} [options]
   * @returns {Promise<any>}
   */
  async function apiFetch(path, options = {}) {
    const { headers: customHeaders, ...restOptions } = options;
    const mergedHeaders = {
      'Content-Type': 'application/json',
      'X-Requested-With': 'PMS-Client',
      ...(customHeaders || {}),
    };
    const res = await fetch(`${API}${path}`, {
      credentials: 'include', // send cookies for authenticated requests
      ...restOptions,
      headers: mergedHeaders,
    });

    if (!res.ok) {
      let msg = `HTTP ${res.status}`;
      try {
        const body = await res.json();
        msg = body.error || msg;
      } catch (_) {}
      throw Object.assign(new Error(msg), { status: res.status });
    }

    const text = await res.text();
    if (!text || !text.trim()) {
      return {};
    }
    try {
      return JSON.parse(text);
    } catch (_) {
      throw new Error('Invalid JSON response from server');
    }
  }

  /**
   * POST JSON helper.
   * @param {string} path
   * @param {unknown} data
   * @returns {Promise<any>}
   */
  async function apiPost(path, data) {
    return apiFetch(path, { method: 'POST', body: JSON.stringify(data) });
  }

  /**
   * PUT JSON helper.
   * @param {string} path
   * @param {unknown} data
   * @returns {Promise<any>}
   */
  async function apiPut(path, data) {
    return apiFetch(path, { method: 'PUT', body: JSON.stringify(data) });
  }

  /**
   * DELETE helper.
   * @param {string} path
   * @returns {Promise<any>}
   */
  async function apiDelete(path) {
    return apiFetch(path, { method: 'DELETE' });
  }

  /* ── SVG Icon Registry (Lucide Craft, 1.5 Stroke, 16px Base) ── */
  /** @type {Record<string, string>} */
  const iconPaths = {
    atom: '<circle cx="12" cy="12" r="1"/><ellipse cx="12" cy="12" rx="10" ry="4"/><ellipse cx="12" cy="12" rx="10" ry="4" transform="rotate(60 12 12)"/><ellipse cx="12" cy="12" rx="10" ry="4" transform="rotate(120 12 12)"/>',
    microscope:
      '<path d="M6 18h8"/><path d="M3 22h18"/><path d="M14 22a7 7 0 1 0 0-14h-1"/><path d="M9 14h2"/><path d="M9 12a2 2 0 0 1-2-2V6h6v4a2 2 0 0 1-2 2Z"/><path d="M12 6V3a1 1 0 0 0-1-1H9a1 1 0 0 0-1 1v3"/>',
    award: '<circle cx="12" cy="8" r="6"/><path d="M15.477 12.89 17 22l-5-3-5 3 1.523-9.11"/>',
    building:
      '<rect width="16" height="20" x="4" y="2" rx="2" ry="2"/><path d="M9 22v-4h6v4"/><path d="M8 6h.01"/><path d="M16 6h.01"/><path d="M12 6h.01"/><path d="M12 10h.01"/><path d="M12 14h.01"/><path d="M16 10h.01"/><path d="M16 14h.01"/><path d="M8 10h.01"/><path d="M8 14h.01"/>',
    calendar:
      '<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/>',
    clock: '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>',
    'map-pin':
      '<path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/><circle cx="12" cy="10" r="3"/>',
    mail: '<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>',
    user: '<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>',
    users:
      '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>',
    layers:
      '<path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"/><path d="m22 12.5-9.17 4.16a2 2 0 0 1-1.66 0L2 12.5"/><path d="m22 17.5-9.17 4.16a2 2 0 0 1-1.66 0L2 17.5"/>',
    cpu: '<rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2"/><path d="M15 20v2"/><path d="M2 15h2"/><path d="M2 9h2"/><path d="M20 15h2"/><path d="M20 9h2"/><path d="M9 2v2"/><path d="M9 20v2"/>',
    'file-text':
      '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 9H8"/><path d="M16 13H8"/><path d="M16 17H8"/>',
    'check-circle':
      '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>',
    x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
    'alert-triangle':
      '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>',
    bell: '<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>',
    copy: '<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
    external:
      '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
    search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
    'graduation-cap':
      '<path d="M21.42 10.922a1 1 0 0 0-.019-1.838L12.83 5.18a2 2 0 0 0-1.66 0L2.6 9.08a1 1 0 0 0 0 1.832l8.57 3.908a2 2 0 0 0 1.66 0z"/><path d="M22 10v6"/><path d="M6 12.5V16a6 3 0 0 0 12 0v-3.5"/>',
    download:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>',
    moon: '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
    'chevron-left': '<path d="m15 18-6-6 6-6"/>',
    'chevron-right': '<path d="m9 18 6-6-6-6"/>',
    pause:
      '<rect x="14" y="4" width="4" height="16" rx="1"/><rect x="6" y="4" width="4" height="16" rx="1"/>',
    play: '<polygon points="6 3 20 12 6 21 6 3"/>',
  };

  /**
   * Render an inline SVG icon from the Lucide subset.
   * @param {string} name
   * @param {number} [size]
   * @param {string} [className]
   * @returns {string}
   */
  function icon(name, size = 16, className = 'icon') {
    const d = iconPaths[name] || iconPaths['file-text'];
    return `<svg class="${className}" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${d}</svg>`;
  }

  /* ── DOM utilities ── */
  /**
   * Query a single element.
   * @param {string} sel
   * @param {ParentNode | null} [ctx]
   * @returns {HTMLElement | null}
   */
  function qs(sel, ctx = document) {
    if (!ctx) return null;
    return /** @type {HTMLElement | null} */ (ctx.querySelector(sel));
  }

  /**
   * Query all matching elements as an array.
   * @param {string} sel
   * @param {ParentNode | null} [ctx]
   * @returns {HTMLElement[]}
   */
  function qsa(sel, ctx = document) {
    if (!ctx) return [];
    return /** @type {HTMLElement[]} */ ([...ctx.querySelectorAll(sel)]);
  }

  /**
   * Check whether an email string has valid basic syntax.
   * @param {string} str
   * @returns {boolean}
   */
  function isEmail(str) {
    return typeof str === 'string' && /^[^\s@<>"]+@[^\s@<>"]+\.[^\s@<>"]+$/.test(str.trim());
  }

  /**
   * Render an accessible, localized empty-state block.
   * @param {string} msg
   * @returns {string}
   */
  function emptyState(msg) {
    return `<div class="card empty-state" style="text-align:center;color:var(--text-muted);grid-column:1/-1;padding:2.5rem"><p>${escHtml(msg)}</p></div>`;
  }

  /**
   * Render an accessible inline error-state block.
   * @param {string} msg
   * @returns {string}
   */
  function errorState(msg) {
    return `<div class="alert alert-error" role="alert" style="grid-column:1/-1">${escHtml(msg)}</div>`;
  }

  /**
   * Render an inline status or error alert inside a container.
   * @param {Element | null} container
   * @param {string} message
   * @param {string} [type]
   * @param {boolean} [isHtml]
   */
  function showAlert(container, message, type = 'error', isHtml = false) {
    if (!container) return;
    const roleAttr = type === 'error' ? 'alert' : 'status';
    container.innerHTML = `<div class="alert alert-${type}" role="${roleAttr}">${isHtml ? message : escHtml(message)}</div>`;
  }

  /**
   * Clear `aria-invalid` attributes as soon as the user modifies a field.
   * @param {HTMLFormElement | null} formEl
   */
  function clearInvalidOnInput(formEl) {
    if (!formEl || formEl.dataset.invalidClearBound) return;
    formEl.dataset.invalidClearBound = 'true';
    /** @param {Event} ev */
    const clear = ev => {
      if (ev.target instanceof HTMLElement && ev.target.hasAttribute('aria-invalid')) {
        ev.target.removeAttribute('aria-invalid');
      }
    };
    formEl.addEventListener('input', clear);
    formEl.addEventListener('change', clear);
  }

  /**
   * Mark invalid form controls with `aria-invalid="true"` and focus the first one.
   * @param {HTMLFormElement | null} formEl
   * @param {Array<Element | null | undefined>} invalidEls
   * @returns {boolean}
   */
  function reportInvalidFields(formEl, invalidEls) {
    if (formEl) {
      formEl.querySelectorAll('[aria-invalid]').forEach(el => el.removeAttribute('aria-invalid'));
      clearInvalidOnInput(formEl);
    }
    const list = /** @type {HTMLElement[]} */ ((invalidEls || []).filter(Boolean));
    list.forEach(el => el.setAttribute('aria-invalid', 'true'));
    if (list.length > 0 && typeof list[0].focus === 'function') {
      list[0].focus();
    }
    return list.length > 0;
  }

  /**
   * Escape HTML special characters for safe interpolation.
   * @param {unknown} str
   * @returns {string}
   */
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
   * Returns raw unescaped string for assignment to DOM properties (.href, .src).
   * @param {unknown} url
   * @returns {string}
   */
  function safeUrlRaw(url) {
    if (!url || typeof url !== 'string') return '#';
    const trimmed = url.trim();
    const lower = trimmed.toLowerCase();
    if (
      lower.startsWith('https://') ||
      lower.startsWith('http://') ||
      lower.startsWith('mailto:') ||
      (trimmed.startsWith('/') && !trimmed.startsWith('//')) ||
      trimmed.startsWith('#')
    ) {
      return trimmed;
    }
    return '#';
  }

  /**
   * HTML-escaped safeUrl for interpolation into HTML template literals.
   * @param {unknown} url
   * @returns {string}
   */
  function safeUrl(url) {
    return escHtml(safeUrlRaw(url));
  }

  /**
   * Toggle disabled/loading text state on a submit button.
   * @param {HTMLElement | null} el
   * @param {boolean} loading
   */
  function setLoading(el, loading) {
    if (!el) return;
    const btn = /** @type {HTMLButtonElement} */ (el);
    if (loading) {
      btn.disabled = true;
      if (!btn.dataset.orig) {
        btn.dataset.orig = btn.textContent || '';
      }
      btn.textContent = 'Loading…';
    } else {
      btn.disabled = false;
      if (btn.dataset.orig !== undefined) {
        btn.textContent = btn.dataset.orig;
        delete btn.dataset.orig;
      } else {
        btn.textContent = 'Submit';
      }
    }
  }

  /**
   * Format an ISO or SQLite timestamp/date string to human-readable form.
   * @param {unknown} iso
   * @returns {string}
   */
  function fmtDate(iso) {
    if (!iso) return '';
    const str = String(iso).trim();
    if (/^\d{4}-\d{2}-\d{2}$/.test(str)) {
      const d = new Date(`${str}T00:00:00Z`);
      if (isNaN(d.getTime())) return str;
      return d.toLocaleDateString('en-PK', {
        timeZone: 'UTC',
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    }
    const normalized = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}(:\d{2}(\.\d+)?)?$/.test(str)
      ? `${str.replace(' ', 'T')}Z`
      : str.replace(' ', 'T');
    const d = new Date(normalized);
    if (isNaN(d.getTime())) return str;
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
   * or C pointers (*ptr). Supports ordered lists, unordered lists, rules, and images.
   * @param {string | null | undefined} md
   * @returns {string}
   */
  function renderMarkdown(md) {
    if (!md) return '';

    const tok = Math.random().toString(36).slice(2, 10) + Date.now().toString(36);
    /** @type {Array<{ lang: string, code: string }>} */
    const codeBlocks = [];
    /** @type {string[]} */
    const inlineCodes = [];
    /** @type {Array<{ alt: string, target: string }>} */
    const images = [];
    /** @type {Array<{ linkText: string, target: string }>} */
    const links = [];

    // Strip NUL bytes so internal placeholder delimiters cannot collide with article text
    let text = String(md).split('\u0000').join('');

    // 1. Protect fenced code blocks (```lang ... ```)
    text = text.replace(/```([a-zA-Z0-9_-]*)\n?([\s\S]*?)```/g, (_, lang, code) => {
      const idx = codeBlocks.length;
      codeBlocks.push({ lang, code: code.trim() });
      return `\n\n\u0000PMS_${tok}_CODE_BLOCK_${idx}\u0000\n\n`;
    });

    // 2. Protect inline code (`code`)
    text = text.replace(/`([^`\n]+)`/g, (_, code) => {
      const idx = inlineCodes.length;
      inlineCodes.push(code);
      return `\u0000PMS_${tok}_INLINE_CODE_${idx}\u0000`;
    });

    // Escape HTML characters in markdown body to prevent XSS
    text = escHtml(text);

    // 3. Protect Images ![alt](url)
    text = text.replace(/!\[([^\]]*)\]\(((?:[^\s()]+|\([^\s()]*\))+)\)/g, (_, alt, url) => {
      const idx = images.length;
      const cleanUrl = url.replace(/&amp;/g, '&');
      const target = safeUrlRaw(cleanUrl);
      images.push({ alt, target: escHtml(target) });
      return `\n\n\u0000PMS_${tok}_IMAGE_${idx}\u0000\n\n`;
    });

    // 4. Protect Links [text](url) - handles parentheses in URL & prevents bold/italic asterisks from corrupting URLs
    text = text.replace(/\[([^\]]+)\]\(((?:[^\s()]+|\([^\s()]*\))+)\)/g, (_, linkText, url) => {
      const idx = links.length;
      const cleanUrl = url.replace(/&amp;/g, '&');
      const target = safeUrl(cleanUrl);
      links.push({ linkText, target });
      return `\u0000PMS_${tok}_LINK_${idx}\u0000`;
    });

    // 5. Horizontal Rules (--- or *** or ___)
    text = text.replace(
      /^(?:---|—{3,}|\*\*\*|___)\s*$/gm,
      '\n\n<hr style="border:0;border-top:1px solid var(--border);margin:2rem 0">\n\n'
    );

    // 6. Headings (shifted h1->h2, h2->h3, h3->h4 because article modal already contains an h1)
    text = text.replace(
      /^### (.*$)/gim,
      '\n\n<h4 style="font-size:1.05rem;margin:1.25rem 0 .45rem;color:var(--text)">$1</h4>\n\n'
    );
    text = text.replace(
      /^## (.*$)/gim,
      '\n\n<h3 style="font-size:1.25rem;margin:1.5rem 0 .55rem;color:var(--text)">$1</h3>\n\n'
    );
    text = text.replace(
      /^# (.*$)/gim,
      '\n\n<h2 style="font-size:1.45rem;margin:1.75rem 0 .7rem;color:var(--text)">$1</h2>\n\n'
    );

    // 7. Bold & Italics
    text = text.replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>');
    text = text.replace(
      /(^|[^\w*])\*([^*\n\s](?:[^*\n]*?[^*\n\s])?)\*(?=[^\w*]|$)/g,
      '$1<em>$2</em>'
    );

    // 8. Blockquotes (group consecutive > or &gt; lines into a single blockquote)
    text = text.replace(/(?:^(?:>|&gt;)[ \t]?(.*)(?:\r?\n|$))+/gm, match => {
      const inner = match
        .trim()
        .split(/\r?\n/)
        .map(line => line.replace(/^(?:>|&gt;)[ \t]?/, ''))
        .join('<br>');
      return `\n\n<blockquote style="border-left:3px solid var(--accent);padding-left:1rem;margin:1.2rem 0;color:var(--text-muted);font-style:italic">${inner}</blockquote>\n\n`;
    });

    // 9. Group consecutive ordered list items into <ol> without orphan <li>
    text = text.replace(/(?:^\s*\d+\.\s+.*(?:\r?\n|$))+/gm, match => {
      const items = match
        .trim()
        .split(/\r?\n/)
        .map(line => {
          const itemContent = line.replace(/^\s*\d+\.\s+/, '');
          return `<li style="margin-bottom:.4rem">${itemContent}</li>`;
        })
        .join('');
      return `\n\n<ol style="margin:1rem 0 1rem 1.75rem;color:var(--text);line-height:1.6">${items}</ol>\n\n`;
    });

    // 10. Group consecutive unordered list items into <ul> without orphan <li>
    text = text.replace(/(?:^\s*[-*]\s+.*(?:\r?\n|$))+/gm, match => {
      const items = match
        .trim()
        .split(/\r?\n/)
        .map(line => {
          const itemContent = line.replace(/^\s*[-*]\s+/, '');
          return `<li style="margin-bottom:.4rem">${itemContent}</li>`;
        })
        .join('');
      return `\n\n<ul style="margin:1rem 0 1rem 1.75rem;color:var(--text);line-height:1.6">${items}</ul>\n\n`;
    });

    // 11. Paragraphs
    const cbPrefix = `\u0000PMS_${tok}_CODE_BLOCK_`;
    const imgPrefix = `\u0000PMS_${tok}_IMAGE_`;
    const blocks = text.split(/\n\s*\n/);
    text = blocks
      .map(b => {
        b = b.trim();
        if (!b) return '';
        if (
          b.startsWith('<h') ||
          b.startsWith(cbPrefix) ||
          b.startsWith(imgPrefix) ||
          b.startsWith('<blockquote') ||
          b.startsWith('<ul') ||
          b.startsWith('<ol') ||
          b.startsWith('<hr')
        ) {
          return b;
        }
        return `<p style="margin-bottom:1.1rem;line-height:1.75;color:var(--text)">${b}</p>`;
      })
      .join('');

    // 12. Restore links (supporting inline bold/italics in link text) with guarded placeholder restoration
    text = text.replace(new RegExp(`\u0000PMS_${tok}_LINK_(\\d+)\u0000`, 'g'), (match, idx) => {
      const item = links[Number(idx)];
      if (!item) return match;
      let formattedText = item.linkText
        .replace(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>')
        .replace(/(^|[^\w*])\*([^*\n\s](?:[^*\n]*?[^*\n\s])?)\*(?=[^\w*]|$)/g, '$1<em>$2</em>');
      return `<a href="${item.target}" target="_blank" rel="noopener" style="color:var(--accent);text-decoration:underline">${formattedText}</a>`;
    });

    // 13. Restore images
    text = text.replace(new RegExp(`\u0000PMS_${tok}_IMAGE_(\\d+)\u0000`, 'g'), (match, idx) => {
      const item = images[Number(idx)];
      if (!item) return match;
      return `<img src="${item.target}" alt="${item.alt}" style="max-width:100%;height:auto;border-radius:8px;margin:1rem 0;display:block" loading="lazy" decoding="async">`;
    });

    // 14. Restore inline code
    text = text.replace(
      new RegExp(`\u0000PMS_${tok}_INLINE_CODE_(\\d+)\u0000`, 'g'),
      (match, idx) => {
        const item = inlineCodes[Number(idx)];
        if (item === undefined) return match;
        return `<code style="background:var(--code-bg);padding:.2rem .45rem;border-radius:4px;font-size:.875em;color:var(--code-fg)">${escHtml(item)}</code>`;
      }
    );

    // 15. Restore code blocks
    text = text.replace(
      new RegExp(`\u0000PMS_${tok}_CODE_BLOCK_(\\d+)\u0000`, 'g'),
      (match, idx) => {
        const item = codeBlocks[Number(idx)];
        if (!item) return match;
        return `<pre class="code-block" style="background:var(--bg-deep);padding:1.1rem;border-radius:10px;overflow-x:auto;margin:1.25rem 0;border:1px solid var(--border)"><code style="font-family:var(--font-mono);font-size:.875rem;line-height:1.55;color:var(--text)">${escHtml(item.code)}</code></pre>`;
      }
    );

    return text;
  }

  /* ── Scroll-Triggered Reveal via IntersectionObserver ── */
  /** @type {IntersectionObserver | null} */
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

  /**
   * Observe newly rendered `.fade-up` elements within a root node.
   * @param {ParentNode} [ctx]
   */
  function observeFadeElements(ctx = document) {
    if (!scrollObserver) return;
    qsa('.fade-up:not(.in-view)', ctx).forEach(el => scrollObserver?.observe(el));
  }

  function prefersReducedMotion() {
    return Boolean(
      window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches
    );
  }

  /* ── Navbar ── */
  function initNavbar() {
    const hamburger = qs('.hamburger');
    const navLinks = qs('.nav-links');
    if (!hamburger || !navLinks) return;

    if (!navLinks.id) navLinks.id = 'nav-links';
    hamburger.setAttribute('aria-controls', navLinks.id);

    /** @param {boolean} [returnFocus] */
    function closeNav(returnFocus = false) {
      navLinks?.classList.remove('open');
      hamburger?.classList.remove('active');
      hamburger?.setAttribute('aria-expanded', 'false');
      if (returnFocus) hamburger?.focus();
    }

    hamburger.addEventListener('click', () => {
      const isOpen = navLinks.classList.toggle('open');
      hamburger.classList.toggle('active', isOpen);
      hamburger.setAttribute('aria-expanded', String(isOpen));
    });

    // Close mobile nav on click outside
    document.addEventListener('click', e => {
      const target = e.target instanceof Node ? e.target : null;
      if (
        target &&
        !navLinks.contains(target) &&
        !hamburger.contains(target) &&
        navLinks.classList.contains('open')
      ) {
        closeNav(false);
      }
    });

    // Close mobile nav on Escape and return focus to hamburger
    document.addEventListener('keydown', e => {
      if (e.key === 'Escape' && navLinks.classList.contains('open')) {
        closeNav(true);
      }
    });

    // Mark current page as active
    const path = window.location.pathname.replace(/\/$/, '') || '/';
    qsa('.nav-links a').forEach(a => {
      const href = (a.getAttribute('href') || '').replace(/\/$/, '') || '/';
      if (href === path || (href === '/blog' && path.startsWith('/blog/'))) {
        a.classList.add('active');
        a.setAttribute('aria-current', 'page');
      }
    });
  }

  /* ── Theme Switcher (Dark / Light Mode) ── */
  function initTheme() {
    const current = document.documentElement.getAttribute('data-theme') || 'dark';
    updateThemeUI(current);

    qsa('#theme-toggle, .theme-toggle-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const active = document.documentElement.getAttribute('data-theme') || 'dark';
        const next = active === 'dark' ? 'light' : 'dark';
        document.documentElement.setAttribute('data-theme', next);
        try {
          localStorage.setItem('pms_theme', next);
        } catch (_) {}
        updateThemeUI(next);
      });
    });

    if (window.matchMedia) {
      window.matchMedia('(prefers-color-scheme: light)').addEventListener('change', e => {
        try {
          if (!localStorage.getItem('pms_theme')) {
            const next = e.matches ? 'light' : 'dark';
            document.documentElement.setAttribute('data-theme', next);
            updateThemeUI(next);
          }
        } catch (_) {}
      });
    }
  }

  /** @param {string} theme */
  function updateThemeUI(theme) {
    const isLight = theme === 'light';
    qsa('#theme-toggle, .theme-toggle-btn').forEach(btn => {
      btn.innerHTML = icon(isLight ? 'moon' : 'sun', 16);
      const label = isLight ? 'Switch to dark mode' : 'Switch to light mode';
      btn.setAttribute('aria-label', label);
      btn.setAttribute('title', label);
    });
  }

  /* ── Sliding Announcement Carousel with Dismiss Option ── */
  /**
   * Load announcements into the top banner carousel.
   * @param {HTMLElement | null} container
   */
  async function loadAnnouncements(container) {
    if (!container) return;
    try {
      if (sessionStorage.getItem('pms_announcement_dismissed') === 'true') {
        container.style.display = 'none';
        return;
      }
    } catch (_) {}

    try {
      const data = await apiFetch('/api/announcements');
      const active = /** @type {any[]} */ (data || []).filter(a => a.is_active !== false);
      if (!active.length) {
        container.style.display = 'none';
        return;
      }

      const slidesHtml = active
        .map((a, idx) => {
          const priority = (a.priority || 'info').toLowerCase();
          let badgeLabel = 'Notice';
          let badgeIcon = 'bell';
          let badgeClass = 'badge-info';

          if (priority === 'urgent') {
            badgeLabel = 'Urgent';
            badgeIcon = 'alert-triangle';
            badgeClass = 'badge-danger';
          } else if (priority === 'warning') {
            badgeLabel = 'Notice';
            badgeIcon = 'alert-triangle';
            badgeClass = 'badge-warning';
          } else if (priority === 'event') {
            badgeLabel = 'Event';
            badgeIcon = 'calendar';
            badgeClass = 'badge-accent';
          } else if (priority === 'success') {
            badgeLabel = 'Update';
            badgeIcon = 'check-circle';
            badgeClass = 'badge-success';
          }

          return `
          <div class="announcement-slide" data-index="${idx}">
            <span class="announcement-badge ${badgeClass}">${icon(badgeIcon, 12)} <span>${badgeLabel}</span></span>
            <span class="announcement-text">${escHtml(a.message)}</span>
            ${a.link_url ? `<a href="${safeUrl(a.link_url)}" class="announcement-link">${escHtml(a.link_text || 'Details')} <span aria-hidden="true">&rarr;</span></a>` : ''}
          </div>`;
        })
        .join('');

      const hasMultiple = active.length > 1;

      container.className = 'announcement-bar';
      container.innerHTML = `
      <div class="container announcement-bar-inner">
        <div class="announcement-slider" id="announcement-slider">
          <div class="announcement-track" id="announcement-track">
            ${slidesHtml}
          </div>
        </div>
        <div class="announcement-controls">
          ${
            hasMultiple
              ? `<button type="button" class="announcement-nav-btn" id="announcement-prev" aria-label="Previous announcement">${icon('chevron-left', 14)}</button>
                 <span class="announcement-counter" id="announcement-counter" aria-live="polite">1 / ${active.length}</span>
                 <button type="button" class="announcement-nav-btn" id="announcement-next" aria-label="Next announcement">${icon('chevron-right', 14)}</button>
                 <button type="button" class="announcement-pause-btn" id="announcement-pause" aria-label="Pause announcements" aria-pressed="false" title="Pause announcements">${icon('pause', 14)}</button>`
              : ''
          }
          <button type="button" class="announcement-close-btn" id="announcement-close" aria-label="Dismiss announcement" title="Dismiss announcement">
            ${icon('x', 14)}
          </button>
        </div>
      </div>`;

      container.style.display = 'block';

      let currentIndex = 0;
      const track = qs('#announcement-track', container);
      const slides = qsa('.announcement-slide', track);
      const counter = qs('#announcement-counter', container);
      const pauseBtn = qs('#announcement-pause', container);
      let isPaused = prefersReducedMotion();
      /** @type {ReturnType<typeof setInterval> | null} */
      let slideTimer = null;

      const updatePauseButton = () => {
        if (!pauseBtn) return;
        pauseBtn.innerHTML = icon(isPaused ? 'play' : 'pause', 14);
        const label = isPaused ? 'Play announcements' : 'Pause announcements';
        pauseBtn.setAttribute('aria-label', label);
        pauseBtn.setAttribute('title', label);
        pauseBtn.setAttribute('aria-pressed', String(isPaused));
      };

      /** @param {number} idx */
      const goToSlide = idx => {
        if (!track) return;
        currentIndex = (idx + active.length) % active.length;
        track.style.transform = `translateX(-${currentIndex * 100}%)`;
        slides.forEach((slide, i) => {
          const isCurrent = i === currentIndex;
          slide.setAttribute('aria-hidden', isCurrent ? 'false' : 'true');
          slide.inert = !isCurrent;
        });
        if (counter) counter.textContent = `${currentIndex + 1} / ${active.length}`;
      };

      const stopTimer = () => {
        if (slideTimer) {
          clearInterval(slideTimer);
          slideTimer = null;
        }
      };

      const startTimer = () => {
        if (!hasMultiple || isPaused || prefersReducedMotion()) return;
        stopTimer();
        slideTimer = setInterval(() => {
          goToSlide(currentIndex + 1);
        }, 5000);
      };

      goToSlide(0);

      if (hasMultiple) {
        updatePauseButton();
        const prevBtn = qs('#announcement-prev', container);
        const nextBtn = qs('#announcement-next', container);
        if (prevBtn) {
          prevBtn.addEventListener('click', e => {
            e.stopPropagation();
            stopTimer();
            goToSlide(currentIndex - 1);
            startTimer();
          });
        }
        if (nextBtn) {
          nextBtn.addEventListener('click', e => {
            e.stopPropagation();
            stopTimer();
            goToSlide(currentIndex + 1);
            startTimer();
          });
        }
        if (pauseBtn) {
          pauseBtn.addEventListener('click', e => {
            e.stopPropagation();
            isPaused = !isPaused;
            if (isPaused) {
              stopTimer();
            } else {
              startTimer();
            }
            updatePauseButton();
          });
        }

        container.addEventListener('mouseenter', stopTimer);
        container.addEventListener('mouseleave', startTimer);
        container.addEventListener('focusin', stopTimer);
        container.addEventListener('focusout', e => {
          if (e.relatedTarget instanceof Node && container.contains(e.relatedTarget)) return;
          startTimer();
        });
        startTimer();
      }

      const closeBtn = qs('#announcement-close', container);
      if (closeBtn) {
        closeBtn.addEventListener('click', e => {
          e.stopPropagation();
          stopTimer();
          container.classList.add('announcement-dismissing');
          try {
            sessionStorage.setItem('pms_announcement_dismissed', 'true');
          } catch (_) {}
          setTimeout(() => {
            container.style.display = 'none';
          }, 220);
        });
      }
    } catch (_) {
      container.style.display = 'none';
    }
  }

  /* ── Animated Number Counters & Live Site Stats ── */
  /**
   * Animate an integer counter up to a target value.
   * @param {Element | null} el
   * @param {number} target
   * @param {number} [duration]
   */
  function animateCounter(el, target, duration = 1200) {
    if (!el || isNaN(target)) return;
    if (prefersReducedMotion()) {
      el.textContent = String(target);
      return;
    }
    const start = 0;
    const startTime = performance.now();

    /** @param {number} currentTime */
    function update(currentTime) {
      if (!el) return;
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);
      // Smooth ease-out cubic
      const easeProgress = 1 - Math.pow(1 - progress, 3);
      const currentVal = Math.floor(start + (target - start) * easeProgress);
      el.textContent = String(currentVal);
      if (progress < 1) {
        requestAnimationFrame(update);
      } else {
        el.textContent = String(target);
      }
    }
    requestAnimationFrame(update);
  }

  /**
   * Load site statistics and contact metadata.
   * @param {Element | null} container
   */
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
      qsa('.pms-constitution-link').forEach(el => {
        if (s.constitution_pdf_url && el instanceof HTMLAnchorElement) {
          el.href = safeUrlRaw(s.constitution_pdf_url);
          el.style.display = 'inline-block';
        }
      });
      qsa('.pms-office-location').forEach(el => {
        if (s.office_location) el.textContent = s.office_location;
      });
      qsa('.pms-contact-email').forEach(el => {
        if (s.contact_email) {
          const rawEmail = String(s.contact_email).trim();
          el.textContent = rawEmail;
          if (el instanceof HTMLAnchorElement && isEmail(rawEmail)) {
            el.href = `mailto:${rawEmail}`;
          }
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
      html += '<div class="skeleton-card" aria-hidden="true"></div>';
    }
    return html;
  }

  /* ── Card Renderers with Staggered Cascading Animation ── */
  /**
   * @param {any} e
   * @param {number} [idx]
   * @returns {string}
   */
  function eventCard(e, idx = 0) {
    const delay = Math.min(idx, 8) * 60;
    return `
    <article class="card fade-up ${e.is_featured ? 'card-featured' : ''}" style="transition-delay:${delay}ms">
      ${e.poster_url ? `<img src="${safeUrl(e.poster_url)}" alt="" width="400" height="180" loading="lazy" decoding="async" style="width:100%;height:180px;object-fit:cover;border-radius:8px;margin-bottom:16px;">` : ''}
      <div style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:12px;">
        <span class="badge">${escHtml(e.category)}</span>
        ${e.is_past ? '<span class="badge" style="color:var(--text-muted)">Past</span>' : '<span class="badge badge-success">Upcoming</span>'}
        ${e.is_featured ? '<span class="badge badge-accent">Featured</span>' : ''}
      </div>
      <h3 style="margin-bottom:8px">${escHtml(e.title)}</h3>
      ${e.speaker_or_organizer ? `<p style="color:var(--text);font-size:13px;font-weight:500;margin-bottom:6px;display:flex;align-items:center;gap:6px">${icon('user', 14)} ${escHtml(e.speaker_or_organizer)}</p>` : ''}
      <p style="color:var(--text-muted);font-size:13px;margin-bottom:6px;display:flex;align-items:center;gap:6px">${icon('calendar', 14)} <span class="tabular-nums">${escHtml(fmtDate(e.date))}${e.time_range ? ' · ' + escHtml(e.time_range) : ''}</span></p>
      ${e.venue ? `<p style="color:var(--text-muted);font-size:13px;margin-bottom:12px;display:flex;align-items:center;gap:6px">${icon('map-pin', 14)} ${escHtml(e.venue)}</p>` : ''}
      <p style="color:var(--text-secondary);font-size:14px;margin-bottom:12px;line-height:1.5">${escHtml(e.short_description || '')}</p>
      ${
        e.full_summary
          ? `
        <details style="margin:12px 0;font-size:13.5px;color:var(--text-secondary);cursor:pointer">
          <summary style="color:var(--text);font-weight:500">View Agenda Details</summary>
          <div style="padding-top:8px;line-height:1.6">${escHtml(e.full_summary)}</div>
        </details>`
          : ''
      }
      <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:auto;padding-top:16px">
        ${e.registration_link ? `<a href="${safeUrl(e.registration_link)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm">${icon('external', 13)} Details</a>` : ''}
        ${!e.is_past ? `<button class="btn btn-primary btn-sm btn-open-reg" data-event-id="${e.id}" data-event-title="${escHtml(e.title)}">Register</button>` : ''}
      </div>
    </article>`;
  }

  /**
   * @param {any} m
   * @param {number} [idx]
   * @returns {string}
   */
  function teamCard(m, idx = 0) {
    const delay = Math.min(idx, 8) * 60;
    return `
    <article class="card fade-up" style="text-align:center;transition-delay:${delay}ms">
      ${
        m.photo_url
          ? `<img src="${safeUrl(m.photo_url)}" alt="${escHtml(m.name)}" width="72" height="72" loading="lazy" decoding="async" style="width:72px;height:72px;border-radius:50%;object-fit:cover;margin:0 auto 12px;border:1px solid var(--border)">`
          : `<div style="width:72px;height:72px;border-radius:50%;background:var(--bg-raised);border:1px solid var(--border);display:flex;align-items:center;justify-content:center;margin:0 auto 12px;color:var(--text-muted)">${icon('user', 28)}</div>`
      }
      <h3 style="font-size:15px;margin-bottom:4px">${escHtml(m.name)}</h3>
      <p style="color:var(--text);font-size:13.5px;font-weight:500;margin-bottom:4px">${!m.is_current ? '<span class="badge" style="margin-right:6px">Former</span>' : ''}${escHtml(m.role)}</p>
      ${m.role_responsibility ? `<p style="color:var(--text-secondary);font-size:12.5px;margin-bottom:6px">${escHtml(m.role_responsibility)}</p>` : ''}
      ${m.batch ? `<p style="color:var(--text-muted);font-size:12.5px;font-family:var(--font-mono)">Batch ${escHtml(m.batch)}</p>` : ''}
      ${m.bio ? `<p style="color:var(--text-secondary);font-size:13px;margin-top:8px;line-height:1.5">${escHtml(m.bio)}</p>` : ''}
      <div style="display:flex;justify-content:center;gap:12px;margin-top:12px;flex-wrap:wrap">
        ${m.email ? `<a href="mailto:${escHtml(m.email)}" style="font-size:13px;color:var(--text-muted);display:inline-flex;align-items:center;gap:4px">${icon('mail', 13)} Email</a>` : ''}
        ${m.linkedin_url ? `<a href="${safeUrl(m.linkedin_url)}" target="_blank" rel="noopener" style="font-size:13px;color:var(--text-secondary);display:inline-flex;align-items:center;gap:4px">${icon('external', 13)} LinkedIn</a>` : ''}
      </div>
    </article>`;
  }

  /**
   * @param {any} p
   * @param {number} [idx]
   * @returns {string}
   */
  function postCard(p, idx = 0) {
    const delay = Math.min(idx, 8) * 60;
    const authorInfo = [
      p.author_name || 'PMS',
      p.author_role,
      p.author_batch ? `Batch ${p.author_batch}` : null,
    ]
      .filter(Boolean)
      .map(escHtml)
      .join(' · ');

    return `
    <article class="card fade-up ${p.is_featured ? 'card-featured' : ''}" style="transition-delay:${delay}ms">
      ${p.cover_image_url ? `<img src="${safeUrl(p.cover_image_url)}" alt="" width="400" height="160" loading="lazy" decoding="async" style="width:100%;height:160px;object-fit:cover;border-radius:8px;margin-bottom:14px;border:1px solid var(--border-subtle)">` : ''}
      <div style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:10px">
        <span class="badge">${escHtml(p.category)}</span>
        ${p.material_formula ? `<span class="tag">[${escHtml(p.material_formula)}]</span>` : ''}
        ${p.is_featured ? '<span class="badge badge-accent">Featured</span>' : ''}
      </div>
      <h3 style="margin-bottom:8px">${escHtml(p.title)}</h3>
      <p style="color:var(--text-secondary);font-size:14px;margin-bottom:12px;line-height:1.5">${escHtml(p.excerpt || '')}</p>
      ${p.crystal_structure ? `<p style="color:var(--text-muted);font-size:12.5px;font-family:var(--font-mono);margin-bottom:8px">Lattice: ${escHtml(p.crystal_structure)}</p>` : ''}
      <div style="display:flex;justify-content:space-between;align-items:center;font-size:12.5px;color:var(--text-muted);margin-top:auto;padding-top:12px;border-top:1px solid var(--border-subtle);flex-wrap:wrap;gap:6px">
        <span>${authorInfo}</span>
        <span class="tabular-nums">${p.read_time_minutes ? `${p.read_time_minutes} min · ` : ''}${fmtDate(p.published_at)}</span>
      </div>
      <button class="btn btn-outline btn-sm btn-read-article" data-slug="${escHtml(p.slug)}" style="width:100%;margin-top:12px">Read Article</button>
    </article>`;
  }

  /**
   * @param {any} r
   * @param {number} [idx]
   * @returns {string}
   */
  function resourceCard(r, idx = 0) {
    const delay = Math.min(idx, 8) * 60;
    return `
    <article class="card fade-up ${r.is_pinned ? 'card-featured' : ''}" style="transition-delay:${delay}ms">
      ${r.is_pinned ? '<div style="margin-bottom:10px"><span class="badge badge-accent">Pinned</span></div>' : ''}
      <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:8px">
        <span class="badge">${escHtml(r.category)}</span>
        ${r.software_tag ? `<span class="tag">${escHtml(r.software_tag)}</span>` : ''}
      </div>
      <h3 style="font-size:15px;margin-bottom:6px">${escHtml(r.title)}</h3>
      <p style="color:var(--text-secondary);font-size:13.5px;margin:8px 0;line-height:1.5">${escHtml(r.description || '')}</p>
      <div style="margin-top:auto;padding-top:12px">
        <a href="${safeUrl(r.url)}" target="_blank" rel="noopener" class="btn btn-outline btn-sm" style="width:100%">${icon('external', 13)} Open Resource</a>
      </div>
    </article>`;
  }

  /* ── Shared Dialog Helpers (Backdrop pointer check + focus return) ── */
  /**
   * Close a modal only when both pointerdown and pointerup occur outside its bounding box.
   * @param {HTMLDialogElement | null} dialog
   */
  function enableBackdropClose(dialog) {
    if (!dialog || dialog.dataset.backdropCloseBound) return;
    dialog.dataset.backdropCloseBound = 'true';
    let downOutside = false;
    /** @param {PointerEvent} e */
    const isOutside = e => {
      const r = dialog.getBoundingClientRect();
      return e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
    };
    dialog.addEventListener('pointerdown', e => {
      downOutside = e.target === dialog && isOutside(e);
    });
    dialog.addEventListener('pointerup', e => {
      if (downOutside && e.target === dialog && isOutside(e)) {
        dialog.close();
      }
      downOutside = false;
    });
  }

  /**
   * Open a dialog modally and return focus to its triggering control upon close.
   * @param {PmsDialog | null} dialog
   * @param {HTMLElement} [triggerEl]
   */
  function showModalWithFocusReturn(dialog, triggerEl) {
    if (!dialog) return;
    if (!dialog.dataset.focusReturnBound) {
      dialog.dataset.focusReturnBound = 'true';
      dialog.addEventListener('close', () => {
        const target = dialog._returnFocusEl;
        dialog._returnFocusEl = null;
        if (
          target &&
          target !== document.body &&
          typeof target.focus === 'function' &&
          target.isConnected
        ) {
          target.focus();
        }
      });
    }
    const candidate =
      triggerEl || (document.activeElement instanceof HTMLElement ? document.activeElement : null);
    if (!dialog.open) {
      dialog._returnFocusEl = candidate && !dialog.contains(candidate) ? candidate : null;
      dialog.showModal();
    }
  }

  /* ── Global Event Registration Modal (Supported on index.html, events.html, etc.) ── */
  /**
   * @param {string} eventId
   * @param {string} [eventTitle]
   * @param {HTMLElement} [triggerEl]
   */
  function openRegistrationModal(eventId, eventTitle, triggerEl) {
    let modal = /** @type {PmsDialog | null} */ (document.getElementById('reg-modal'));
    if (!modal) {
      modal = /** @type {PmsDialog} */ (document.createElement('dialog'));
      modal.id = 'reg-modal';
      modal.className = 'modal modal--sm';
      modal.setAttribute('aria-labelledby', 'reg-modal-title');
      modal.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.25rem">
        <h2 id="reg-modal-title" style="margin:0;font-size:1.35rem">Register for Event</h2>
        <button type="button" id="reg-close-icon" class="btn btn-outline btn-sm" aria-label="Close dialog">${icon('x', 14)} Close</button>
      </div>
      <form id="reg-form" novalidate>
        <input type="hidden" id="reg-event-id">
        <div class="form-group">
          <label class="form-label" for="reg-name">Full Name *</label>
          <input id="reg-name" class="form-control" required maxlength="200" autocomplete="name" placeholder="Your full name">
        </div>
        <div class="grid grid-2" style="gap:.75rem">
          <div class="form-group">
            <label class="form-label" for="reg-roll">Roll Number *</label>
            <input id="reg-roll" class="form-control" required maxlength="50" autocomplete="off" placeholder="e.g. 22-DMME-01">
          </div>
          <div class="form-group">
            <label class="form-label" for="reg-batch">Batch *</label>
            <input id="reg-batch" class="form-control" required maxlength="50" autocomplete="off" placeholder="e.g. 2022-2026">
          </div>
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-dept">Academic Department *</label>
          <input id="reg-dept" class="form-control" required maxlength="200" autocomplete="organization" placeholder="e.g. Metallurgy & Materials Engineering (DMME)">
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-email">PIEAS Email *</label>
          <input id="reg-email" type="email" inputmode="email" class="form-control" required maxlength="254" autocomplete="email" placeholder="student@pieas.edu.pk">
        </div>
        <div class="form-group">
          <label class="form-label" for="reg-phone">Contact Phone</label>
          <input id="reg-phone" type="tel" inputmode="tel" class="form-control" maxlength="50" autocomplete="tel" placeholder="Optional for SMS updates">
        </div>
        <div id="reg-msg" role="status" aria-live="polite" style="margin-top:1rem"></div>
        <div style="display:flex;justify-content:flex-end;gap:.75rem;margin-top:1.5rem">
          <button type="button" class="btn btn-outline" id="reg-close">Cancel</button>
          <button type="submit" class="btn btn-primary" id="reg-submit">Confirm Registration</button>
        </div>
      </form>
    `;
      document.body.appendChild(modal);
      enableBackdropClose(modal);
    }

    // Reset form inputs, set fields and reset alert
    const form = /** @type {HTMLFormElement | null} */ (modal.querySelector('#reg-form'));
    if (form) {
      form.reset();
      reportInvalidFields(form, []);
    }
    const titleEl = modal.querySelector('#reg-modal-title');
    if (titleEl)
      titleEl.textContent = eventTitle ? `Register: ${eventTitle}` : 'Register for Event';
    const idEl = /** @type {HTMLInputElement | null} */ (modal.querySelector('#reg-event-id'));
    if (idEl) idEl.value = eventId;
    const msgBox = modal.querySelector('#reg-msg');
    if (msgBox) msgBox.innerHTML = '';

    // Check if code is already stored in localStorage
    let savedCode = null;
    try {
      savedCode = localStorage.getItem(`pms_cert_${eventId}`);
    } catch (_) {}
    if (savedCode && msgBox) {
      showAlert(
        msgBox,
        `Previously registered for this event. Saved verification passcode: <br><strong style="font-size:1.1rem;color:var(--text);font-family:var(--font-mono)">${escHtml(savedCode)}</strong>`,
        'info',
        true
      );
    }

    // Ensure submit button is re-enabled and visible on opening modal
    const submitBtn = /** @type {HTMLButtonElement | null} */ (modal.querySelector('#reg-submit'));
    if (submitBtn) {
      submitBtn.disabled = false;
      submitBtn.style.display = '';
    }

    // Bind close buttons and form submit once
    const dialogEl = modal;
    if (!dialogEl.dataset.initialized) {
      dialogEl.dataset.initialized = 'true';
      dialogEl.querySelectorAll('#reg-close, #reg-close-icon').forEach(b => {
        if (b instanceof HTMLElement) b.onclick = () => dialogEl.close();
      });

      const formEl = /** @type {HTMLFormElement | null} */ (dialogEl.querySelector('#reg-form'));
      if (formEl) {
        formEl.addEventListener('submit', async ev => {
          ev.preventDefault();
          const curSubmitBtn = /** @type {HTMLButtonElement | null} */ (
            dialogEl.querySelector('#reg-submit')
          );
          const curMsgBox = dialogEl.querySelector('#reg-msg');
          const curEventInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-event-id')
          );
          const curEventId = curEventInput ? curEventInput.value : '';

          const nameInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-name')
          );
          const rollInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-roll')
          );
          const batchInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-batch')
          );
          const deptInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-dept')
          );
          const emailInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-email')
          );
          const phoneInput = /** @type {HTMLInputElement | null} */ (
            dialogEl.querySelector('#reg-phone')
          );

          const name = (nameInput?.value || '').trim();
          const roll = (rollInput?.value || '').trim();
          const batch = (batchInput?.value || '').trim();
          const dept = (deptInput?.value || '').trim();
          const email = (emailInput?.value || '').trim();
          const phone = (phoneInput?.value || '').trim();

          const missingInputs = [nameInput, rollInput, batchInput, deptInput, emailInput].filter(
            el => !el || !el.value.trim()
          );
          if (missingInputs.length) {
            reportInvalidFields(formEl, missingInputs);
            showAlert(curMsgBox, 'Please fill in all required fields.', 'error');
            return;
          }
          if (!isEmail(email)) {
            reportInvalidFields(formEl, [emailInput]);
            showAlert(curMsgBox, 'Please enter a valid email address.', 'error');
            return;
          }
          reportInvalidFields(formEl, []);

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

            const code = result.certificate_code;
            try {
              localStorage.setItem(`pms_cert_${curEventId}`, code);
            } catch (_) {}

            showAlert(
              curMsgBox,
              `Registration confirmed. Digital verification passcode: <br><strong style="font-size:1.15rem;letter-spacing:1px;color:var(--text);font-family:var(--font-mono)" id="cert-code-text">${escHtml(code)}</strong><br><button type="button" class="btn btn-outline btn-sm" id="btn-copy-cert-code" style="margin:.5rem 0 .25rem;padding:.3rem .75rem;font-size:.8rem">${icon('copy', 13)} Copy Passcode</button><br><small style="color:var(--text-muted)">Save this passcode to verify your certificate on this portal.</small>`,
              'success',
              true
            );

            const copyBtn = /** @type {HTMLButtonElement | null} */ (
              dialogEl.querySelector('#btn-copy-cert-code')
            );
            if (copyBtn) {
              copyBtn.onclick = () => {
                if (navigator.clipboard && navigator.clipboard.writeText) {
                  navigator.clipboard
                    .writeText(code)
                    .then(() => {
                      copyBtn.textContent = 'Copied';
                      setTimeout(() => {
                        copyBtn.innerHTML = `${icon('copy', 13)} Copy Passcode`;
                      }, 2000);
                    })
                    .catch(() => {
                      prompt('Copy your code:', code);
                    });
                } else {
                  prompt('Copy your code:', code);
                }
              };
            }

            formEl.reset();
            isSuccess = true;
            // Hide and disable submit button to prevent accidental double-registration
            if (curSubmitBtn) {
              curSubmitBtn.disabled = true;
              curSubmitBtn.style.display = 'none';
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : 'Registration failed.';
            showAlert(curMsgBox, msg || 'Registration failed.', 'error');
          } finally {
            setLoading(curSubmitBtn, false);
            if (isSuccess && curSubmitBtn) {
              curSubmitBtn.disabled = true;
              curSubmitBtn.style.display = 'none';
            }
          }
        });
      }
    }

    showModalWithFocusReturn(dialogEl, triggerEl);
  }

  /* ── Lightbox for SEM & Optical Micrographs (gallery.html) ── */
  /**
   * @param {string} imageUrl
   * @param {string} title
   * @param {string} [caption]
   * @param {HTMLElement} [triggerEl]
   */
  function openLightbox(imageUrl, title, caption = '', triggerEl) {
    let modal = /** @type {PmsDialog | null} */ (document.getElementById('lightbox-modal'));
    if (!modal) {
      modal = /** @type {PmsDialog} */ (document.createElement('dialog'));
      modal.id = 'lightbox-modal';
      modal.className = 'modal lightbox-modal';
      modal.setAttribute('aria-labelledby', 'lightbox-title');
      modal.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem">
        <h2 id="lightbox-title" style="font-size:1.15rem;color:var(--text)">Micrograph</h2>
        <button type="button" id="lightbox-close-btn" class="btn btn-outline btn-sm" aria-label="Close dialog">${icon('x', 14)} Close</button>
      </div>
      <img id="lightbox-img" src="" alt="Micrograph" style="max-width:100%;max-height:70vh;object-fit:contain;border-radius:8px">
      <p id="lightbox-caption" style="color:var(--text-muted);font-size:.9rem;line-height:1.6;margin-top:.75rem"></p>
    `;
      document.body.appendChild(modal);
      const lbModal = modal;
      const lbCloseBtn = /** @type {HTMLButtonElement | null} */ (
        lbModal.querySelector('#lightbox-close-btn')
      );
      if (lbCloseBtn) lbCloseBtn.onclick = () => lbModal.close();
      enableBackdropClose(lbModal);
    }

    const cleanTitle = title && title !== 'null' ? title : 'Micrograph Detail';
    const cleanCaption = caption && caption !== 'null' ? caption : '';
    const lightboxImg = /** @type {HTMLImageElement | null} */ (
      modal.querySelector('#lightbox-img')
    );
    if (lightboxImg) {
      lightboxImg.src = safeUrlRaw(imageUrl);
      lightboxImg.alt = cleanTitle;
    }
    const lbTitle = modal.querySelector('#lightbox-title');
    if (lbTitle) lbTitle.textContent = cleanTitle;
    const lbCaption = modal.querySelector('#lightbox-caption');
    if (lbCaption) lbCaption.textContent = cleanCaption;
    showModalWithFocusReturn(modal, triggerEl);
  }

  /**
   * Extract an article slug from `/blog/<slug>` or legacy `?article=<slug>` URLs.
   * @returns {string | null}
   */
  function getArticleSlugFromUrl() {
    const match = window.location.pathname.match(/^\/blog\/([^/]+)\/?$/);
    if (match) {
      try {
        return decodeURIComponent(match[1]);
      } catch (_) {
        return match[1];
      }
    }
    return new URLSearchParams(window.location.search).get('article');
  }

  /* ── Open Article Modal and load full article (with URL Deep Linking) ── */
  /**
   * Open the article reader dialog and load a blog post by slug.
   * @param {string} slug
   * @param {HTMLElement} [triggerEl]
   * @param {{ skipPush?: boolean, replace?: boolean }} [opts]
   */
  async function openArticleModal(slug, triggerEl, opts = {}) {
    if (!slug) return;
    let modal = /** @type {PmsDialog | null} */ (document.getElementById('article-reader-modal'));
    if (!modal) {
      modal = /** @type {PmsDialog} */ (document.createElement('dialog'));
      modal.id = 'article-reader-modal';
      modal.className = 'modal modal--lg';
      modal.setAttribute('aria-labelledby', 'article-reader-title');
      document.body.appendChild(modal);
      enableBackdropClose(modal);
    }

    const articleModal = modal;

    // Deep linking: canonical path /blog/<slug>
    if (!opts.skipPush) {
      try {
        const targetPath = `/blog/${encodeURIComponent(slug)}`;
        const currentPath = window.location.pathname.replace(/\/$/, '') || '/';
        if (opts.replace || new URLSearchParams(window.location.search).has('article')) {
          history.replaceState(null, '', targetPath);
        } else if (currentPath !== targetPath) {
          history.pushState(null, '', targetPath);
        }
      } catch (_) {}
    }

    // Restore /blog URL on manual modal close
    if (!articleModal.dataset.hasCloseHandler) {
      articleModal.dataset.hasCloseHandler = 'true';
      articleModal.addEventListener('close', () => {
        if (articleModal._closingFromPopstate) {
          articleModal._closingFromPopstate = false;
          return;
        }
        try {
          const url = new URL(window.location.href);
          if (/^\/blog\/[^/]+\/?$/.test(url.pathname)) {
            history.replaceState(null, '', '/blog');
          } else if (url.searchParams.has('article')) {
            url.searchParams.delete('article');
            const cleanUrl = url.pathname + (url.search ? url.search : '') + url.hash;
            history.replaceState(null, '', cleanUrl);
          }
        } catch (_) {}
      });
    }

    articleModal.innerHTML = `
    <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.5rem">
      <span class="badge">Technical Article</span>
      <button type="button" id="close-article-modal" class="btn btn-outline btn-sm" aria-label="Close dialog">${icon('x', 14)} Close</button>
    </div>
    <div id="article-reader-content" aria-busy="true"><div class="spinner" role="status" aria-live="polite" aria-label="Loading article"></div></div>
  `;
    showModalWithFocusReturn(articleModal, triggerEl);
    const closeArticleBtn = /** @type {HTMLButtonElement | null} */ (
      document.getElementById('close-article-modal')
    );
    if (closeArticleBtn) closeArticleBtn.onclick = () => articleModal.close();

    const contentEl = document.getElementById('article-reader-content');
    if (!contentEl) return;
    try {
      const post = await apiFetch(`/api/posts/${encodeURIComponent(slug)}`);
      contentEl.innerHTML = `
      ${post.cover_image_url ? `<img src="${safeUrl(post.cover_image_url)}" alt="" width="720" height="300" decoding="async" style="width:100%;height:auto;max-height:300px;object-fit:cover;border-radius:10px;margin-bottom:1.5rem">` : ''}
      <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-bottom:.75rem">
        <span class="badge">${escHtml(post.category)}</span>
        ${post.material_formula ? `<span class="tag">[${escHtml(post.material_formula)}]</span>` : ''}
        ${post.crystal_structure ? `<span class="tag">Lattice: ${escHtml(post.crystal_structure)}</span>` : ''}
      </div>
      <h1 id="article-reader-title" style="font-size:1.8rem;margin-bottom:.75rem;color:var(--text)">${escHtml(post.title)}</h1>
      <div style="display:flex;gap:1.5rem;font-size:.85rem;color:var(--text-muted);border-bottom:1px solid var(--border);padding-bottom:1rem;margin-bottom:1.5rem;flex-wrap:wrap">
        <span style="display:inline-flex;align-items:center;gap:6px">${icon('user', 14)} <strong>${escHtml(post.author_name || 'PMS Technical Council')}</strong> ${post.author_role ? '· ' + escHtml(post.author_role) : ''}</span>
        <span style="display:inline-flex;align-items:center;gap:6px">${icon('calendar', 14)} ${fmtDate(post.published_at)}</span>
        ${post.read_time_minutes ? `<span style="display:inline-flex;align-items:center;gap:6px">${icon('clock', 14)} ${post.read_time_minutes} min read</span>` : ''}
      </div>
      <div class="article-body" style="font-size:1rem;color:var(--text);line-height:1.75">
        ${renderMarkdown(post.body_markdown)}
      </div>
    `;
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Unknown error';
      contentEl.innerHTML = `
      <div class="alert alert-error" role="alert">Failed to load article: ${escHtml(msg)}</div>
    `;
    } finally {
      contentEl.setAttribute('aria-busy', 'false');
    }
  }

  /* ── Event delegation for Global Actions ── */
  document.addEventListener('click', ev => {
    if (!(ev.target instanceof Element)) return;
    // Read article modal
    const readBtn = ev.target.closest('.btn-read-article');
    if (readBtn instanceof HTMLElement && readBtn.dataset.slug) {
      ev.preventDefault();
      openArticleModal(readBtn.dataset.slug, readBtn);
      return;
    }

    // Register for event modal (works on homepage featured cards and events.html)
    const regBtn = ev.target.closest('.btn-open-reg');
    if (regBtn instanceof HTMLElement && regBtn.dataset.eventId) {
      ev.preventDefault();
      openRegistrationModal(regBtn.dataset.eventId, regBtn.dataset.eventTitle, regBtn);
      return;
    }

    // Micrograph lightbox
    const lbTrigger = ev.target.closest('.micrograph-lightbox-trigger');
    if (lbTrigger instanceof HTMLElement) {
      ev.preventDefault();
      openLightbox(
        lbTrigger.dataset.imgUrl || '',
        lbTrigger.dataset.imgTitle || '',
        lbTrigger.dataset.imgCaption || '',
        lbTrigger
      );
    }
  });

  /* ── Popstate handler for back/forward navigation with article deep links ── */
  window.addEventListener('popstate', () => {
    const articleSlug = getArticleSlugFromUrl();
    const modal = /** @type {PmsDialog | null} */ (document.getElementById('article-reader-modal'));
    if (articleSlug) {
      const hasLegacy = new URLSearchParams(window.location.search).has('article');
      openArticleModal(articleSlug, undefined, { skipPush: !hasLegacy, replace: hasLegacy });
    } else if (modal && modal.open) {
      modal._closingFromPopstate = true;
      modal.close();
    }
  });

  /* ── On DOM ready ── */
  document.addEventListener('DOMContentLoaded', () => {
    initTheme();
    initNavbar();
    initScrollReveal();
    loadAnnouncements(qs('#announcement-banner'));
    loadStats(qs('#stats-strip'));

    // Auto-open article modal for /blog/<slug> or legacy ?article=<slug>
    const hasLegacy = new URLSearchParams(window.location.search).has('article');
    const articleSlug = getArticleSlugFromUrl();
    if (articleSlug) {
      openArticleModal(articleSlug, undefined, { skipPush: !hasLegacy, replace: hasLegacy });
    }
  });

  /** Export shared client helpers for use across page scripts. */
  /** @type {any} */ (window).PMS = {
    initTheme,
    icon,
    apiFetch,
    apiPost,
    apiPut,
    apiDelete,
    escHtml,
    safeUrlRaw,
    safeUrl,
    isEmail,
    emptyState,
    errorState,
    fmtDate,
    renderMarkdown,
    openArticleModal,
    openRegistrationModal,
    openLightbox,
    enableBackdropClose,
    showModalWithFocusReturn,
    skeletonGrid,
    showAlert,
    reportInvalidFields,
    setLoading,
    observeFadeElements,
    eventCard,
    teamCard,
    postCard,
    resourceCard,
  };
})();
