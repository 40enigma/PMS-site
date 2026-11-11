/**
 * PIEAS Materials Society (PMS)
 * Events Page Logic
 * frontend/static/assets/pages/events.js
 */

(() => {
  'use strict';

  let eventsLoadSeq = 0;

  async function loadEvents(isPast) {
    const req = ++eventsLoadSeq;
    const grid = document.getElementById('events-grid');
    if (!grid) return;
    grid.setAttribute('aria-busy', 'true');
    grid.innerHTML = PMS.skeletonGrid(3);
    try {
      const qs = isPast !== null ? `?is_past=${isPast}` : '';
      const events = await PMS.apiFetch(`/api/events${qs}`);
      if (req !== eventsLoadSeq) return;
      grid.innerHTML = events.length
        ? events.map((e, idx) => PMS.eventCard(e, idx)).join('')
        : PMS.emptyState('No events found in this category.');
      PMS.observeFadeElements(grid);
    } catch (_) {
      if (req !== eventsLoadSeq) return;
      grid.innerHTML = PMS.errorState('Could not load events.');
    } finally {
      if (req === eventsLoadSeq) grid.setAttribute('aria-busy', 'false');
    }
  }

  // Note: Event registration modal & delegation handled globally by PMS.openRegistrationModal in app.js

  document.querySelectorAll('.filter-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.filter-btn').forEach(b => {
        const isActive = b === btn;
        b.classList.toggle('active', isActive);
        b.setAttribute('aria-pressed', String(isActive));
      });
      const f = btn.dataset.filter;
      loadEvents(f === '' ? null : f === 'true');
    });
  });

  const verifyForm = document.getElementById('verify-form');
  if (verifyForm) {
    verifyForm.addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('verify-submit');
      const msgEl = document.getElementById('verify-msg');
      const codeInput = document.getElementById('verify-code');
      const code = codeInput ? codeInput.value.trim() : '';
      if (!code) {
        PMS.reportInvalidFields(verifyForm, [codeInput]);
        PMS.showAlert(msgEl, 'Please enter a certificate verification code.');
        return;
      }
      PMS.reportInvalidFields(verifyForm, []);
      PMS.setLoading(btn, true);
      try {
        const res = await PMS.apiFetch(`/api/certificates/verify?code=${encodeURIComponent(code)}`);
        if (res.valid) {
          PMS.showAlert(
            msgEl,
            `<strong>Digitally verified</strong><br>Participant: <strong>${PMS.escHtml(res.student_name)}</strong><br>Event: ${PMS.escHtml(res.event_title)}<br>Certificate code: <code class="mono">${PMS.escHtml(res.certificate_code)}</code>`,
            'success',
            true
          );
        } else if (res.reason === 'not_attended') {
          PMS.showAlert(
            msgEl,
            'Registered, but attendance has not been confirmed yet. Contact the organizers.',
            'error'
          );
        } else {
          PMS.showAlert(
            msgEl,
            'Certificate code not found. Please verify the code or contact DMME administration.',
            'error'
          );
        }
      } catch (e) {
        PMS.showAlert(msgEl, e.message || 'Verification service error.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }

  loadEvents(null);
})();
