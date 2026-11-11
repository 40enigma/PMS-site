/**
 * PIEAS Materials Society (PMS)
 * Admin Console Logic
 * frontend/static/assets/pages/admin.js
 */

let currentTab = 'announcements';
let currentCache = [];
let currentEditingId = null;
let loadSeq = 0;
let eventsMap = {};

const PAGINATED_TABS = new Set(['registrations', 'membership', 'contacts', 'newsletter']);

function isUnauthorized(err) {
  return Boolean(err && err.status === 401);
}

function currentAcademicYear() {
  const now = new Date();
  const y = now.getFullYear();
  return now.getMonth() >= 7 ? `${y}-${y + 1}` : `${y - 1}-${y}`;
}

async function fetchAllRows(tab, api) {
  if (!PAGINATED_TABS.has(tab)) {
    return PMS.apiFetch(api);
  }
  const limit = 200;
  const maxIterations = 500;
  const all = [];
  for (let iter = 0; iter < maxIterations; iter++) {
    const sep = api.includes('?') ? '&' : '?';
    const page = await PMS.apiFetch(`${api}${sep}limit=${limit}&offset=${iter * limit}`);
    if (!Array.isArray(page) || page.length === 0) break;
    all.push(...page);
    if (page.length < limit) break;
  }
  return all;
}

