/**
 * PIEAS Materials Society (PMS)
 * Membership & Contact Page Logic
 * frontend/static/assets/pages/join.js
 */

const joinForm = document.getElementById('join-form');
if (joinForm) {
  joinForm.addEventListener('submit', async ev => {
    ev.preventDefault();
    const btn = document.getElementById('join-submit');
    const msg = document.getElementById('join-msg');

    const fullName = document.getElementById('join-name').value.trim();
    const rollNumber = document.getElementById('join-roll').value.trim();
    const batch = document.getElementById('join-batch').value.trim();
    const department = document.getElementById('join-dept').value.trim();
    const email = document.getElementById('join-email').value.trim();

    if (!fullName || !rollNumber || !batch || !department || !email) {
      PMS.showAlert(msg, 'Please complete all required fields (*).');
      return;
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      PMS.showAlert(msg, 'Please provide a valid email address.');
      return;
    }

    PMS.setLoading(btn, true);
    try {
      await PMS.apiPost('/api/membership', {
        full_name:   fullName,
        roll_number: rollNumber,
        batch:       batch,
        department:  department,
        email:       email,
        phone:       document.getElementById('join-phone').value.trim() || null,
        interests:   document.getElementById('join-interests').value.trim() || null,
        why_join:    document.getElementById('join-why').value.trim() || null,
      });
      PMS.showAlert(msg, '🎉 Application submitted successfully! The executive council will review your submission.', 'success');
      joinForm.reset();
    } catch(e) {
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

    const name = document.getElementById('contact-name').value.trim();
    const email = document.getElementById('contact-email').value.trim();
    const message = document.getElementById('contact-message').value.trim();

    if (!name || !email || !message) {
      PMS.showAlert(msg, 'Please fill in your name, email, and message.');
      return;
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      PMS.showAlert(msg, 'Please enter a valid email address.');
      return;
    }

    PMS.setLoading(btn, true);
    try {
      await PMS.apiPost('/api/contact', {
        name:    name,
        email:   email,
        subject: document.getElementById('contact-subject').value.trim() || null,
        message: message,
      });
      PMS.showAlert(msg, '✉️ Your message has been sent to PMS administration.', 'success');
      contactForm.reset();
    } catch(e) {
      PMS.showAlert(msg, e.message || 'Message could not be sent. Please try again.');
    } finally {
      PMS.setLoading(btn, false);
    }
  });
}

