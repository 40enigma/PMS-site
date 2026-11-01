/**
 * PIEAS Materials Society (PMS)
 * Home Page Logic
 * frontend/static/assets/pages/index.js
 */

(async () => {
  // 1. Fetch Dynamic Site Sections (home_hero, home_activities)
  try {
    const sections = await PMS.apiFetch('/api/sections');
    const heroSec = sections.find(s => s.section_key === 'home_hero');
    if (heroSec) {
      if (heroSec.badge) {
        const badgeEl = document.getElementById('hero-badge');
        if (badgeEl) badgeEl.textContent = heroSec.badge;
      }
      if (heroSec.title) {
        const titleEl = document.getElementById('hero-heading');
        if (titleEl) titleEl.textContent = heroSec.title;
      }
      if (heroSec.subtitle) {
        const subEl = document.getElementById('hero-subtitle');
        if (subEl) {
          subEl.textContent = heroSec.subtitle;
          subEl.style.display = 'block';
        }
      }
      if (heroSec.content) {
        const descEl = document.getElementById('hero-desc');
        if (descEl) descEl.textContent = heroSec.content;
      }
      if (heroSec.extra_json) {
        try {
          const cfg = JSON.parse(heroSec.extra_json);
          const actionsEl = document.getElementById('hero-actions');
          if (actionsEl) {
            actionsEl.innerHTML = `
              ${cfg.cta_primary_url ? `<a href="${PMS.safeUrl(cfg.cta_primary_url)}" class="btn btn-primary">${PMS.escHtml(cfg.cta_primary_text || 'Join PMS →')}</a>` : ''}
              ${cfg.cta_secondary_url ? `<a href="${PMS.safeUrl(cfg.cta_secondary_url)}" class="btn btn-outline">${PMS.escHtml(cfg.cta_secondary_text || 'Student Resources')}</a>` : ''}
            `;
          }
        } catch (_) {}
      }
    }

    const actSec = sections.find(s => s.section_key === 'home_activities');
    const actGrid = document.getElementById('activities-grid');
    if (actSec && actGrid) {
      if (actSec.badge) {
        const badgeEl = document.getElementById('activities-badge');
        if (badgeEl) badgeEl.textContent = actSec.badge;
      }
      if (actSec.title) {
        const titleEl = document.getElementById('activities-heading');
        if (titleEl) titleEl.textContent = actSec.title;
      }
      if (actSec.subtitle) {
        const subEl = document.getElementById('activities-subtitle');
        if (subEl) {
          subEl.textContent = actSec.subtitle;
          subEl.style.display = 'block';
        }
      }
      if (actSec.content) {
        const descEl = document.getElementById('activities-desc');
        if (descEl) descEl.textContent = actSec.content;
      }

      let rendered = false;
      if (actSec.extra_json) {
        try {
          const cards = JSON.parse(actSec.extra_json);
          if (Array.isArray(cards) && cards.length > 0) {
            const icons = { Atom: '⚛️', Cpu: '🔬', Award: '🏆', Building: '🏭' };
            actGrid.innerHTML = cards.map(c => `
              <article class="card fade-up">
                <div style="font-size:2rem;margin-bottom:.75rem">${icons[c.icon] || '✨'}</div>
                <h3 style="font-size:1.1rem;margin-bottom:.5rem">${PMS.escHtml(c.title)}</h3>
                <p style="color:var(--text-muted);font-size:.875rem;line-height:1.6">${PMS.escHtml(c.desc)}</p>
              </article>
            `).join('');
            PMS.observeFadeElements(actGrid);
            rendered = true;
          }
        } catch (_) {}
      }
      if (!rendered) {
        actGrid.innerHTML = '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">Activities configured in site sections.</p>';
      }
    } else if (actGrid) {
      actGrid.innerHTML = '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">Activities information will be announced soon.</p>';
    }
  } catch (_) {
    const actGrid = document.getElementById('activities-grid');
    if (actGrid) actGrid.innerHTML = '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">Activities information will be announced soon.</p>';
  }

  // 2. Load featured events
  const eventsGrid = document.getElementById('events-grid');
  if (eventsGrid) {
    eventsGrid.innerHTML = PMS.skeletonGrid(3);
    try {
      const events = await PMS.apiFetch('/api/events?is_featured=true&is_past=false');
      eventsGrid.innerHTML = events.length
        ? events.slice(0, 3).map((e, idx) => PMS.eventCard(e, idx)).join('')
        : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No upcoming featured events listed at this time.</p>';
      PMS.observeFadeElements(eventsGrid);
    } catch(e) {
      eventsGrid.innerHTML = `<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">Could not load events.</p>`;
    }
  }

  // 3. Load latest technical posts
  const postsGrid = document.getElementById('posts-grid');
  if (postsGrid) {
    postsGrid.innerHTML = PMS.skeletonGrid(3);
    try {
      const posts = await PMS.apiFetch('/api/posts');
      postsGrid.innerHTML = posts.length
        ? posts.slice(0, 3).map((p, idx) => PMS.postCard(p, idx)).join('')
        : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No articles published yet.</p>';
      PMS.observeFadeElements(postsGrid);
    } catch(e) {
      postsGrid.innerHTML = `<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">Could not load posts.</p>`;
    }
  }

  // 4. Newsletter form with client validation
  const newsletterForm = document.getElementById('newsletter-form');
  if (newsletterForm) {
    newsletterForm.addEventListener('submit', async (ev) => {
      ev.preventDefault();
      const btn = document.getElementById('newsletter-submit');
      const msgEl = document.getElementById('newsletter-msg');
      const emailInput = document.getElementById('newsletter-email');
      const email = emailInput ? emailInput.value.trim() : '';
      if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
        PMS.showAlert(msgEl, 'Please enter a valid email address.');
        return;
      }
      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/newsletter', { email });
        PMS.showAlert(msgEl, '🎉 Thank you for subscribing to the PMS Bulletin!', 'success');
        if (emailInput) emailInput.value = '';
      } catch(e) {
        PMS.showAlert(msgEl, e.message || 'Could not subscribe. Please try again.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }
})();