/* ── Schemas for all collections ── */
const SCHEMAS = {
  announcements: {
    title: 'Announcements',
    api: '/api/admin/announcements',
    singular: 'Announcement',
    columns: ['id', 'message', 'priority', 'is_active', 'created_at'],
    canCreate: true,
    fields: [
      { name: 'message', label: 'Message *', type: 'textarea', required: true },
      { name: 'link_text', label: 'Link Text', type: 'text', placeholder: 'e.g. Read Guidelines →' },
      { name: 'link_url', label: 'Link URL', type: 'text', placeholder: 'https:// or /events' },
      { name: 'priority', label: 'Priority', type: 'select', options: ['info', 'urgent', 'warning', 'event', 'success'] },
      { name: 'is_active', label: 'Is Active', type: 'checkbox', default: true }
    ]
  },
  events: {
    title: 'Events',
    api: '/api/admin/events',
    singular: 'Event',
    columns: ['id', 'title', 'slug', 'category', 'date', 'speaker_or_organizer', 'is_past'],
    canCreate: true,
    fields: [
      { name: 'title', label: 'Event Title *', type: 'text', required: true },
      { name: 'slug', label: 'Slug / URL Key (Leave empty to auto-generate)', type: 'text', placeholder: 'e.g. annual-metallurgy-symposium-2026' },
      { name: 'category', label: 'Category', type: 'select', options: ['Seminar', 'Workshop', 'Competition', 'Industrial Visit', 'Symposium', 'Lab Visit', 'Alumni Talk'] },
      { name: 'date', label: 'Event Date *', type: 'date', required: true },
      { name: 'time_range', label: 'Time Range', type: 'text', placeholder: 'e.g. 10:00 AM – 1:00 PM' },
      { name: 'venue', label: 'Venue', type: 'text', placeholder: 'e.g. DMME Block Lecture Hall A' },
      { name: 'speaker_or_organizer', label: 'Speaker / Organizer', type: 'text', placeholder: 'e.g. Dr. Engr. Asim (NESCOM)' },
      { name: 'short_description', label: 'Short Description', type: 'textarea' },
      { name: 'full_summary', label: 'Full Agenda / Summary', type: 'textarea' },
      { name: 'poster_url', label: 'Poster Image URL', type: 'text', upload: true, placeholder: 'https:// or /uploads/...' },
      { name: 'registration_link', label: 'External Registration Link', type: 'text', placeholder: 'Optional external form' },
      { name: 'is_past', label: 'Mark as Past Event', type: 'checkbox', default: false },
      { name: 'is_featured', label: 'Feature on Homepage', type: 'checkbox', default: false }
    ]
  },
  registrations: {
    title: 'Event Registrations',
    api: '/api/admin/registrations',
    singular: 'Registration',
    columns: ['id', 'event_id', 'student_name', 'roll_number', 'batch', 'department', 'email', 'certificate_code', 'attended', 'created_at'],
    canCreate: false,
    canEdit: false
  },
  team: {
    title: 'Team & Council Members',
    api: '/api/admin/team',
    singular: 'Member',
    columns: ['id', 'name', 'role', 'batch', 'tenure_year', 'is_current', 'is_faculty_advisor'],
    canCreate: true,
    fields: [
      { name: 'name', label: 'Full Name *', type: 'text', required: true },
      { name: 'role', label: 'Role / Designation *', type: 'text', required: true, placeholder: 'e.g. President, General Secretary' },
      { name: 'role_responsibility', label: 'Key Responsibilities', type: 'text' },
      { name: 'photo_url', label: 'Photo URL', type: 'text', upload: true, placeholder: '/uploads/... or https://' },
      { name: 'batch', label: 'Batch', type: 'text', placeholder: 'e.g. 2022-2026' },
      { name: 'tenure_year', label: 'Tenure Year *', type: 'text', required: true, placeholder: `e.g. ${currentAcademicYear()}` },
      { name: 'is_current', label: 'Current Council Member', type: 'checkbox', default: true },
      { name: 'is_faculty_advisor', label: 'Faculty Advisor', type: 'checkbox', default: false },
      { name: 'bio', label: 'Short Bio', type: 'textarea' },
      { name: 'linkedin_url', label: 'LinkedIn Profile URL', type: 'text' },
      { name: 'email', label: 'Email Address', type: 'email', placeholder: 'e.g. member@pieas.edu.pk' },
      { name: 'display_order', label: 'Display Order', type: 'number', default: 0 }
    ]
  },
  posts: {
    title: 'Technical Posts & Articles',
    api: '/api/admin/posts',
    singular: 'Post',
    columns: ['id', 'title', 'slug', 'category', 'author_name', 'material_formula', 'published_at'],
    canCreate: true,
    fields: [
      { name: 'title', label: 'Article Title *', type: 'text', required: true },
      { name: 'slug', label: 'Slug / URL Key (Leave empty to auto-generate)', type: 'text', placeholder: 'e.g. titanium-alloys-aerospace' },
      { name: 'category', label: 'Category', type: 'select', options: ['Material of the Month', 'Explainer', 'Technical', 'Alloy Highlight', 'Characterization', 'Additive Manufacturing', 'Student Research'] },
      { name: 'excerpt', label: 'Excerpt Summary *', type: 'textarea', required: true },
      { name: 'body_markdown', label: 'Full Article Markdown *', type: 'textarea', required: true, rows: 10 },
      { name: 'material_formula', label: 'Chemical / Alloy Formula', type: 'text', placeholder: 'e.g. Zr-1.5Sn-0.2Fe or Inconel 718' },
      { name: 'crystal_structure', label: 'Crystal Structure', type: 'text', placeholder: 'e.g. HCP (α), FCC (γ)' },
      { name: 'cover_image_url', label: 'Cover Image URL', type: 'text', upload: true },
      { name: 'author_name', label: 'Author Name', type: 'text', default: 'PIEAS Materials Society' },
      { name: 'author_role', label: 'Author Title / Batch', type: 'text', placeholder: "e.g. Technical Head (DMME '26)" },
      { name: 'author_batch', label: 'Author Batch', type: 'text', placeholder: 'e.g. 2022-2026' },
      { name: 'read_time_minutes', label: 'Read Time (Minutes)', type: 'number', default: 5 },
      { name: 'is_featured', label: 'Feature on Homepage', type: 'checkbox', default: false }
    ]
  },
  resources: {
    title: 'Toolkit Resources & ASTM Standards',
    api: '/api/admin/resources',
    singular: 'Resource',
    columns: ['id', 'title', 'category', 'software_tag', 'url'],
    canCreate: true,
    fields: [
      { name: 'title', label: 'Resource Title *', type: 'text', required: true },
      { name: 'category', label: 'Category *', type: 'select', options: ['Standards', 'Software', 'Safety', 'Databases', 'Books', 'Tools'] },
      { name: 'description', label: 'Description', type: 'textarea' },
      { name: 'url', label: 'URL / Document Link *', type: 'text', upload: true, required: true },
      { name: 'software_tag', label: 'Software / Sub-Tag', type: 'text', placeholder: 'e.g. Thermo-Calc, ASTM E8, ImageJ' },
      { name: 'is_pinned', label: 'Pin to Top of Toolkit', type: 'checkbox', default: false }
    ]
  },
  opportunities: {
    title: 'Opportunities & Scholarships',
    api: '/api/admin/opportunities',
    singular: 'Opportunity',
    columns: ['id', 'title', 'organization', 'opportunity_type', 'deadline', 'is_active'],
    canCreate: true,
    fields: [
      { name: 'title', label: 'Position / Program Title *', type: 'text', required: true },
      { name: 'organization', label: 'Organization / University *', type: 'text', required: true },
      { name: 'opportunity_type', label: 'Type *', type: 'select', options: ['Internship', 'Scholarship', 'Job', 'Research', 'Research Position', 'Conference', 'Competition'] },
      { name: 'location', label: 'Location', type: 'text', placeholder: 'e.g. Islamabad, Remote, Germany' },
      { name: 'deadline', label: 'Application Deadline', type: 'date' },
      { name: 'description', label: 'Summary Details', type: 'textarea' },
      { name: 'posted_by_alumnus', label: 'Posted By Alumnus / Referral', type: 'text', placeholder: 'e.g. Dr. Fatima Zahra (Batch 2018)' },
      { name: 'apply_url', label: 'Application Link', type: 'text' },
      { name: 'is_active', label: 'Is Active', type: 'checkbox', default: true }
    ]
  },
  alumni: {
    title: 'DMME Alumni Directory',
    api: '/api/admin/alumni',
    singular: 'Alumnus',
    columns: ['id', 'name', 'graduating_batch', 'current_role', 'current_organization'],
    canCreate: true,
    fields: [
      { name: 'name', label: 'Full Name *', type: 'text', required: true },
      { name: 'graduating_batch', label: 'Graduation Batch *', type: 'text', required: true, placeholder: 'e.g. 2018-2022' },
      { name: 'current_role', label: 'Current Role / Job Title', type: 'text' },
      { name: 'current_organization', label: 'Organization / Company', type: 'text' },
      { name: 'industry_sector', label: 'Industry Sector', type: 'text', placeholder: 'e.g. Aerospace, Nuclear, Steel, Semiconductor' },
      { name: 'location', label: 'Location', type: 'text', placeholder: 'e.g. Cambridge, MA, USA' },
      { name: 'quote_testimonial', label: 'Quote / Testimonial', type: 'textarea' },
      { name: 'linkedin_url', label: 'LinkedIn Profile URL', type: 'text' },
      { name: 'photo_url', label: 'Photo URL', type: 'text', upload: true }
    ]
  },
  projects: {
    title: 'FYP & Student Projects',
    api: '/api/admin/projects',
    singular: 'Project',
    columns: ['id', 'title', 'project_type', 'batch', 'supervisors'],
    canCreate: true,
    fields: [
      { name: 'title', label: 'Project Title *', type: 'text', required: true },
      { name: 'project_type', label: 'Project Type', type: 'select', options: ['FYP', 'Graduate Research', 'Term Project', 'Competition'] },
      { name: 'batch', label: 'Batch *', type: 'text', required: true, placeholder: 'e.g. 2022-2026' },
      { name: 'students', label: 'Student Investigators', type: 'text', placeholder: 'Names separated by commas' },
      { name: 'supervisors', label: 'Faculty Supervisors', type: 'text', placeholder: 'Dr. ..., Engr. ...' },
      { name: 'abstract', label: 'Abstract / Technical Summary', type: 'textarea' },
      { name: 'tags', label: 'Tags / Keywords', type: 'text', placeholder: 'e.g. Titanium, SLM, Microstructure' },
      { name: 'image_url', label: 'Project Image URL', type: 'text', upload: true },
      { name: 'paper_or_repo_url', label: 'Paper / Code Repository URL', type: 'text' }
    ]
  },
  gallery: {
    title: 'Gallery & Micrographs',
    api: '/api/admin/gallery',
    singular: 'Gallery Item',
    columns: ['id', 'image_url', 'caption', 'event_name', 'year'],
    canCreate: true,
    fields: [
      { name: 'image_url', label: 'Image URL *', type: 'text', upload: true, required: true, placeholder: 'https:// or /uploads/...' },
      { name: 'caption', label: 'Caption / Micrograph Spec', type: 'text', placeholder: 'e.g. Ti-6Al-4V SLM Microstructure (1000x)' },
      { name: 'event_name', label: 'Event / Context', type: 'text' },
      { name: 'event_id', label: 'Linked Event ID (Optional)', type: 'number' },
      { name: 'year', label: 'Year *', type: 'text', required: true, default: String(new Date().getFullYear()) }
    ]
  },
  sponsors: {
    title: 'Industry Partners & Sponsors',
    api: '/api/admin/sponsors',
    singular: 'Sponsor',
    columns: ['id', 'name', 'tier', 'since_year', 'website_url'],
    canCreate: true,
    fields: [
      { name: 'name', label: 'Partner / Company Name *', type: 'text', required: true },
      { name: 'tier', label: 'Partnership Tier', type: 'select', options: ['Platinum', 'Gold', 'Silver', 'Bronze', 'Partner'] },
      { name: 'logo_url', label: 'Logo URL', type: 'text', upload: true, required: false },
      { name: 'website_url', label: 'Website URL', type: 'text' },
      { name: 'since_year', label: 'Partner Since (Year)', type: 'text', placeholder: 'e.g. 2024' },
      { name: 'display_order', label: 'Display Order', type: 'number', default: 0 }
    ]
  },
  faqs: {
    title: 'Frequently Asked Questions',
    api: '/api/admin/faqs',
    singular: 'FAQ',
    columns: ['id', 'question', 'category', 'display_order'],
    canCreate: true,
    fields: [
      { name: 'question', label: 'Question *', type: 'text', required: true },
      { name: 'answer', label: 'Answer *', type: 'textarea', required: true },
      { name: 'category', label: 'Category', type: 'select', options: ['General', 'Membership', 'Events', 'Academic', 'Careers', 'Council', 'Certificates', 'Collaboration'] },
      { name: 'display_order', label: 'Display Order', type: 'number', default: 0 }
    ]
  },
  sections: {
    title: 'Dynamic Site Sections',
    api: '/api/admin/sections',
    singular: 'Site Section',
    columns: ['id', 'section_key', 'title', 'badge', 'updated_at'],
    canCreate: true,
    fields: [
      { name: 'section_key', label: 'Section Key * (Unique Identifier)', type: 'text', required: true, placeholder: 'e.g. home_hero, home_activities' },
      { name: 'title', label: 'Section Headline *', type: 'text', required: true },
      { name: 'subtitle', label: 'Subtitle', type: 'text' },
      { name: 'badge', label: 'Badge Text', type: 'text' },
      { name: 'content', label: 'Main Content Body *', type: 'textarea', required: true },
      { name: 'extra_json', label: 'Extra JSON (Cards / Links config)', type: 'textarea', rows: 4, placeholder: 'Valid JSON array or object' }
    ]
  },
  membership: {
    title: 'Membership Applications',
    api: '/api/admin/membership',
    singular: 'Application',
    columns: ['id', 'full_name', 'roll_number', 'batch', 'department', 'email', 'status', 'created_at'],
    canCreate: false,
    canEdit: false
  },
  contacts: {
    title: 'Contact Messages',
    api: '/api/admin/contacts',
    singular: 'Message',
    columns: ['id', 'name', 'email', 'subject', 'message', 'created_at'],
    canCreate: false,
    canEdit: false
  },
  newsletter: {
    title: 'Newsletter Subscribers',
    api: '/api/admin/newsletter',
    singular: 'Subscriber',
    columns: ['id', 'email', 'subscribed_at'],
    canCreate: false,
    canEdit: false
  }
};

