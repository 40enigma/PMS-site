/**
 * PIEAS Materials Society (PMS)
 * Admin Authentication & Session State
 * frontend/static/assets/pages/admin/auth.js
 */
(() => {
  'use strict';

  /**
   * Verify admin session on load and toggle login vs dashboard view.
   */
  async function checkAuth() {
    try {
      const user = await PMS.apiFetch('/api/auth/me');
      showDashboard(user);
    } catch (_) {
      showLogin();
    }
  }

  /**
   * Display the login view and hide dashboard controls.
   */
  function showLogin() {
    // Never leave previously loaded records (member PII) in the DOM after logout
    const content = document.getElementById('admin-content');
    if (content) content.innerHTML = '';
    document.querySelectorAll('dialog[open]').forEach(dialog => {
      if (dialog instanceof HTMLDialogElement) dialog.close();
    });
    if (window.PMSAdmin) {
      window.PMSAdmin.currentCache = [];
      window.PMSAdmin.currentEditingId = null;
    }
    const loginSec = document.getElementById('login-section');
    if (loginSec) loginSec.style.display = '';
    const dash = document.getElementById('admin-dashboard');
    if (dash) dash.style.display = 'none';
    const userItem = document.getElementById('nav-user-item');
    if (userItem) userItem.style.display = 'none';
    const logoutItem = document.getElementById('nav-logout-item');
    if (logoutItem) logoutItem.style.display = 'none';
    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) logoutBtn.style.display = 'none';
    const navUser = document.getElementById('nav-user');
    if (navUser) navUser.textContent = '';
  }

  /**
   * Display the admin console dashboard for an authenticated admin user.
   * @param {{ name?: string }} user
   */
  function showDashboard(user) {
    const loginSec = document.getElementById('login-section');
    if (loginSec) loginSec.style.display = 'none';
    const dash = document.getElementById('admin-dashboard');
    if (dash) dash.style.display = '';
    const userItem = document.getElementById('nav-user-item');
    if (userItem) userItem.style.display = '';
    const logoutItem = document.getElementById('nav-logout-item');
    if (logoutItem) logoutItem.style.display = '';
    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) logoutBtn.style.display = '';
    if (user && user.name) {
      const navUser = document.getElementById('nav-user');
      if (navUser) navUser.textContent = user.name;
    }
    PMSAdmin.loadSection(PMSAdmin.currentTab);
  }

  document.addEventListener('DOMContentLoaded', () => {
    const loginForm = document.getElementById('login-form');
    if (loginForm) {
      loginForm.addEventListener('submit', async ev => {
        ev.preventDefault();
        const btn = document.getElementById('login-submit');
        const msg = document.getElementById('login-msg');
        const emailEl = document.getElementById('login-email');
        const pwEl = document.getElementById('login-password');
        const missing = [emailEl, pwEl].filter(el => !el || !el.value.trim());
        if (missing.length) {
          PMS.reportInvalidFields(loginForm, missing, msg);
          PMS.showAlert(msg, 'Please enter your administrator email and password.');
          return;
        }
        if (emailEl && !PMS.isEmail(emailEl.value)) {
          PMS.reportInvalidFields(loginForm, [emailEl], msg);
          PMS.showAlert(msg, 'Please enter a valid administrator email address.');
          return;
        }
        PMS.reportInvalidFields(loginForm, []);
        PMS.setLoading(btn, true);
        try {
          const res = await PMS.apiPost('/api/auth/login', {
            email: emailEl.value,
            password: pwEl.value,
          });
          showDashboard(res.admin || res);
        } catch (e) {
          PMS.reportError('login', e);
          PMS.showAlert(msg, e.message || 'Invalid administrator credentials.');
        } finally {
          PMS.setLoading(btn, false);
        }
      });
    }

    const logoutBtn = document.getElementById('logout-btn');
    if (logoutBtn) {
      logoutBtn.addEventListener('click', async () => {
        try {
          await PMS.apiPost('/api/auth/logout', {});
        } catch (_) {}
        showLogin();
        const navLinks = document.querySelector('.nav-links');
        const hamburger = document.querySelector('.hamburger');
        if (navLinks) navLinks.classList.remove('open');
        if (hamburger) {
          hamburger.classList.remove('active');
          hamburger.setAttribute('aria-expanded', 'false');
        }
      });
    }

    checkAuth();
  });

  window.PMSAdmin = Object.assign(window.PMSAdmin || {}, {
    checkAuth,
    showLogin,
    showDashboard,
  });
})();
