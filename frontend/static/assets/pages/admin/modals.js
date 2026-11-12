/**
 * PIEAS Materials Society (PMS)
 * Admin CRUD, View Details, and Inline Upload Modals
 * frontend/static/assets/pages/admin/modals.js
 */
(() => {
  'use strict';

  const modal = document.getElementById('crud-modal');
  if (modal) PMS.enableBackdropClose(modal);
  const modalCancelBtn = document.getElementById('modal-cancel-btn');
  if (modalCancelBtn && modal) modalCancelBtn.onclick = () => modal.close();
  const modalCancelFooterBtn = document.getElementById('modal-cancel-footer-btn');
  if (modalCancelFooterBtn && modal) modalCancelFooterBtn.onclick = () => modal.close();

  /**
   * Render schema form fields into `#crud-fields`.
   * @param {any[]} fields
   * @param {Record<string, any>} currentData
   */
  function renderFields(fields, currentData) {
    const container = document.getElementById('crud-fields');
    if (!container) return;
    container.innerHTML = fields
      .map(f => {
        const rawVal =
          currentData[f.name] !== undefined
            ? currentData[f.name]
            : f.default !== undefined
              ? f.default
              : '';
        const val =
          f.type === 'date' && /^\d{4}-\d{2}-\d{2}/.test(String(rawVal || ''))
            ? String(rawVal).slice(0, 10)
            : rawVal;
        const isSectionKeyEdit = PMSAdmin.currentEditingId && f.name === 'section_key';

        if (f.type === 'textarea') {
          return `
            <div class="form-group">
              <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
              <textarea id="f-${f.name}" class="form-control" rows="${f.rows || 3}" ${f.required ? 'required' : ''}>${PMS.escHtml(val)}</textarea>
            </div>`;
        }
        if (f.type === 'checkbox') {
          return `
            <div class="form-group" style="display:flex;align-items:center;gap:.75rem">
              <input type="checkbox" id="f-${f.name}" ${val ? 'checked' : ''} style="width:18px;height:18px">
              <label class="form-label" for="f-${f.name}" style="margin:0;cursor:pointer">${PMS.escHtml(f.label)}</label>
            </div>`;
        }
        if (f.type === 'select') {
          const optsList = [...f.options];
          // If current value is not in options, prepend it so editing never silently changes it
          if (
            val !== '' &&
            val !== null &&
            val !== undefined &&
            !optsList.map(String).includes(String(val))
          ) {
            optsList.unshift(val);
          }
          const opts = optsList
            .map(
              o =>
                `<option value="${PMS.escHtml(o)}" ${String(val) === String(o) ? 'selected' : ''}>${PMS.escHtml(o)}</option>`
            )
            .join('');
          return `
            <div class="form-group">
              <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
              <select id="f-${f.name}" class="form-control">${opts}</select>
            </div>`;
        }
        if (f.upload) {
          return `
            <div class="form-group">
              <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
              <div style="display:flex;gap:.5rem;align-items:center">
                <input type="${f.type || 'text'}" id="f-${f.name}" class="form-control" value="${PMS.escHtml(val)}" placeholder="${PMS.escHtml(f.placeholder || '')}" ${f.required ? 'required' : ''} style="flex:1">
                <label class="btn btn-outline btn-sm inline-upload-label" style="margin:0;white-space:nowrap;cursor:pointer;padding:.4rem .7rem">
                  <span class="upload-status-text">Upload</span>
                  <input type="file" data-target-input="f-${f.name}" class="inline-file-input">
                </label>
              </div>
            </div>`;
        }
        return `
          <div class="form-group">
            <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
            <input type="${f.type || 'text'}" id="f-${f.name}" class="form-control" value="${PMS.escHtml(val)}" placeholder="${PMS.escHtml(f.placeholder || '')}" ${f.required ? 'required' : ''} ${isSectionKeyEdit ? 'readonly style="opacity:0.7;cursor:not-allowed"' : ''}>
          </div>`;
      })
      .join('');
  }

  /**
   * Open the CRUD modal in creation mode.
   * @param {string} tab
   * @param {HTMLElement} [triggerEl]
   */
  function openCreateModal(tab, triggerEl) {
    const schema = PMSAdmin.SCHEMAS[tab];
    PMSAdmin.currentEditingId = null;
    const title = document.getElementById('modal-title');
    if (title) title.textContent = `New ${schema.singular}`;
    const msg = document.getElementById('crud-msg');
    if (msg) msg.innerHTML = '';
    renderFields(schema.fields, {});
    PMS.showModalWithFocusReturn(modal, triggerEl);
  }

  /**
   * Open the CRUD modal pre-filled for an existing row.
   * @param {string} tab
   * @param {string} id
   * @param {HTMLElement} [triggerEl]
   */
  function openEditModal(tab, id, triggerEl) {
    const schema = PMSAdmin.SCHEMAS[tab];
    const item = PMSAdmin.currentCache.find(x => String(x.id) === String(id));
    if (!item) return;
    PMSAdmin.currentEditingId = id;
    const title = document.getElementById('modal-title');
    if (title) title.textContent = `Edit ${schema.singular} (#${id})`;
    const msg = document.getElementById('crud-msg');
    if (msg) msg.innerHTML = '';
    renderFields(schema.fields, item);
    PMS.showModalWithFocusReturn(modal, triggerEl);
  }

  /**
   * Open the read-only record inspection modal.
   * @param {string} tab
   * @param {string} id
   * @param {HTMLElement} [triggerEl]
   */
  function openViewModal(tab, id, triggerEl) {
    const item = PMSAdmin.currentCache.find(x => String(x.id) === String(id));
    if (!item) return;
    const schema = PMSAdmin.SCHEMAS[tab];
    const viewModal = document.getElementById('view-modal');
    if (!viewModal) return;
    const title = document.getElementById('view-modal-title');
    if (title) title.textContent = `${schema ? schema.singular : 'Record'} #${id} Full Details`;
    const container = document.getElementById('view-modal-content');
    if (container) {
      container.innerHTML = Object.entries(item)
        .map(
          ([k, v]) => `
        <div style="border-bottom:1px solid var(--border);padding:.6rem 0">
          <div style="font-size:.75rem;text-transform:uppercase;letter-spacing:.5px;color:var(--text-muted);font-weight:600;margin-bottom:.25rem">${PMS.escHtml(k.replace(/_/g, ' '))}</div>
          <div style="font-size:.9rem;color:var(--text);white-space:pre-wrap;word-break:break-word">${v !== null && v !== undefined && v !== '' ? PMS.escHtml(String(v)) : '<span style="color:var(--text-muted)">—</span>'}</div>
        </div>
      `
        )
        .join('');
    }
    PMS.showModalWithFocusReturn(viewModal, triggerEl);
  }

  const viewModal = document.getElementById('view-modal');
  if (viewModal) {
    PMS.enableBackdropClose(viewModal);
    const closeBtn = document.getElementById('view-modal-close-btn');
    const closeFooterBtn = document.getElementById('view-modal-close-footer-btn');
    if (closeBtn) closeBtn.onclick = () => viewModal.close();
    if (closeFooterBtn) closeFooterBtn.onclick = () => viewModal.close();
  }

  // Inline file uploader listener for crud modal
  const crudFields = document.getElementById('crud-fields');
  if (crudFields) {
    crudFields.addEventListener('change', async ev => {
      const fileInput = ev.target.closest('.inline-file-input');
      if (!fileInput || !fileInput.files || !fileInput.files[0]) return;
      const file = fileInput.files[0];
      const targetId = fileInput.dataset.targetInput;
      const labelEl = fileInput.closest('.inline-upload-label');
      const statusSpan = labelEl ? labelEl.querySelector('.upload-status-text') : null;
      const expectedEditingId = PMSAdmin.currentEditingId;
      const expectedTab = PMSAdmin.currentTab;

      if (statusSpan) statusSpan.textContent = 'Uploading…';
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
        if (
          modal &&
          modal.open &&
          PMSAdmin.currentEditingId === expectedEditingId &&
          PMSAdmin.currentTab === expectedTab
        ) {
          const targetEl = document.getElementById(targetId);
          if (targetEl) targetEl.value = data.url;
          if (statusSpan) statusSpan.textContent = 'Uploaded';
        }
      } catch (err) {
        if (
          modal &&
          modal.open &&
          PMSAdmin.currentEditingId === expectedEditingId &&
          PMSAdmin.currentTab === expectedTab
        ) {
          PMS.toast(`Upload failed: ${err.message}`, 'error');
          if (statusSpan) statusSpan.textContent = 'Upload';
        }
      } finally {
        fileInput.value = '';
      }
    });
  }

  const crudForm = document.getElementById('crud-form');
  if (crudForm) {
    crudForm.addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('modal-save-btn');
      const msg = document.getElementById('crud-msg');
      const schema = PMSAdmin.SCHEMAS[PMSAdmin.currentTab];
      if (!schema) return;

      const payload = {};
      const missingFields = [];
      let firstMissingLabel = '';
      for (const f of schema.fields) {
        const el = document.getElementById(`f-${f.name}`);
        if (!el) continue;
        if (f.type === 'checkbox') {
          payload[f.name] = el.checked;
        } else if (f.type === 'number') {
          const parsed = parseInt(el.value, 10);
          payload[f.name] = el.value === '' || isNaN(parsed) ? null : parsed;
        } else {
          const trimmed = el.value.trim();
          if (
            trimmed === '' &&
            PMSAdmin.currentEditingId &&
            PMSAdmin.currentTab === 'sections' &&
            ['subtitle', 'badge', 'extra_json'].includes(f.name)
          ) {
            payload[f.name] = '';
          } else {
            payload[f.name] = trimmed === '' ? null : trimmed;
          }
        }
        if (f.required && (payload[f.name] === null || payload[f.name] === '')) {
          missingFields.push(el);
          if (!firstMissingLabel) firstMissingLabel = f.label.replace('*', '').trim();
        }
      }
      if (missingFields.length) {
        PMS.reportInvalidFields(crudForm, missingFields, msg);
        PMS.showAlert(msg, `Please fill out required field: ${firstMissingLabel}`);
        return;
      }

      // Client-Side extra_json Validation
      if (payload.extra_json) {
        try {
          JSON.parse(payload.extra_json);
        } catch (jsonErr) {
          PMS.reportInvalidFields(crudForm, [document.getElementById('f-extra_json')], msg);
          PMS.showAlert(msg, `Invalid JSON in Extra JSON field: ${jsonErr.message}`);
          return;
        }
      }
      PMS.reportInvalidFields(crudForm, []);

      PMS.setLoading(btn, true);
      try {
        if (PMSAdmin.currentEditingId) {
          await PMS.apiPut(`${schema.api}/${PMSAdmin.currentEditingId}`, payload);
        } else {
          await PMS.apiPost(schema.api, payload);
        }
        if (modal) modal.close();
        PMSAdmin.loadSection(PMSAdmin.currentTab);
      } catch (err) {
        if (PMSAdmin.isUnauthorized(err)) {
          PMSAdmin.showLogin();
          return;
        }
        PMS.reportError('crud-save', err);
        PMS.showAlert(msg, err.message || 'Operation failed.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }

  window.PMSAdmin = Object.assign(window.PMSAdmin || {}, {
    openCreateModal,
    openEditModal,
    openViewModal,
    renderFields,
  });
})();