/* ── Authentication State ── */
async function checkAuth() {
  try {
    const user = await PMS.apiFetch('/api/auth/me');
    showDashboard(user);
  } catch (_) {
    showLogin();
  }
}

function showLogin() {
  const loginSec = document.getElementById('login-section');
  if (loginSec) loginSec.style.display = '';
  const dash = document.getElementById('admin-dashboard');
  if (dash) dash.style.display = 'none';
  const userItem = document.getElementById('nav-user-item');
  if (userItem) userItem.style.display = 'none';
  const logoutItem = document.getElementById('nav-logout-item');
  if (logoutItem) logoutItem.style.display = 'none';
  const logoutBtn = document.getElementById('logout-btn');
  if (logoutBtn) logoutBtn.style.display = 'none';
  const navUser = document.getElementById('nav-user');
  if (navUser) navUser.textContent = '';
}

function showDashboard(user) {
  const loginSec = document.getElementById('login-section');
  if (loginSec) loginSec.style.display = 'none';
  const dash = document.getElementById('admin-dashboard');
  if (dash) dash.style.display = '';
  const userItem = document.getElementById('nav-user-item');
  if (userItem) userItem.style.display = '';
  const logoutItem = document.getElementById('nav-logout-item');
  if (logoutItem) logoutItem.style.display = '';
  const logoutBtn = document.getElementById('logout-btn');
  if (logoutBtn) logoutBtn.style.display = '';
  if (user && user.name) {
    const navUser = document.getElementById('nav-user');
    if (navUser) navUser.textContent = user.name;
  }
  loadSection(currentTab);
}

