/**
 * PIEAS Materials Society (PMS)
 * Events Page Logic
 * frontend/static/assets/pages/events.js
 */

async function loadEvents(isPast) {
  const grid = document.getElementById('events-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(3);
  try {
    const qs = isPast !== null ? `?is_past=${isPast}` : '';
    const events = await PMS.apiFetch(`/api/events${qs}`);
    grid.innerHTML = events.length
      ? events.map((e, idx) => PMS.eventCard(e, idx)).join('')
      : '<p style="grid-column:1/-1;text-align:center;color:var(--text-muted);padding:2rem 0">No events found in this category.</p>';
    PMS.observeFadeElements(grid);
  } catch(err) {
    grid.innerHTML = `<p style="color:var(--danger);grid-column:1/-1;text-align:center">Failed to load events.</p>`;
  }
}

// Note: Event registration modal & delegation handled globally by PMS.openRegistrationModal in app.js

document.querySelectorAll('.filter-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    const f = btn.dataset.filter;
    loadEvents(f === '' ? null : f === 'true');
  });
});

const verifyForm = document.getElementById('verify-form');
if (verifyForm) {
  verifyForm.addEventListener('submit', async (ev) => {
    ev.preventDefault();
    const btn = document.getElementById('verify-submit');
    const msgEl = document.getElementById('verify-msg');
    const code = document.getElementById('verify-code').value.trim();
    if (!code) { PMS.showAlert(msgEl, 'Please enter a certificate verification code.'); return; }
    PMS.setLoading(btn, true);
    try {
      const res = await PMS.apiFetch(`/api/certificates/verify?code=${encodeURIComponent(code)}`);
      if (res.valid) {
        PMS.showAlert(msgEl, `<strong>Digitally verified</strong><br>Participant: <strong>${PMS.escHtml(res.student_name)}</strong><br>Event: ${PMS.escHtml(res.event_title)}<br>Certificate code: <code class="mono">${PMS.escHtml(res.certificate_code)}</code>`, 'success', true);
      } else {
        PMS.showAlert(msgEl, 'Certificate code not found. Please verify the code or contact DMME administration.', 'error');
      }
    } catch(e) {
      PMS.showAlert(msgEl, e.message || 'Verification service error.');
    } finally {
      PMS.setLoading(btn, false);
    }
  });
}

loadEvents(null);

