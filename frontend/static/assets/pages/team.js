/**
 * PIEAS Materials Society (PMS)
 * Team & Council Page Logic
 * frontend/static/assets/pages/team.js
 */

(() => {
  'use strict';

  let allMembers = [];

  async function loadTeamData() {
    const grid = document.getElementById('team-grid');
    if (!grid) return;
    grid.setAttribute('aria-busy', 'true');
    grid.innerHTML = PMS.skeletonGrid(4);
    try {
      allMembers = await PMS.apiFetch('/api/team');
      renderFilteredTeam('current');
    } catch (err) {
      PMS.reportError('team', err);
      grid.innerHTML = PMS.errorState('Could not load team directory.');
    } finally {
      grid.setAttribute('aria-busy', 'false');
    }
  }

  function renderFilteredTeam(mode) {
    const grid = document.getElementById('team-grid');
    if (!grid) return;
    if (mode === 'past') {
      const pastMembers = allMembers.filter(m => !m.is_current && !m.is_faculty_advisor);
      if (!pastMembers.length) {
        grid.innerHTML = PMS.emptyState('No members listed in this category.');
        PMS.observeFadeElements(grid);
        return;
      }
      const groups = new Map();
      pastMembers.forEach(m => {
        const yr = (m.tenure_year && String(m.tenure_year).trim()) || 'Other';
        if (!groups.has(yr)) groups.set(yr, []);
        groups.get(yr).push(m);
      });
      const yearValue = y => {
        const match = /^(\d{4})/.exec(y);
        return match ? parseInt(match[1], 10) : -1;
      };
      const sortedYears = Array.from(groups.keys()).sort(
        (a, b) => yearValue(b) - yearValue(a) || b.localeCompare(a)
      );
      grid.innerHTML = sortedYears
        .map(yr => {
          const heading = `<h2 class="council-year">${PMS.escHtml(yr)}</h2>`;
          const cards = groups
            .get(yr)
            .map((m, idx) => PMS.teamCard(m, idx))
            .join('');
          return heading + cards;
        })
        .join('');
      PMS.observeFadeElements(grid);
      return;
    }
    let filtered = [];
    if (mode === 'current') {
      filtered = allMembers.filter(m => m.is_current && !m.is_faculty_advisor);
    } else if (mode === 'faculty') {
      // Show all faculty advisors: current first, former second
      const faculty = allMembers.filter(m => m.is_faculty_advisor);
      filtered = faculty.slice().sort((a, b) => (b.is_current ? 1 : 0) - (a.is_current ? 1 : 0));
    }
    grid.innerHTML = filtered.length
      ? filtered.map((m, idx) => PMS.teamCard(m, idx)).join('')
      : PMS.emptyState('No members listed in this category.');
    PMS.observeFadeElements(grid);
  }

  // Button toggles
  document.querySelectorAll('.team-toggle').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.team-toggle').forEach(b => {
        const isActive = b === btn;
        b.classList.toggle('active', isActive);
        b.setAttribute('aria-pressed', String(isActive));
      });
      renderFilteredTeam(btn.dataset.filter);
    });
  });

  // Load team guidance portfolios from site_sections
  async function loadTeamGuidance() {
    const grid = document.getElementById('guidance-grid');
    if (!grid) return;
    grid.setAttribute('aria-busy', 'true');
    try {
      const sections = await PMS.apiFetchCached('/api/sections');
      const sec = sections.find(s => s.section_key === 'team_guidance');
      if (sec) {
        if (sec.badge) {
          const badgeEl = document.getElementById('guidance-badge');
          if (badgeEl) badgeEl.textContent = sec.badge;
        }
        if (sec.title) {
          const titleEl = document.getElementById('guidance-title');
          if (titleEl) titleEl.textContent = sec.title;
        }
        if (sec.subtitle) {
          const subEl = document.getElementById('guidance-subtitle');
          if (subEl) {
            subEl.textContent = sec.subtitle;
            subEl.style.display = 'block';
          }
        }
        if (sec.content) {
          const descEl = document.getElementById('guidance-desc');
          if (descEl) descEl.textContent = sec.content;
        }

        if (sec.extra_json) {
          try {
            const portfolios = JSON.parse(sec.extra_json);
            grid.innerHTML = portfolios
              .map(
                p => `
              <article class="card fade-up">
                <span class="badge" style="margin-bottom:var(--sp-2)">${PMS.escHtml(p.scope)}</span>
                <h3 class="card__title card__title--xs">${PMS.escHtml(p.title)}</h3>
                <p class="card__text card__text--sm">${PMS.escHtml(p.desc)}</p>
              </article>
            `
              )
              .join('');
            PMS.observeFadeElements(grid);
            return;
          } catch (err) {
            PMS.reportError('team-guidance-json', err);
          }
        }
      }
      grid.innerHTML = PMS.emptyState('Portfolio details will be announced soon.');
    } catch (err) {
      PMS.reportError('team-guidance', err);
      grid.innerHTML = PMS.errorState('Could not load guidance portfolios.');
    } finally {
      grid.setAttribute('aria-busy', 'false');
    }
  }

  loadTeamData();
  loadTeamGuidance();
})();