const loginForm = document.getElementById('login-form');
if (loginForm) {
  loginForm.addEventListener('submit', async ev => {
    ev.preventDefault();
    const btn = document.getElementById('login-submit');
    const msg = document.getElementById('login-msg');
    const emailEl = document.getElementById('login-email');
    const pwEl = document.getElementById('login-password');
    const missing = [emailEl, pwEl].filter(el => !el || !el.value.trim());
    if (missing.length) {
      PMS.reportInvalidFields(loginForm, missing);
      PMS.showAlert(msg, 'Please enter your administrator email and password.');
      return;
    }
    PMS.reportInvalidFields(loginForm, []);
    PMS.setLoading(btn, true);
    try {
      const res = await PMS.apiPost('/api/auth/login', {
        email: emailEl.value,
        password: pwEl.value,
      });
      showDashboard(res.admin || res);
    } catch (e) {
      PMS.showAlert(msg, e.message || 'Invalid administrator credentials.');
    } finally {
      PMS.setLoading(btn, false);
    }
  });
}

const logoutBtn = document.getElementById('logout-btn');
if (logoutBtn) {
  logoutBtn.addEventListener('click', async () => {
    try { await PMS.apiPost('/api/auth/logout', {}); } catch (_) {}
    showLogin();
    const navLinks = document.querySelector('.nav-links');
    const hamburger = document.querySelector('.hamburger');
    if (navLinks) navLinks.classList.remove('open');
    if (hamburger) {
      hamburger.classList.remove('active');
      hamburger.setAttribute('aria-expanded', 'false');
    }
  });
}

/* ── Tab Switcher & ARIA Tablist Navigation ── */
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
  currentTab = btn.dataset.section;
  loadSection(currentTab);
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

