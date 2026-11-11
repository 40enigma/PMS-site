/**
 * PIEAS Materials Society (PMS)
 * Admin Tablist Navigation, Data Table Loader, CSV Export, and Row Actions
 * frontend/static/assets/pages/admin/table.js
 */
(() => {
  'use strict';

  /**
   * Activate an admin tab and load its data section.
   * @param {HTMLElement} btn
   * @param {boolean} [shouldFocus]
   */
  function activateAdminTab(btn, shouldFocus = false) {
    if (!btn) return;
    const tabs = Array.from(document.querySelectorAll('.admin-nav [role="tab"]'));
    tabs.forEach(b => {
      const isSelected = b === btn;
      b.classList.toggle('active', isSelected);
      b.setAttribute('aria-selected', String(isSelected));
      b.setAttribute('tabindex', isSelected ? '0' : '-1');
    });
    const contentEl = document.getElementById('admin-content');
    if (contentEl && btn.id) {
      contentEl.setAttribute('aria-labelledby', btn.id);
    }
    if (shouldFocus) btn.focus();
    PMSAdmin.currentTab = btn.dataset.section;
    loadSection(PMSAdmin.currentTab);
  }

  const adminTablist = document.querySelector('.admin-nav[role="tablist"]');
  if (adminTablist) {
    const tabs = Array.from(adminTablist.querySelectorAll('[role="tab"]'));
    tabs.forEach(btn => {
      btn.addEventListener('click', () => activateAdminTab(btn, false));
    });
    adminTablist.addEventListener('keydown', ev => {
      const tabsList = Array.from(adminTablist.querySelectorAll('[role="tab"]'));
      const curIdx = tabsList.indexOf(document.activeElement);
      if (curIdx === -1 || !tabsList.length) return;
      let nextIdx = -1;
      if (ev.key === 'ArrowRight') nextIdx = (curIdx + 1) % tabsList.length;
      else if (ev.key === 'ArrowLeft') nextIdx = (curIdx - 1 + tabsList.length) % tabsList.length;
      else if (ev.key === 'Home') nextIdx = 0;
      else if (ev.key === 'End') nextIdx = tabsList.length - 1;
      if (nextIdx !== -1) {
        ev.preventDefault();
        activateAdminTab(tabsList[nextIdx], true);
      }
    });
  }

  /**
   * Fetch and render an admin section table or tool view.
   * @param {string} tab
   */
  async function loadSection(tab) {
    const req = ++PMSAdmin.loadSeq;
    const content = document.getElementById('admin-content');
    const toolbar = document.getElementById('toolbar-actions');
    if (!content) return;
    let savedRegFilter = '';
    if (tab === 'registrations') {
      const existingFilter = document.getElementById('reg-event-filter');
      if (existingFilter) savedRegFilter = existingFilter.value;
    }
    content.setAttribute('aria-busy', 'true');
    content.innerHTML =
      '<div class="spinner" role="status" aria-live="polite" aria-label="Loading section"></div>';
    if (toolbar) toolbar.innerHTML = '';

    try {
      if (tab === 'stats') {
        await PMSAdmin.renderStatsEditor(content);
        return;
      }
      if (tab === 'upload') {
        PMSAdmin.renderUploadTool(content);
        return;
      }

      const schema = PMSAdmin.SCHEMAS[tab];
      if (!schema) return;

      if (schema.canCreate && toolbar) {
        toolbar.innerHTML = `<button class="btn btn-primary btn-sm" id="btn-create-record">＋ New ${PMS.escHtml(schema.singular)}</button>`;
        const createRecordBtn = document.getElementById('btn-create-record');
        if (createRecordBtn)
          createRecordBtn.onclick = () => PMSAdmin.openCreateModal(tab, createRecordBtn);
      }

      if (tab === 'registrations') {
        try {
          const evts = await PMS.apiFetch('/api/admin/events');
          if (req !== PMSAdmin.loadSeq) return;
          PMSAdmin.eventsMap = {};
          evts.forEach(e => {
            PMSAdmin.eventsMap[e.id] = e.title;
          });
        } catch (_) { }
      }

      const data = PMSAdmin.PAGINATED_TABS.has(tab)
        ? await PMSAdmin.fetchAllRows(tab, schema.api)
        : await PMS.apiFetch(schema.api);
      if (req !== PMSAdmin.loadSeq) return;
      PMSAdmin.currentCache = data;

      if (!data.length) {
        content.innerHTML = PMS.emptyState(`No records found in ${schema.title}.`);
        return;
      }

      if (tab === 'registrations' && toolbar) {
        const uniqueEvents = Object.keys(PMSAdmin.eventsMap).map(id => ({
          id,
          title: PMSAdmin.eventsMap[id],
        }));
        const opts = uniqueEvents
          .map(e => `<option value="${e.id}">${PMS.escHtml(e.title)}</option>`)
          .join('');
        toolbar.innerHTML = `
          <div style="display:flex;gap:.75rem;align-items:center;flex-wrap:wrap">
            <select id="reg-event-filter" class="form-control" style="width:auto;padding:.35rem .75rem;font-size:.85rem">
              <option value="">All Events (${data.length})</option>
              ${opts}
            </select>
            <button class="btn btn-outline btn-sm" id="btn-export-csv" style="display:inline-flex;align-items:center;gap:6px">${PMS.icon('download', 13)} Export CSV</button>
          </div>
        `;
      }

      const renderRow = item => {
        const tds = schema.columns
          .map(c => {
            const val = item[c];
            if (c === 'event_id' && tab === 'registrations') {
              const t = PMSAdmin.eventsMap[val] || `Event #${val}`;
              return `<td><span title="${PMS.escHtml(t)}" style="font-weight:600;color:var(--text)">${PMS.escHtml(t)}</span></td>`;
            }
            if (c === 'attended' && tab === 'registrations') {
              return `<td><span class="status-badge ${val ? 'status-approved' : 'status-rejected'}">${val ? 'Attended' : 'Absent'}</span></td>`;
            }
            if (c === 'status' && tab === 'membership') {
              const s = String(val).toLowerCase();
              const cls =
                s === 'approved'
                  ? 'status-approved'
                  : s === 'rejected'
                    ? 'status-rejected'
                    : 'status-pending';
              return `<td><span class="status-badge ${cls}">${PMS.escHtml(String(val))}</span></td>`;
            }
            if (typeof val === 'boolean') return `<td>${val ? 'Yes' : '—'}</td>`;
            if (val === null || val === undefined) return '<td>—</td>';
            const str = String(val);
            const display = str.length > 50 ? str.slice(0, 48) + '…' : str;
            return `<td>${PMS.escHtml(display)}</td>`;
          })
          .join('');

        let actions = '';
        if (tab === 'membership') {
          actions = `
            <button class="action-btn btn-outline" data-action="view" data-id="${item.id}">View</button>
            <button class="action-btn btn-success" data-action="status" data-id="${item.id}" data-status="Approved">Approve</button>
            <button class="action-btn btn-danger" data-action="status" data-id="${item.id}" data-status="Rejected">Reject</button>
            <button class="action-btn" data-action="status" data-id="${item.id}" data-status="Pending">Reset</button>
            <button class="action-btn btn-danger" data-action="delete" data-id="${item.id}">Delete</button>
          `;
        } else if (tab === 'registrations') {
          actions = `
            <button class="action-btn btn-outline" data-action="view" data-id="${item.id}">View</button>
            <button class="action-btn ${item.attended ? 'btn-danger' : 'btn-success'}" data-action="toggle-attended" data-id="${item.id}" data-attended="${item.attended ? 'true' : 'false'}">${item.attended ? 'Mark Absent' : 'Mark Attended'}</button>
            <button class="action-btn btn-danger" data-action="delete" data-id="${item.id}">Delete</button>
          `;
        } else {
          if (tab === 'contacts') {
            actions += `<button class="action-btn btn-outline" data-action="view" data-id="${item.id}">View</button>`;
          }
          if (schema.canEdit !== false) {
            actions += `<button class="action-btn btn-primary" data-action="edit" data-id="${item.id}">Edit</button>`;
          }
          actions += `<button class="action-btn btn-danger" data-action="delete" data-id="${item.id}">Delete</button>`;
        }

        return `<tr>${tds}<td><div style="display:flex;flex-wrap:nowrap;gap:.25rem">${actions}</div></td></tr>`;
      };

      const ths = schema.columns.map(c => `<th>${PMS.escHtml(c.replace(/_/g, ' '))}</th>`).join('');

      content.innerHTML = `
        <div style="overflow-x:auto">
          <table>
            <thead><tr>${ths}<th>Actions</th></tr></thead>
            <tbody id="table-body">${data.map(renderRow).join('')}</tbody>
          </table>
        </div>`;

      if (tab === 'registrations') {
        let displayed = data;
        const filterEl = document.getElementById('reg-event-filter');
        const exportBtn = document.getElementById('btn-export-csv');

        if (filterEl) {
          filterEl.onchange = () => {
            const val = filterEl.value;
            displayed = val ? data.filter(d => String(d.event_id) === String(val)) : data;
            const tbody = document.getElementById('table-body');
            if (tbody) tbody.innerHTML = displayed.map(renderRow).join('');
          };
          if (savedRegFilter) {
            filterEl.value = savedRegFilter;
            filterEl.dispatchEvent(new Event('change'));
          }
        }

        if (exportBtn) {
          exportBtn.onclick = () => {
            if (!displayed.length) {
              alert('No registrations to export.');
              return;
            }
            const headers = [
              'ID',
              'Event ID',
              'Event Title',
              'Student Name',
              'Roll Number',
              'Batch',
              'Department',
              'Email',
              'Phone',
              'Certificate Code',
              'Attended',
              'Created At',
            ];
            const rows = displayed.map(r => [
              r.id,
              r.event_id,
              PMSAdmin.eventsMap[r.event_id] || `Event #${r.event_id}`,
              r.student_name,
              r.roll_number,
              r.batch,
              r.department,
              r.email,
              r.phone || '',
              r.certificate_code || '',
              r.attended ? 'Yes' : 'No',
              r.created_at || '',
            ]);

            // Neutralize formula injection with single quote prefix if cell starts with = + - @ \t \r
            const csv = [headers, ...rows]
              .map(row =>
                row
                  .map(cell => {
                    let str = String(cell ?? '');
                    if (/^[=+\-@\t\r]/.test(str)) {
                      str = "'" + str;
                    }
                    return `"${str.replace(/"/g, '""')}"`;
                  })
                  .join(',')
              )
              .join('\r\n');

            const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `pms_event_registrations_${new Date().toISOString().slice(0, 10)}.csv`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
          };
        }
      }
    } catch (err) {
      if (req !== PMSAdmin.loadSeq) return;
      if (PMSAdmin.isUnauthorized(err)) {
        PMSAdmin.showLogin();
      } else {
        content.innerHTML = PMS.errorState(`Error loading records: ${err.message}`);
      }
    } finally {
      if (req === PMSAdmin.loadSeq) content.setAttribute('aria-busy', 'false');
    }
  }

  /* ── Event Delegation for Table Action Buttons ── */
  const adminContent = document.getElementById('admin-content');
  if (adminContent) {
    adminContent.addEventListener('click', async ev => {
      const btn = ev.target.closest('[data-action]');
      if (!btn) return;
      const action = btn.dataset.action;
      const id = btn.dataset.id;
      const tab = PMSAdmin.currentTab;
      const schema = PMSAdmin.SCHEMAS[tab];

      if (action === 'delete') {
        const confirmMsg =
          tab === 'events'
            ? 'Are you sure you want to permanently delete this event? WARNING: All registrations and issued certificates for this event will also be deleted!'
            : 'Are you sure you want to permanently delete this record?';
        if (!confirm(confirmMsg)) return;
        try {
          await PMS.apiDelete(`${schema.api}/${id}`);
          loadSection(tab);
        } catch (err) {
          if (PMSAdmin.isUnauthorized(err)) {
            PMSAdmin.showLogin();
            return;
          }
          alert('Delete failed: ' + err.message);
        }
      } else if (action === 'toggle-attended') {
        const isAttended = btn.dataset.attended === 'true';
        try {
          await PMS.apiPut(`/api/admin/registrations/${id}`, { attended: !isAttended });
          loadSection(tab);
        } catch (err) {
          if (PMSAdmin.isUnauthorized(err)) {
            PMSAdmin.showLogin();
            return;
          }
          alert('Attendance update failed: ' + err.message);
        }
      } else if (action === 'edit') {
        PMSAdmin.openEditModal(tab, id, btn);
      } else if (action === 'view') {
        PMSAdmin.openViewModal(tab, id, btn);
      } else if (action === 'status') {
        const status = btn.dataset.status;
        try {
          await PMS.apiPut(`/api/admin/membership/${id}`, { status });
          loadSection(tab);
        } catch (err) {
          if (PMSAdmin.isUnauthorized(err)) {
            PMSAdmin.showLogin();
            return;
          }
          alert('Status update failed: ' + err.message);
        }
      }
    });
  }

  window.PMSAdmin = Object.assign(window.PMSAdmin || {}, {
    activateAdminTab,
    loadSection,
  });
})();
