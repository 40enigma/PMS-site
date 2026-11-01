/**
 * PIEAS Materials Society (PMS)
 * Team & Council Page Logic
 * frontend/static/assets/pages/team.js
 */

let allMembers = [];

async function loadTeamData() {
  const grid = document.getElementById('team-grid');
  if (!grid) return;
  grid.innerHTML = PMS.skeletonGrid(4);
  try {
    allMembers = await PMS.apiFetch('/api/team');
    renderFilteredTeam('current');
  } catch (e) {
    grid.innerHTML = '<p style="color:var(--danger);grid-column:1/-1">Failed to load team directory.</p>';
  }
}

function renderFilteredTeam(mode) {
  const grid = document.getElementById('team-grid');
  if (!grid) return;
  let filtered = [];
  if (mode === 'current') {
    filtered = allMembers.filter(m => m.is_current && !m.is_faculty_advisor);
  } else if (mode === 'faculty') {
    // Show all faculty advisors: current first, former second
    const faculty = allMembers.filter(m => m.is_faculty_advisor);
    filtered = faculty.slice().sort((a, b) => (b.is_current ? 1 : 0) - (a.is_current ? 1 : 0));
  } else if (mode === 'past') {
    filtered = allMembers.filter(m => !m.is_current && !m.is_faculty_advisor);
  }
  grid.innerHTML = filtered.length
    ? filtered.map((m, idx) => PMS.teamCard(m, idx)).join('')
    : '<p style="color:var(--text-muted);grid-column:1/-1;text-align:center">No members listed in this category.</p>';
  PMS.observeFadeElements(grid);
}

// Button toggles
document.querySelectorAll('.team-toggle').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.team-toggle').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    renderFilteredTeam(btn.dataset.filter);
  });
});

// Load team guidance portfolios from site_sections
async function loadTeamGuidance() {
  const grid = document.getElementById('guidance-grid');
  if (!grid) return;
  try {
    const sections = await PMS.apiFetch('/api/sections');
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
          grid.innerHTML = portfolios.map(p => `
            <article class="card fade-up">
              <span class="badge" style="margin-bottom:.5rem">${PMS.escHtml(p.scope)}</span>
              <h3 style="font-size:1.15rem;margin-bottom:.5rem;color:var(--text)">${PMS.escHtml(p.title)}</h3>
              <p style="color:var(--text-muted);font-size:.875rem;line-height:1.6">${PMS.escHtml(p.desc)}</p>
            </article>
          `).join('');
          PMS.observeFadeElements(grid);
          return;
        } catch (_) {}
      }
    }
    grid.innerHTML = '<p style="color:var(--text-muted);grid-column:1/-1">Portfolios configured in administration.</p>';
  } catch (_) {
    grid.innerHTML = '';
  }
}

loadTeamData();
loadTeamGuidance();