/* ── Section Loader ── */
async function loadSection(tab) {
  const req = ++loadSeq;
  const content = document.getElementById('admin-content');
  const toolbar = document.getElementById('toolbar-actions');
  if (!content) return;
  let savedRegFilter = '';
  if (tab === 'registrations') {
    const existingFilter = document.getElementById('reg-event-filter');
    if (existingFilter) savedRegFilter = existingFilter.value;
  }
  content.setAttribute('aria-busy', 'true');
  content.innerHTML = '<div class="spinner" role="status" aria-live="polite" aria-label="Loading section"></div>';
  if (toolbar) toolbar.innerHTML = '';

  try {
    if (tab === 'stats') {
      await renderStatsEditor(content);
      return;
    }
    if (tab === 'upload') {
      renderUploadTool(content);
      return;
    }

    const schema = SCHEMAS[tab];
    if (!schema) return;

    if (schema.canCreate && toolbar) {
      toolbar.innerHTML = `<button class="btn btn-primary btn-sm" id="btn-create-record">＋ New ${PMS.escHtml(schema.singular)}</button>`;
      const createRecordBtn = document.getElementById('btn-create-record');
      if (createRecordBtn) createRecordBtn.onclick = () => openCreateModal(tab, createRecordBtn);
    }

    if (tab === 'registrations') {
      try {
        const evts = await PMS.apiFetch('/api/admin/events');
        if (req !== loadSeq) return;
        eventsMap = {};
        evts.forEach(e => { eventsMap[e.id] = e.title; });
      } catch (_) {}
    }

    const data = PAGINATED_TABS.has(tab)
      ? await fetchAllRows(tab, schema.api)
      : await PMS.apiFetch(schema.api);
    if (req !== loadSeq) return;
    currentCache = data;

    if (!data.length) {
      content.innerHTML = `<p style="color:var(--text-muted);padding:1.5rem 0">No records found in ${PMS.escHtml(schema.title)}.</p>`;
      return;
    }

    if (tab === 'registrations' && toolbar) {
      const uniqueEvents = Object.keys(eventsMap).map(id => ({ id, title: eventsMap[id] }));
      const opts = uniqueEvents.map(e => `<option value="${e.id}">${PMS.escHtml(e.title)}</option>`).join('');
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

    function renderRow(item) {
      const tds = schema.columns.map(c => {
        let val = item[c];
        if (c === 'event_id' && tab === 'registrations') {
          const t = eventsMap[val] || `Event #${val}`;
          return `<td><span title="${PMS.escHtml(t)}" style="font-weight:600;color:var(--text)">${PMS.escHtml(t)}</span></td>`;
        }
        if (c === 'attended' && tab === 'registrations') {
          return `<td><span class="status-badge ${val ? 'status-approved' : 'status-rejected'}">${val ? 'Attended' : 'Absent'}</span></td>`;
        }
        if (c === 'status' && tab === 'membership') {
          const s = String(val).toLowerCase();
          const cls = s === 'approved' ? 'status-approved' : (s === 'rejected' ? 'status-rejected' : 'status-pending');
          return `<td><span class="status-badge ${cls}">${PMS.escHtml(String(val))}</span></td>`;
        }
        if (typeof val === 'boolean') return `<td>${val ? 'Yes' : '—'}</td>`;
        if (val === null || val === undefined) return `<td>—</td>`;
        const str = String(val);
        const display = str.length > 50 ? str.slice(0, 48) + '…' : str;
        return `<td>${PMS.escHtml(display)}</td>`;
      }).join('');

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
    }

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
          const headers = ['ID', 'Event ID', 'Event Title', 'Student Name', 'Roll Number', 'Batch', 'Department', 'Email', 'Phone', 'Certificate Code', 'Attended', 'Created At'];
          const rows = displayed.map(r => [
            r.id,
            r.event_id,
            eventsMap[r.event_id] || `Event #${r.event_id}`,
            r.student_name,
            r.roll_number,
            r.batch,
            r.department,
            r.email,
            r.phone || '',
            r.certificate_code || '',
            r.attended ? 'Yes' : 'No',
            r.created_at || ''
          ]);

          // Neutralize formula injection with single quote prefix if cell starts with = + - @ \t \r
          const csv = [headers, ...rows]
            .map(row => row.map(cell => {
              let str = String(cell ?? '');
              if (/^[=+\-@\t\r]/.test(str)) {
                str = "'" + str;
              }
              return `"${str.replace(/"/g, '""')}"`;
            }).join(','))
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
    if (req !== loadSeq) return;
    if (isUnauthorized(err)) {
      showLogin();
    } else {
      content.innerHTML = `<div class="alert alert-error" role="alert">Error loading records: ${PMS.escHtml(err.message)}</div>`;
    }
  } finally {
    if (req === loadSeq) content.setAttribute('aria-busy', 'false');
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
    const schema = SCHEMAS[currentTab];

    if (action === 'delete') {
      const confirmMsg = currentTab === 'events'
        ? 'Are you sure you want to permanently delete this event? WARNING: All registrations and issued certificates for this event will also be deleted!'
        : 'Are you sure you want to permanently delete this record?';
      if (!confirm(confirmMsg)) return;
      try {
        await PMS.apiDelete(`${schema.api}/${id}`);
        loadSection(currentTab);
      } catch (err) {
        if (isUnauthorized(err)) {
          showLogin();
          return;
        }
        alert('Delete failed: ' + err.message);
      }
    } else if (action === 'toggle-attended') {
      const isAttended = btn.dataset.attended === 'true';
      try {
        await PMS.apiPut(`/api/admin/registrations/${id}`, { attended: !isAttended });
        loadSection(currentTab);
      } catch (err) {
        if (isUnauthorized(err)) {
          showLogin();
          return;
        }
        alert('Attendance update failed: ' + err.message);
      }
    } else if (action === 'edit') {
      openEditModal(currentTab, id, btn);
    } else if (action === 'view') {
      openViewModal(currentTab, id, btn);
    } else if (action === 'status') {
      const status = btn.dataset.status;
      try {
        await PMS.apiPut(`/api/admin/membership/${id}`, { status });
        loadSection(currentTab);
      } catch (err) {
        if (isUnauthorized(err)) {
          showLogin();
          return;
        }
        alert('Status update failed: ' + err.message);
      }
    }
  });
}

/* ── Full Details View Modal ── */
function openViewModal(tab, id, triggerEl) {
  const item = currentCache.find(x => String(x.id) === String(id));
  if (!item) return;
  const schema = SCHEMAS[tab];
  const modal = document.getElementById('view-modal');
  if (!modal) return;
  const title = document.getElementById('view-modal-title');
  if (title) title.textContent = `${schema ? schema.singular : 'Record'} #${id} Full Details`;
  const container = document.getElementById('view-modal-content');
  if (container) {
    container.innerHTML = Object.entries(item).map(([k, v]) => `
      <div style="border-bottom:1px solid var(--border);padding:.6rem 0">
        <div style="font-size:.75rem;text-transform:uppercase;letter-spacing:.5px;color:var(--text-muted);font-weight:600;margin-bottom:.25rem">${PMS.escHtml(k.replace(/_/g, ' '))}</div>
        <div style="font-size:.9rem;color:var(--text);white-space:pre-wrap;word-break:break-word">${v !== null && v !== undefined && v !== '' ? PMS.escHtml(String(v)) : '<span style="color:var(--text-muted)">—</span>'}</div>
      </div>
    `).join('');
  }
  PMS.showModalWithFocusReturn(modal, triggerEl);
}

