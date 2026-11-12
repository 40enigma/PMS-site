/**
 * PIEAS Materials Society (PMS)
 * Home Page Logic
 * frontend/static/assets/pages/index.js
 */

(async () => {
  'use strict';

  const actGrid = document.getElementById('activities-grid');
  if (actGrid) actGrid.setAttribute('aria-busy', 'true');
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
              ${cfg.cta_primary_url ? `<a href="${PMS.safeUrl(cfg.cta_primary_url)}" class="btn btn-primary">${PMS.escHtml(cfg.cta_primary_text || 'Join PMS')}</a>` : ''}
              ${cfg.cta_secondary_url ? `<a href="${PMS.safeUrl(cfg.cta_secondary_url)}" class="btn btn-outline">${PMS.escHtml(cfg.cta_secondary_text || 'Student Resources')}</a>` : ''}
            `;
          }
        } catch (_) {}
      }
    }

    const actSec = sections.find(s => s.section_key === 'home_activities');
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
            const icons = { Atom: 'atom', Cpu: 'cpu', Award: 'award', Building: 'building' };
            actGrid.innerHTML = cards
              .map(
                c => `
              <article class="card fade-up">
                <div style="margin-bottom:14px;color:var(--text)">${PMS.icon(icons[c.icon] || 'layers', 22)}</div>
                <h3 style="font-size:15px;margin-bottom:8px">${PMS.escHtml(c.title)}</h3>
                <p style="color:var(--text-secondary);font-size:13.5px;line-height:1.5">${PMS.escHtml(c.desc)}</p>
              </article>
            `
              )
              .join('');
            PMS.observeFadeElements(actGrid);
            rendered = true;
          }
        } catch (_) {}
      }
      if (!rendered) {
        actGrid.innerHTML = PMS.emptyState('Activities configured in site sections.');
      }
    } else if (actGrid) {
      actGrid.innerHTML = PMS.emptyState('Activities information will be announced soon.');
    }
  } catch (_) {
    if (actGrid) actGrid.innerHTML = PMS.errorState('Could not load activities.');
  } finally {
    if (actGrid) actGrid.setAttribute('aria-busy', 'false');
  }

  // 2. Load featured events
  const eventsGrid = document.getElementById('events-grid');
  if (eventsGrid) {
    eventsGrid.setAttribute('aria-busy', 'true');
    eventsGrid.innerHTML = PMS.skeletonGrid(3);
    try {
      const events = await PMS.apiFetch('/api/events?is_featured=true&is_past=false');
      eventsGrid.innerHTML = events.length
        ? events
            .slice(0, 3)
            .map((e, idx) => PMS.eventCard(e, idx))
            .join('')
        : PMS.emptyState('No upcoming featured events listed at this time.');
      PMS.observeFadeElements(eventsGrid);
    } catch (_) {
      eventsGrid.innerHTML = PMS.errorState('Could not load events.');
    } finally {
      eventsGrid.setAttribute('aria-busy', 'false');
    }
  }

  // 3. Load latest technical posts
  const postsGrid = document.getElementById('posts-grid');
  if (postsGrid) {
    postsGrid.setAttribute('aria-busy', 'true');
    postsGrid.innerHTML = PMS.skeletonGrid(3);
    try {
      const posts = await PMS.apiFetch('/api/posts');
      postsGrid.innerHTML = posts.length
        ? posts
            .slice(0, 3)
            .map((p, idx) => PMS.postCard(p, idx))
            .join('')
        : PMS.emptyState('No articles published yet.');
      PMS.observeFadeElements(postsGrid);
    } catch (_) {
      postsGrid.innerHTML = PMS.errorState('Could not load posts.');
    } finally {
      postsGrid.setAttribute('aria-busy', 'false');
    }
  }

  // 4. Newsletter form with client validation
  const newsletterForm = document.getElementById('newsletter-form');
  if (newsletterForm) {
    newsletterForm.addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('newsletter-submit');
      const msgEl = document.getElementById('newsletter-msg');
      const emailInput = document.getElementById('newsletter-email');
      const email = emailInput ? emailInput.value.trim() : '';
      if (!email || !PMS.isEmail(email)) {
        PMS.reportInvalidFields(newsletterForm, [emailInput]);
        PMS.showAlert(msgEl, 'Please enter a valid email address.');
        return;
      }
      PMS.reportInvalidFields(newsletterForm, []);
      PMS.setLoading(btn, true);
      try {
        await PMS.apiPost('/api/newsletter', { email });
        PMS.showAlert(msgEl, 'Subscribed to the PMS Bulletin.', 'success');
        if (emailInput) emailInput.value = '';
      } catch (e) {
        PMS.showAlert(msgEl, e.message || 'Could not subscribe. Please try again.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  }
})();
