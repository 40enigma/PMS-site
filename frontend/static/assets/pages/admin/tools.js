/**
 * PIEAS Materials Society (PMS)
 * Admin Site Stats Editor & Media Upload Tool
 * frontend/static/assets/pages/admin/tools.js
 */
(() => {
  'use strict';

  /**
   * Render and handle the site statistics and contact governance editor.
   * @param {HTMLElement} container
   */
  async function renderStatsEditor(container) {
    try {
      const stats = await PMS.apiFetch('/api/admin/stats');
      container.innerHTML = `
        <h2 style="font-size:1.25rem;margin-bottom:.5rem">Edit Key Site Statistics &amp; Society Details</h2>
        <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:1.5rem">These metrics power the live counter strip and contact info on the homepage.</p>
        <form id="stats-form">
          <div class="grid grid-2" style="gap:1rem">
            <div class="form-group">
              <label class="form-label" for="stat-members">Active Members Count</label>
              <input id="stat-members" type="number" min="0" class="form-control" value="${stats.active_members || 0}">
            </div>
            <div class="form-group">
              <label class="form-label" for="stat-events">Total Events Held</label>
              <input id="stat-events" type="number" min="0" class="form-control" value="${stats.events_held || 0}">
            </div>
            <div class="form-group">
              <label class="form-label" for="stat-years">Years Active</label>
              <input id="stat-years" type="number" min="0" class="form-control" value="${stats.years_active || 0}">
            </div>
            <div class="form-group">
              <label class="form-label" for="stat-partners">Industrial Partners Count</label>
              <input id="stat-partners" type="number" min="0" class="form-control" value="${stats.industrial_partners || 0}">
            </div>
            <div class="form-group" style="grid-column:1/-1">
              <label class="form-label" for="stat-pdf">Constitution PDF URL</label>
              <input id="stat-pdf" type="text" class="form-control" value="${PMS.escHtml(stats.constitution_pdf_url || '')}">
            </div>
            <div class="form-group">
              <label class="form-label" for="stat-loc">Office Location</label>
              <input id="stat-loc" type="text" class="form-control" value="${PMS.escHtml(stats.office_location || '')}">
            </div>
            <div class="form-group">
              <label class="form-label" for="stat-email">Official Contact Email</label>
              <input id="stat-email" type="email" class="form-control" value="${PMS.escHtml(stats.contact_email || '')}">
            </div>
          </div>
          <div id="stats-msg" aria-live="polite" style="margin-top:1rem"></div>
          <button type="submit" class="btn btn-primary" style="margin-top:1rem" id="stats-save-btn">Save Statistics</button>
        </form>
      `;

      document.getElementById('stats-form').addEventListener('submit', async ev => {
        ev.preventDefault();
        const btn = document.getElementById('stats-save-btn');
        const msg = document.getElementById('stats-msg');

        const active_members = parseInt(document.getElementById('stat-members').value, 10) || 0;
        const events_held = parseInt(document.getElementById('stat-events').value, 10) || 0;
        const years_active = parseInt(document.getElementById('stat-years').value, 10) || 0;
        const industrial_partners =
          parseInt(document.getElementById('stat-partners').value, 10) || 0;

        const statsForm = document.getElementById('stats-form');
        const statInputs = ['stat-members', 'stat-events', 'stat-years', 'stat-partners'].map(id =>
          document.getElementById(id)
        );
        const negativeInputs = statInputs.filter(el => (parseInt(el.value, 10) || 0) < 0);
        if (negativeInputs.length) {
          PMS.reportInvalidFields(statsForm, negativeInputs, msg);
          PMS.showAlert(msg, 'Statistic counts cannot be negative numbers.');
          return;
        }
        PMS.reportInvalidFields(statsForm, []);

        PMS.setLoading(btn, true);
        try {
          await PMS.apiPut('/api/admin/stats', {
            active_members,
            events_held,
            years_active,
            industrial_partners,
            constitution_pdf_url: document.getElementById('stat-pdf').value.trim(),
            office_location: document.getElementById('stat-loc').value.trim(),
            contact_email: document.getElementById('stat-email').value.trim(),
          });
          PMS.showAlert(msg, 'Site statistics updated.', 'success');
        } catch (e) {
          if (PMSAdmin.isUnauthorized(e)) {
            PMSAdmin.showLogin();
            return;
          }
          PMS.showAlert(msg, e.message || 'Failed to update stats.');
        } finally {
          PMS.setLoading(btn, false);
        }
      });
    } catch (err) {
      if (PMSAdmin.isUnauthorized(err)) {
        PMSAdmin.showLogin();
      } else {
        container.innerHTML = PMS.errorState(`Error loading statistics: ${err.message}`);
      }
    }
  }

  /**
   * Render the static media asset uploader tool.
   * @param {HTMLElement} container
   */
  function renderUploadTool(container) {
    container.innerHTML = `
      <h2 style="font-size:1.25rem;margin-bottom:.5rem">Upload Static Media Asset</h2>
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:1.5rem">
        Allowed formats: PNG, JPEG, WebP, PDF, text. Maximum 20 MB. Files are stored in <code>/uploads/</code> with strict non-executable permissions.
      </p>
      <form id="upload-form">
        <div class="form-group">
          <label class="form-label" for="upload-file">Select File</label>
          <input id="upload-file" type="file" class="form-control" accept="image/*,application/pdf,text/plain" required>
        </div>
        <div id="upload-msg" aria-live="polite" style="margin-top:1rem"></div>
        <button type="submit" class="btn btn-primary" style="margin-top:1rem" id="upload-submit">Upload Asset</button>
      </form>
    `;

    document.getElementById('upload-form').addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('upload-submit');
      const msg = document.getElementById('upload-msg');
      const file = document.getElementById('upload-file').files[0];
      if (!file) return;
      PMS.setLoading(btn, true);
      try {
        const fd = new FormData();
        fd.append('file', file);
        const res = await fetch('/api/admin/upload', {
          method: 'POST',
          credentials: 'include',
          headers: { 'X-Requested-With': 'PMS-Client' },
          body: fd,
        });
        if (!res.ok) {
          if (res.status === 401) {
            PMSAdmin.showLogin();
            return;
          }
          let errDetail = '';
          try {
            const b = await res.json();
            errDetail = b.error;
          } catch (_) {}
          throw new Error(errDetail || `Upload failed (HTTP ${res.status})`);
        }
        const data = await res.json();
        PMS.showAlert(
          msg,
          `Asset uploaded: <a href="${PMS.safeUrl(data.url)}" target="_blank" rel="noopener noreferrer" style="color:var(--text);font-weight:600">${PMS.escHtml(data.url)}</a>`,
          'success',
          true
        );
      } catch (e) {
        PMS.reportError('upload', e);
        PMS.showAlert(msg, e.message);
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }

  window.PMSAdmin = Object.assign(window.PMSAdmin || {}, {
    renderStatsEditor,
    renderUploadTool,
  });
})();