const viewModal = document.getElementById('view-modal');
if (viewModal) {
  PMS.enableBackdropClose(viewModal);
  const closeBtn = document.getElementById('view-modal-close-btn');
  const closeFooterBtn = document.getElementById('view-modal-close-footer-btn');
  if (closeBtn) closeBtn.onclick = () => viewModal.close();
  if (closeFooterBtn) closeFooterBtn.onclick = () => viewModal.close();
}

/* ── Site Stats Editor ── */
async function renderStatsEditor(container) {
  try {
    const stats = await PMS.apiFetch('/api/admin/stats');
    container.innerHTML = `
      <h2 style="font-size:1.25rem;margin-bottom:.5rem">Edit Key Site Statistics &amp; Society Details</h2>
      <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:1.5rem">These metrics power the live counter strip and contact info on the homepage.</p>
      <form id="stats-form">
        <div class="grid grid-2" style="gap:1rem">
          <div class="form-group">
            <label class="form-label" for="stat-members">Active Members Count</label>
            <input id="stat-members" type="number" min="0" class="form-control" value="${stats.active_members || 0}">
          </div>
          <div class="form-group">
            <label class="form-label" for="stat-events">Total Events Held</label>
            <input id="stat-events" type="number" min="0" class="form-control" value="${stats.events_held || 0}">
          </div>
          <div class="form-group">
            <label class="form-label" for="stat-years">Years Active</label>
            <input id="stat-years" type="number" min="0" class="form-control" value="${stats.years_active || 0}">
          </div>
          <div class="form-group">
            <label class="form-label" for="stat-partners">Industrial Partners Count</label>
            <input id="stat-partners" type="number" min="0" class="form-control" value="${stats.industrial_partners || 0}">
          </div>
          <div class="form-group" style="grid-column:1/-1">
            <label class="form-label" for="stat-pdf">Constitution PDF URL</label>
            <input id="stat-pdf" type="text" class="form-control" value="${PMS.escHtml(stats.constitution_pdf_url || '')}">
          </div>
          <div class="form-group">
            <label class="form-label" for="stat-loc">Office Location</label>
            <input id="stat-loc" type="text" class="form-control" value="${PMS.escHtml(stats.office_location || '')}">
          </div>
          <div class="form-group">
            <label class="form-label" for="stat-email">Official Contact Email</label>
            <input id="stat-email" type="email" class="form-control" value="${PMS.escHtml(stats.contact_email || '')}">
          </div>
        </div>
        <div id="stats-msg" role="status" aria-live="polite" style="margin-top:1rem"></div>
        <button type="submit" class="btn btn-primary" style="margin-top:1rem" id="stats-save-btn">Save Statistics</button>
      </form>
    `;

    document.getElementById('stats-form').addEventListener('submit', async ev => {
      ev.preventDefault();
      const btn = document.getElementById('stats-save-btn');
      const msg = document.getElementById('stats-msg');

      const active_members = parseInt(document.getElementById('stat-members').value, 10) || 0;
      const events_held = parseInt(document.getElementById('stat-events').value, 10) || 0;
      const years_active = parseInt(document.getElementById('stat-years').value, 10) || 0;
      const industrial_partners = parseInt(document.getElementById('stat-partners').value, 10) || 0;

      const statsForm = document.getElementById('stats-form');
      const statInputs = ['stat-members', 'stat-events', 'stat-years', 'stat-partners']
        .map(id => document.getElementById(id));
      const negativeInputs = statInputs.filter(el => (parseInt(el.value, 10) || 0) < 0);
      if (negativeInputs.length) {
        PMS.reportInvalidFields(statsForm, negativeInputs);
        PMS.showAlert(msg, 'Statistic counts cannot be negative numbers.');
        return;
      }
      PMS.reportInvalidFields(statsForm, []);

      PMS.setLoading(btn, true);
      try {
        await PMS.apiPut('/api/admin/stats', {
          active_members,
          events_held,
          years_active,
          industrial_partners,
          constitution_pdf_url: document.getElementById('stat-pdf').value.trim(),
          office_location: document.getElementById('stat-loc').value.trim(),
          contact_email: document.getElementById('stat-email').value.trim(),
        });
        PMS.showAlert(msg, 'Site statistics updated.', 'success');
      } catch (e) {
        if (isUnauthorized(e)) {
          showLogin();
          return;
        }
        PMS.showAlert(msg, e.message || 'Failed to update stats.');
      } finally {
        PMS.setLoading(btn, false);
      }
    });
  } catch (err) {
    if (isUnauthorized(err)) {
      showLogin();
    } else {
      container.innerHTML = `<div class="alert alert-error" role="alert">Error loading statistics: ${PMS.escHtml(err.message)}</div>`;
    }
  }
}

