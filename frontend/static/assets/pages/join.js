/**
 * PIEAS Materials Society (PMS)
 * Membership & Contact Page Logic
 * frontend/static/assets/pages/join.js
 */

(() => {
  'use strict';

  const joinForm = document.getElementById('join-form');
  if (joinForm) {
    joinForm.addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('join-submit');
      const msg = document.getElementById('join-msg');

      const nameEl = document.getElementById('join-name');
      const rollEl = document.getElementById('join-roll');
      const batchEl = document.getElementById('join-batch');
      const deptEl = document.getElementById('join-dept');
      const emailEl = document.getElementById('join-email');

      const fullName = nameEl.value.trim();
      const rollNumber = rollEl.value.trim();
      const batch = batchEl.value.trim();
      const department = deptEl.value.trim();
      const email = emailEl.value.trim();

      const missing = [nameEl, rollEl, batchEl, deptEl, emailEl].filter(el => !el.value.trim());
      if (missing.length) {
        PMS.reportInvalidFields(joinForm, missing);
        PMS.showAlert(msg, 'Please complete all required fields (*).');
        return;
      }
      if (!PMS.isEmail(email)) {
        PMS.reportInvalidFields(joinForm, [emailEl]);
        PMS.showAlert(msg, 'Please provide a valid email address.');
        return;
      }
      PMS.reportInvalidFields(joinForm, []);

      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/membership', {
          full_name: fullName,
          roll_number: rollNumber,
          batch: batch,
          department: department,
          email: email,
          phone: document.getElementById('join-phone').value.trim() || null,
          interests: document.getElementById('join-interests').value.trim() || null,
          why_join: document.getElementById('join-why').value.trim() || null,
        });
        PMS.showAlert(
          msg,
          'Application submitted. The executive council will review your submission.',
          'success'
        );
        joinForm.reset();
      } catch (e) {
        PMS.showAlert(msg, e.message || 'Submission failed. Please try again.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }

  const contactForm = document.getElementById('contact-form');
  if (contactForm) {
    contactForm.addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('contact-submit');
      const msg = document.getElementById('contact-msg');

      const nameEl = document.getElementById('contact-name');
      const emailEl = document.getElementById('contact-email');
      const msgEl = document.getElementById('contact-message');

      const name = nameEl.value.trim();
      const email = emailEl.value.trim();
      const message = msgEl.value.trim();

      const missing = [nameEl, emailEl, msgEl].filter(el => !el.value.trim());
      if (missing.length) {
        PMS.reportInvalidFields(contactForm, missing);
        PMS.showAlert(msg, 'Please fill in your name, email, and message.');
        return;
      }
      if (!PMS.isEmail(email)) {
        PMS.reportInvalidFields(contactForm, [emailEl]);
        PMS.showAlert(msg, 'Please enter a valid email address.');
        return;
      }
      PMS.reportInvalidFields(contactForm, []);

      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/contact', {
          name: name,
          email: email,
          subject: document.getElementById('contact-subject').value.trim() || null,
          message: message,
        });
        PMS.showAlert(msg, 'Message sent to society administration.', 'success');
        contactForm.reset();
      } catch (e) {
        PMS.showAlert(msg, e.message || 'Message could not be sent. Please try again.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }
})();
