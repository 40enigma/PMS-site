/**
 * PIEAS Materials Society (PMS)
 * Membership & Contact Page Logic
 * frontend/static/assets/pages/join.js
 */

(() => {
  'use strict';

  /**
   * Read a trimmed value without assuming the element exists.
   * @param {string} id
   * @returns {string}
   */
  const valueOf = id => {
    const el = /** @type {HTMLInputElement | HTMLTextAreaElement | null} */ (
      document.getElementById(id)
    );
    return el ? el.value.trim() : '';
  };

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

      const fullName = valueOf('join-name');
      const rollNumber = valueOf('join-roll');
      const batch = valueOf('join-batch');
      const department = valueOf('join-dept');
      const email = valueOf('join-email');

      const fieldEls = [nameEl, rollEl, batchEl, deptEl, emailEl];
      const missing = fieldEls.filter(
        el => !el || !(/** @type {HTMLInputElement} */ (el).value.trim())
      );
      if (missing.length) {
        PMS.reportInvalidFields(joinForm, missing, msg);
        PMS.showAlert(msg, 'Please complete all required fields (*).');
        return;
      }
      if (!PMS.isEmail(email)) {
        PMS.reportInvalidFields(joinForm, [emailEl], msg);
        PMS.showAlert(msg, 'Please provide a valid email address.');
        return;
      }
      PMS.reportInvalidFields(joinForm, []);

      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/membership', {
          full_name: fullName,
          roll_number: rollNumber,
          batch,
          department,
          email,
          phone: valueOf('join-phone') || null,
          interests: valueOf('join-interests') || null,
          why_join: valueOf('join-why') || null,
        });
        PMS.showAlert(
          msg,
          'Application submitted. The executive council will review your submission.',
          'success'
        );
        joinForm.reset();
      } catch (err) {
        PMS.reportError('membership', err);
        PMS.showAlert(msg, err.message || 'Submission failed. Please try again.');
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
      const messageEl = document.getElementById('contact-message');

      const name = valueOf('contact-name');
      const email = valueOf('contact-email');
      const message = valueOf('contact-message');

      const fieldEls = [nameEl, emailEl, messageEl];
      const missing = fieldEls.filter(
        el => !el || !(/** @type {HTMLInputElement} */ (el).value.trim())
      );
      if (missing.length) {
        PMS.reportInvalidFields(contactForm, missing, msg);
        PMS.showAlert(msg, 'Please fill in your name, email, and message.');
        return;
      }
      if (!PMS.isEmail(email)) {
        PMS.reportInvalidFields(contactForm, [emailEl], msg);
        PMS.showAlert(msg, 'Please enter a valid email address.');
        return;
      }
      PMS.reportInvalidFields(contactForm, []);

      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/contact', {
          name,
          email,
          subject: valueOf('contact-subject') || null,
          message,
        });
        PMS.showAlert(msg, 'Message sent to society administration.', 'success');
        contactForm.reset();
      } catch (err) {
        PMS.reportError('contact', err);
        PMS.showAlert(msg, err.message || 'Message could not be sent. Please try again.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }
})();