/* ── Upload Tool ── */
function renderUploadTool(container) {
  container.innerHTML = `
    <h2 style="font-size:1.25rem;margin-bottom:.5rem">Upload Static Media Asset</h2>
    <p style="color:var(--text-muted);font-size:.875rem;margin-bottom:1.5rem">
      Allowed formats: PNG, JPEG, WebP, PDF, text. Maximum 20 MB. Files are stored in <code>/uploads/</code> with strict non-executable permissions.
    </p>
    <form id="upload-form">
      <div class="form-group">
        <label class="form-label" for="upload-file">Select File</label>
        <input id="upload-file" type="file" class="form-control" accept="image/*,application/pdf,text/plain" required>
      </div>
      <div id="upload-msg" role="status" aria-live="polite" style="margin-top:1rem"></div>
      <button type="submit" class="btn btn-primary" style="margin-top:1rem" id="upload-submit">Upload Asset</button>
    </form>
  `;

  document.getElementById('upload-form').addEventListener('submit', async ev => {
    ev.preventDefault();
    const btn = document.getElementById('upload-submit');
    const msg = document.getElementById('upload-msg');
    const file = document.getElementById('upload-file').files[0];
    if (!file) return;
    PMS.setLoading(btn, true);
    try {
      const fd = new FormData();
      fd.append('file', file);
      const res = await fetch('/api/admin/upload', {
        method: 'POST',
        credentials: 'include',
        headers: { 'X-Requested-With': 'PMS-Client' },
        body: fd
      });
      if (!res.ok) {
        if (res.status === 401) {
          showLogin();
          return;
        }
        let errDetail = '';
        try {
          const b = await res.json();
          errDetail = b.error;
        } catch (_) {}
        throw new Error(errDetail || `Upload failed (HTTP ${res.status})`);
      }
      const data = await res.json();
      PMS.showAlert(msg, `Asset uploaded: <a href="${PMS.safeUrl(data.url)}" target="_blank" rel="noopener noreferrer" style="color:var(--text);font-weight:600">${PMS.escHtml(data.url)}</a>`, 'success', true);
    } catch (e) {
      PMS.showAlert(msg, e.message);
    } finally {
      PMS.setLoading(btn, false);
    }
  });
}

/* ── Create & Edit Modal Operations ── */
const modal = document.getElementById('crud-modal');
if (modal) PMS.enableBackdropClose(modal);
const modalCancelBtn = document.getElementById('modal-cancel-btn');
if (modalCancelBtn) modalCancelBtn.onclick = () => modal.close();
const modalCancelFooterBtn = document.getElementById('modal-cancel-footer-btn');
if (modalCancelFooterBtn) modalCancelFooterBtn.onclick = () => modal.close();

function openCreateModal(tab, triggerEl) {
  const schema = SCHEMAS[tab];
  currentEditingId = null;
  const title = document.getElementById('modal-title');
  if (title) title.textContent = `New ${schema.singular}`;
  const msg = document.getElementById('crud-msg');
  if (msg) msg.innerHTML = '';
  renderFields(schema.fields, {});
  PMS.showModalWithFocusReturn(modal, triggerEl);
}

function openEditModal(tab, id, triggerEl) {
  const schema = SCHEMAS[tab];
  const item = currentCache.find(x => String(x.id) === String(id));
  if (!item) return;
  currentEditingId = id;
  const title = document.getElementById('modal-title');
  if (title) title.textContent = `Edit ${schema.singular} (#${id})`;
  const msg = document.getElementById('crud-msg');
  if (msg) msg.innerHTML = '';
  renderFields(schema.fields, item);
  PMS.showModalWithFocusReturn(modal, triggerEl);
}

function renderFields(fields, currentData) {
  const container = document.getElementById('crud-fields');
  if (!container) return;
  container.innerHTML = fields.map(f => {
    const rawVal = currentData[f.name] !== undefined ? currentData[f.name] : (f.default !== undefined ? f.default : '');
    const val = f.type === 'date' && /^\d{4}-\d{2}-\d{2}/.test(String(rawVal || '')) ? String(rawVal).slice(0, 10) : rawVal;
    const isSectionKeyEdit = currentEditingId && f.name === 'section_key';

    if (f.type === 'textarea') {
      return `
        <div class="form-group">
          <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
          <textarea id="f-${f.name}" class="form-control" rows="${f.rows || 3}" ${f.required ? 'required' : ''}>${PMS.escHtml(val)}</textarea>
        </div>`;
    }
    if (f.type === 'checkbox') {
      return `
        <div class="form-group" style="display:flex;align-items:center;gap:.75rem">
          <input type="checkbox" id="f-${f.name}" ${val ? 'checked' : ''} style="width:18px;height:18px">
          <label class="form-label" for="f-${f.name}" style="margin:0;cursor:pointer">${PMS.escHtml(f.label)}</label>
        </div>`;
    }
    if (f.type === 'select') {
      let optsList = [...f.options];
      // F-05: If current value is not in options, prepend it so editing never silently changes it
      if (val !== '' && val !== null && val !== undefined && !optsList.map(String).includes(String(val))) {
        optsList.unshift(val);
      }
      const opts = optsList.map(o => `<option value="${PMS.escHtml(o)}" ${String(val) === String(o) ? 'selected' : ''}>${PMS.escHtml(o)}</option>`).join('');
      return `
        <div class="form-group">
          <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
          <select id="f-${f.name}" class="form-control">${opts}</select>
        </div>`;
    }
    if (f.upload) {
      return `
        <div class="form-group">
          <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
          <div style="display:flex;gap:.5rem;align-items:center">
            <input type="${f.type || 'text'}" id="f-${f.name}" class="form-control" value="${PMS.escHtml(val)}" placeholder="${PMS.escHtml(f.placeholder || '')}" ${f.required ? 'required' : ''} style="flex:1">
            <label class="btn btn-outline btn-sm inline-upload-label" style="margin:0;white-space:nowrap;cursor:pointer;padding:.4rem .7rem">
              <span class="upload-status-text">Upload</span>
              <input type="file" data-target-input="f-${f.name}" class="inline-file-input">
            </label>
          </div>
        </div>`;
    }
    return `
      <div class="form-group">
        <label class="form-label" for="f-${f.name}">${PMS.escHtml(f.label)}</label>
        <input type="${f.type || 'text'}" id="f-${f.name}" class="form-control" value="${PMS.escHtml(val)}" placeholder="${PMS.escHtml(f.placeholder || '')}" ${f.required ? 'required' : ''} ${isSectionKeyEdit ? 'readonly style="opacity:0.7;cursor:not-allowed"' : ''}>
      </div>`;
  }).join('');
}

