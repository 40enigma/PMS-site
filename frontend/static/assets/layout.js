// @ts-check
/**
 * Shared navbar and footer injection for PMS public and admin pages.
 * Replaces `#site-header` and `#site-footer` via `outerHTML` so `.navbar`'s
 * `position: sticky` continues to stick directly against `<body>`.
 */
(() => {
  'use strict';

  /**
   * Build the public navigation bar markup.
   * @returns {string}
   */
  function buildPublicHeader() {
    return (
      '<nav class="navbar" role="navigation" aria-label="Main navigation">' +
      '<div class="container nav-inner">' +
      '<a href="/" class="nav-brand">' +
      '<img src="/pms-logo.jpeg" alt="PIEAS Materials Society Logo" class="nav-logo" width="28" height="28" decoding="async"> PIEAS Materials Society' +
      '</a>' +
      '<ul class="nav-links" id="nav-links" role="list">' +
      '<li><a href="/">Home</a></li>' +
      '<li><a href="/about">About</a></li>' +
      '<li><a href="/team">Team</a></li>' +
      '<li><a href="/events">Events</a></li>' +
      '<li><a href="/blog">Blog</a></li>' +
      '<li><a href="/resources">Resources</a></li>' +
      '<li><a href="/opportunities">Opportunities</a></li>' +
      '<li><a href="/alumni">Alumni</a></li>' +
      '<li><a href="/projects">Projects</a></li>' +
      '<li><a href="/gallery">Gallery</a></li>' +
      '<li><a href="/sponsors">Sponsors</a></li>' +
      '<li><a href="/join" class="nav-cta btn btn-primary btn-sm">Join PMS</a></li>' +
      '</ul>' +
      '<div class="nav-actions">' +
      '<button id="theme-toggle" class="theme-toggle-btn" type="button" aria-label="Toggle theme" title="Toggle theme"></button>' +
      '<button type="button" class="hamburger" id="nav-toggle" aria-label="Toggle navigation" aria-expanded="false" aria-controls="nav-links">' +
      '<span></span><span></span><span></span>' +
      '</button>' +
      '</div>' +
      '</div>' +
      '</nav>'
    );
  }

  /**
   * Build the admin navigation bar markup.
   * @returns {string}
   */
  function buildAdminHeader() {
    return (
      '<nav class="navbar" role="navigation" aria-label="Main navigation">' +
      '<div class="container nav-inner">' +
      '<a href="/" class="nav-brand">' +
      '<img src="/pms-logo.jpeg" alt="PIEAS Materials Society Logo" class="nav-logo" width="28" height="28" decoding="async">' +
      ' PIEAS Materials Society <span>Admin</span>' +
      '</a>' +
      '<ul class="nav-links" id="nav-links" role="list">' +
      '<li><a href="/">View Live Site</a></li>' +
      '<li id="nav-user-item" style="display:none"><span id="nav-user" style="color:var(--text-muted);font-size:.85rem;font-weight:600"></span></li>' +
      '<li id="nav-logout-item" style="display:none"><button type="button" id="logout-btn" class="btn btn-outline btn-sm" style="display:none">Logout</button></li>' +
      '</ul>' +
      '<div class="nav-actions">' +
      '<button id="theme-toggle" class="theme-toggle-btn" type="button" aria-label="Toggle theme" title="Toggle theme"></button>' +
      '<button type="button" class="hamburger" id="nav-toggle" aria-label="Toggle navigation" aria-expanded="false" aria-controls="nav-links">' +
      '<span></span><span></span><span></span>' +
      '</button>' +
      '</div>' +
      '</div>' +
      '</nav>'
    );
  }

  /**
   * Build the shared public footer markup.
   * @param {number} year
   * @returns {string}
   */
  function buildPublicFooter(year) {
    return (
      '<footer>' +
      '<div class="container">' +
      '<div class="footer-grid">' +
      '<div>' +
      '<div class="footer-title">PMS PIEAS</div>' +
      '<p style="font-size:13px;line-height:1.6">Official academic society of the Department of Metallurgy &amp; Materials Engineering (DMME), PIEAS Islamabad.</p>' +
      '</div>' +
      '<div>' +
      '<div class="footer-title">Quick Links</div>' +
      '<ul class="footer-links">' +
      '<li><a href="/events">Events</a></li>' +
      '<li><a href="/blog">Technical Blog</a></li>' +
      '<li><a href="/resources">Toolkit &amp; ASTM</a></li>' +
      '<li><a href="/opportunities">Opportunities</a></li>' +
      '</ul>' +
      '</div>' +
      '<div>' +
      '<div class="footer-title">Society</div>' +
      '<ul class="footer-links">' +
      '<li><a href="/about">About PMS</a></li>' +
      '<li><a href="/team">Our Council</a></li>' +
      '<li><a href="/alumni">Alumni Network</a></li>' +
      '<li><a href="/sponsors">Sponsors</a></li>' +
      '</ul>' +
      '</div>' +
      '<div>' +
      '<div class="footer-title">Contact &amp; Governance</div>' +
      '<p style="font-size:.85rem;line-height:1.6">' +
      '<a href="mailto:pms@pieas.edu.pk" class="pms-contact-email" style="color:var(--text-muted)">pms@pieas.edu.pk</a><br>' +
      '<span class="pms-office-location">DMME Block, PIEAS, Nilore, Islamabad</span><br>' +
      '<a href="#" class="pms-constitution-link" style="display:none;margin-top:.45rem;color:var(--text);font-weight:500">Society Constitution (PDF) &#8599;</a>' +
      '</p>' +
      '</div>' +
      '</div>' +
      '<div class="footer-bottom">' +
      '<p>&copy; ' +
      year +
      ' PIEAS Materials Society (PMS). Department of Metallurgy &amp; Materials Engineering.</p>' +
      '</div>' +
      '</div>' +
      '</footer>'
    );
  }

  /**
   * Build the admin footer markup.
   * @param {number} year
   * @returns {string}
   */
  function buildAdminFooter(year) {
    return (
      '<footer>' +
      '<div class="container footer-bottom">' +
      '<p>&copy; ' +
      year +
      ' PIEAS Materials Society (PMS) &bull; Department of Metallurgy &amp; Materials Engineering (DMME)</p>' +
      '</div>' +
      '</footer>'
    );
  }

  /**
   * Inject header and footer placeholders into the document.
   */
  function injectLayout() {
    const year = new Date().getFullYear();
    const headerSlot = document.getElementById('site-header');
    if (headerSlot) {
      const isAdminHeader = headerSlot.getAttribute('data-layout') === 'admin';
      headerSlot.outerHTML = isAdminHeader ? buildAdminHeader() : buildPublicHeader();
    }
    const footerSlot = document.getElementById('site-footer');
    if (footerSlot) {
      const isAdminFooter = footerSlot.getAttribute('data-layout') === 'admin';
      footerSlot.outerHTML = isAdminFooter ? buildAdminFooter(year) : buildPublicFooter(year);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', injectLayout);
  } else {
    injectLayout();
  }
})();