// Inline file uploader listener for crud modal
const crudFields = document.getElementById('crud-fields');
if (crudFields) {
  crudFields.addEventListener('change', async ev => {
    const fileInput = ev.target.closest('.inline-file-input');
    if (!fileInput || !fileInput.files || !fileInput.files[0]) return;
    const file = fileInput.files[0];
    const targetId = fileInput.dataset.targetInput;
    const labelEl = fileInput.closest('.inline-upload-label');
    const statusSpan = labelEl ? labelEl.querySelector('.upload-status-text') : null;
    const expectedEditingId = currentEditingId;
    const expectedTab = currentTab;

    if (statusSpan) statusSpan.textContent = 'Uploading…';
    try {
      const fd = new FormData();
      fd.append('file', file);
      const res = await fetch('/api/admin/upload', {
        method: 'POST',
        credentials: 'include',
        headers: { 'X-Requested-With': 'PMS-Client' },
        body: fd
      });
      if (!res.ok) {
        if (res.status === 401) {
          showLogin();
          return;
        }
        let errDetail = '';
        try {
          const b = await res.json();
          errDetail = b.error;
        } catch (_) {}
        throw new Error(errDetail || `Upload failed (HTTP ${res.status})`);
      }
      const data = await res.json();
      if (modal.open && currentEditingId === expectedEditingId && currentTab === expectedTab) {
        const targetEl = document.getElementById(targetId);
        if (targetEl) targetEl.value = data.url;
        if (statusSpan) statusSpan.textContent = 'Uploaded';
      }
    } catch (err) {
      if (modal.open && currentEditingId === expectedEditingId && currentTab === expectedTab) {
        alert('Upload failed: ' + err.message);
        if (statusSpan) statusSpan.textContent = 'Upload';
      }
    } finally {
      fileInput.value = '';
    }
  });
}

const crudForm = document.getElementById('crud-form');
if (crudForm) {
  crudForm.addEventListener('submit', async ev => {
    ev.preventDefault();
    const btn = document.getElementById('modal-save-btn');
    const msg = document.getElementById('crud-msg');
    const schema = SCHEMAS[currentTab];
    if (!schema) return;

    const payload = {};
    const missingFields = [];
    let firstMissingLabel = '';
    for (const f of schema.fields) {
      const el = document.getElementById(`f-${f.name}`);
      if (!el) continue;
      if (f.type === 'checkbox') {
        payload[f.name] = el.checked;
      } else if (f.type === 'number') {
        const parsed = parseInt(el.value, 10);
        payload[f.name] = el.value === '' || isNaN(parsed) ? null : parsed;
      } else {
        const trimmed = el.value.trim();
        if (
          trimmed === '' &&
          currentEditingId &&
          currentTab === 'sections' &&
          ['subtitle', 'badge', 'extra_json'].includes(f.name)
        ) {
          payload[f.name] = '';
        } else {
          payload[f.name] = trimmed === '' ? null : trimmed;
        }
      }
      if (f.required && (payload[f.name] === null || payload[f.name] === '')) {
        missingFields.push(el);
        if (!firstMissingLabel) firstMissingLabel = f.label.replace('*', '').trim();
      }
    }
    if (missingFields.length) {
      PMS.reportInvalidFields(crudForm, missingFields);
      PMS.showAlert(msg, `Please fill out required field: ${firstMissingLabel}`);
      return;
    }

    // Client-Side extra_json Validation
    if (payload.extra_json) {
      try {
        JSON.parse(payload.extra_json);
      } catch (jsonErr) {
        PMS.reportInvalidFields(crudForm, [document.getElementById('f-extra_json')]);
        PMS.showAlert(msg, `Invalid JSON in Extra JSON field: ${jsonErr.message}`);
        return;
      }
    }
    PMS.reportInvalidFields(crudForm, []);

    PMS.setLoading(btn, true);
    try {
      if (currentEditingId) {
        await PMS.apiPut(`${schema.api}/${currentEditingId}`, payload);
      } else {
        await PMS.apiPost(schema.api, payload);
      }
      modal.close();
      loadSection(currentTab);
    } catch (err) {
      if (isUnauthorized(err)) {
        showLogin();
        return;
      }
      PMS.showAlert(msg, err.message || 'Operation failed.');
    } finally {
      PMS.setLoading(btn, false);
    }
  });
}

/* Initialize */
checkAuth();
