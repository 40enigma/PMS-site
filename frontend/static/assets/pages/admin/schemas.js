/**
 * PIEAS Materials Society (PMS)
 * Admin Console Schemas, State, and Pagination Helpers
 * frontend/static/assets/pages/admin/schemas.js
 */
(() => {
  'use strict';

  /** Rows requested per page when walking a list endpoint. */
  const PAGE_SIZE = 200;
  /** Hard stop so a broken endpoint cannot loop forever. */
  const MAX_PAGES = 100;

  /**
   * Detect an HTTP 401 Unauthorized error thrown by `PMS.apiFetch`.
   * @param {unknown} err
   * @returns {boolean}
   */
  function isUnauthorized(err) {
    return Boolean(err && typeof err === 'object' && 'status' in err && err.status === 401);
  }

  /**
   * Compute the active PIEAS academic year (August to July rollover).
   * @returns {string}
   */
  function currentAcademicYear() {
    const now = new Date();
    const y = now.getFullYear();
    return now.getMonth() >= 7 ? `${y}-${y + 1}` : `${y - 1}-${y}`;
  }

  /**
   * Fetch all paginated rows for the four paginated admin tabs.
   * Guarded against more than 500 pages to prevent accidental infinite loops.
   * @param {string} tab
   * @param {string} api
   * @returns {Promise<any[]>}
   */
  async function fetchAllRows(tab, api) {
    if (!PAGINATED_TABS.has(tab)) {
      return PMS.apiFetch(api);
    }
    const all = [];
    for (let page = 0; page < MAX_PAGES; page++) {
      const sep = api.includes('?') ? '&' : '?';
      const rows = await PMS.apiFetch(`${api}${sep}limit=${PAGE_SIZE}&offset=${page * PAGE_SIZE}`);
      if (!Array.isArray(rows) || rows.length === 0) break;
      // Guard against endpoints that ignore offset and replay the first page.
      if (all.length && rows[0] && all[0] && rows[0].id === all[0].id) break;
      all.push(...rows);
      if (rows.length < PAGE_SIZE) break;
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
        {
          name: 'link_text',
          label: 'Link Text',
          type: 'text',
          placeholder: 'e.g. Read Guidelines →',
        },
        { name: 'link_url', label: 'Link URL', type: 'text', placeholder: 'https:// or /events' },
        {
          name: 'priority',
          label: 'Priority',
          type: 'select',
          options: ['info', 'urgent', 'warning', 'event', 'success'],
        },
        { name: 'is_active', label: 'Is Active', type: 'checkbox', default: true },
      ],
    },
    events: {
      title: 'Events',
      api: '/api/admin/events',
      singular: 'Event',
      columns: ['id', 'title', 'slug', 'category', 'date', 'speaker_or_organizer', 'is_past'],
      canCreate: true,
      fields: [
        { name: 'title', label: 'Event Title *', type: 'text', required: true },
        {
          name: 'slug',
          label: 'Slug / URL Key (Leave empty to auto-generate)',
          type: 'text',
          placeholder: 'e.g. annual-metallurgy-symposium-2026',
        },
        {
          name: 'category',
          label: 'Category',
          type: 'select',
          options: [
            'Seminar',
            'Workshop',
            'Competition',
            'Industrial Visit',
            'Symposium',
            'Lab Visit',
            'Alumni Talk',
          ],
        },
        { name: 'date', label: 'Event Date *', type: 'date', required: true },
        {
          name: 'time_range',
          label: 'Time Range',
          type: 'text',
          placeholder: 'e.g. 10:00 AM – 1:00 PM',
        },
        {
          name: 'venue',
          label: 'Venue',
          type: 'text',
          placeholder: 'e.g. DMME Block Lecture Hall A',
        },
        {
          name: 'speaker_or_organizer',
          label: 'Speaker / Organizer',
          type: 'text',
          placeholder: 'e.g. Dr. Engr. Asim (NESCOM)',
        },
        { name: 'short_description', label: 'Short Description', type: 'textarea' },
        { name: 'full_summary', label: 'Full Agenda / Summary', type: 'textarea' },
        {
          name: 'poster_url',
          label: 'Poster Image URL',
          type: 'text',
          upload: true,
          placeholder: 'https:// or /uploads/...',
        },
        {
          name: 'registration_link',
          label: 'External Registration Link',
          type: 'text',
          placeholder: 'Optional external form',
        },
        { name: 'is_past', label: 'Mark as Past Event', type: 'checkbox', default: false },
        { name: 'is_featured', label: 'Feature on Homepage', type: 'checkbox', default: false },
      ],
    },
    registrations: {
      title: 'Event Registrations',
      api: '/api/admin/registrations',
      singular: 'Registration',
      columns: [
        'id',
        'event_id',
        'student_name',
        'roll_number',
        'batch',
        'department',
        'email',
        'certificate_code',
        'attended',
        'created_at',
      ],
      canCreate: false,
      canEdit: false,
    },
    team: {
      title: 'Team & Council Members',
      api: '/api/admin/team',
      singular: 'Member',
      columns: ['id', 'name', 'role', 'batch', 'tenure_year', 'is_current', 'is_faculty_advisor'],
      canCreate: true,
      fields: [
        { name: 'name', label: 'Full Name *', type: 'text', required: true },
        {
          name: 'role',
          label: 'Role / Designation *',
          type: 'text',
          required: true,
          placeholder: 'e.g. President, General Secretary',
        },
        { name: 'role_responsibility', label: 'Key Responsibilities', type: 'text' },
        {
          name: 'photo_url',
          label: 'Photo URL',
          type: 'text',
          upload: true,
          placeholder: '/uploads/... or https://',
        },
        { name: 'batch', label: 'Batch', type: 'text', placeholder: 'e.g. 2022-2026' },
        {
          name: 'tenure_year',
          label: 'Tenure Year *',
          type: 'text',
          required: true,
          placeholder: `e.g. ${currentAcademicYear()}`,
        },
        { name: 'is_current', label: 'Current Council Member', type: 'checkbox', default: true },
        { name: 'is_faculty_advisor', label: 'Faculty Advisor', type: 'checkbox', default: false },
        { name: 'bio', label: 'Short Bio', type: 'textarea' },
        { name: 'linkedin_url', label: 'LinkedIn Profile URL', type: 'text' },
        {
          name: 'email',
          label: 'Email Address',
          type: 'email',
          placeholder: 'e.g. member@pieas.edu.pk',
        },
        { name: 'display_order', label: 'Display Order', type: 'number', default: 0 },
      ],
    },
    posts: {
      title: 'Technical Posts & Articles',
      api: '/api/admin/posts',
      singular: 'Post',
      columns: [
        'id',
        'title',
        'slug',
        'category',
        'author_name',
        'material_formula',
        'published_at',
      ],
      canCreate: true,
      fields: [
        { name: 'title', label: 'Article Title *', type: 'text', required: true },
        {
          name: 'slug',
          label: 'Slug / URL Key (Leave empty to auto-generate)',
          type: 'text',
          placeholder: 'e.g. titanium-alloys-aerospace',
        },
        {
          name: 'category',
          label: 'Category',
          type: 'select',
          options: [
            'Material of the Month',
            'Explainer',
            'Technical',
            'Alloy Highlight',
            'Characterization',
            'Additive Manufacturing',
            'Student Research',
          ],
        },
        { name: 'excerpt', label: 'Excerpt Summary *', type: 'textarea', required: true },
        {
          name: 'body_markdown',
          label: 'Full Article Markdown *',
          type: 'textarea',
          required: true,
          rows: 10,
        },
        {
          name: 'material_formula',
          label: 'Chemical / Alloy Formula',
          type: 'text',
          placeholder: 'e.g. Zr-1.5Sn-0.2Fe or Inconel 718',
        },
        {
          name: 'crystal_structure',
          label: 'Crystal Structure',
          type: 'text',
          placeholder: 'e.g. HCP (α), FCC (γ)',
        },
        { name: 'cover_image_url', label: 'Cover Image URL', type: 'text', upload: true },
        {
          name: 'author_name',
          label: 'Author Name',
          type: 'text',
          default: 'PIEAS Materials Society',
        },
        {
          name: 'author_role',
          label: 'Author Title / Batch',
          type: 'text',
          placeholder: "e.g. Technical Head (DMME '26)",
        },
        {
          name: 'author_batch',
          label: 'Author Batch',
          type: 'text',
          placeholder: 'e.g. 2022-2026',
        },
        { name: 'read_time_minutes', label: 'Read Time (Minutes)', type: 'number', default: 5 },
        { name: 'is_featured', label: 'Feature on Homepage', type: 'checkbox', default: false },
      ],
    },
    resources: {
      title: 'Toolkit Resources & ASTM Standards',
      api: '/api/admin/resources',
      singular: 'Resource',
      columns: ['id', 'title', 'category', 'software_tag', 'url'],
      canCreate: true,
      fields: [
        { name: 'title', label: 'Resource Title *', type: 'text', required: true },
        {
          name: 'category',
          label: 'Category *',
          type: 'select',
          options: ['Standards', 'Software', 'Safety', 'Databases', 'Books', 'Tools'],
        },
        { name: 'description', label: 'Description', type: 'textarea' },
        { name: 'url', label: 'URL / Document Link *', type: 'text', upload: true, required: true },
        {
          name: 'software_tag',
          label: 'Software / Sub-Tag',
          type: 'text',
          placeholder: 'e.g. Thermo-Calc, ASTM E8, ImageJ',
        },
        { name: 'is_pinned', label: 'Pin to Top of Toolkit', type: 'checkbox', default: false },
      ],
    },
    opportunities: {
      title: 'Opportunities & Scholarships',
      api: '/api/admin/opportunities',
      singular: 'Opportunity',
      columns: ['id', 'title', 'organization', 'opportunity_type', 'deadline', 'is_active'],
      canCreate: true,
      fields: [
        { name: 'title', label: 'Position / Program Title *', type: 'text', required: true },
        {
          name: 'organization',
          label: 'Organization / University *',
          type: 'text',
          required: true,
        },
        {
          name: 'opportunity_type',
          label: 'Type *',
          type: 'select',
          options: [
            'Internship',
            'Scholarship',
            'Job',
            'Research',
            'Research Position',
            'Conference',
            'Competition',
          ],
        },
        {
          name: 'location',
          label: 'Location',
          type: 'text',
          placeholder: 'e.g. Islamabad, Remote, Germany',
        },
        { name: 'deadline', label: 'Application Deadline', type: 'date' },
        { name: 'description', label: 'Summary Details', type: 'textarea' },
        {
          name: 'posted_by_alumnus',
          label: 'Posted By Alumnus / Referral',
          type: 'text',
          placeholder: 'e.g. Dr. Fatima Zahra (Batch 2018)',
        },
        { name: 'apply_url', label: 'Application Link', type: 'text' },
        { name: 'is_active', label: 'Is Active', type: 'checkbox', default: true },
      ],
    },
    alumni: {
      title: 'DMME Alumni Directory',
      api: '/api/admin/alumni',
      singular: 'Alumnus',
      columns: ['id', 'name', 'graduating_batch', 'current_role', 'current_organization'],
      canCreate: true,
      fields: [
        { name: 'name', label: 'Full Name *', type: 'text', required: true },
        {
          name: 'graduating_batch',
          label: 'Graduation Batch *',
          type: 'text',
          required: true,
          placeholder: 'e.g. 2018-2022',
        },
        { name: 'current_role', label: 'Current Role / Job Title', type: 'text' },
        { name: 'current_organization', label: 'Organization / Company', type: 'text' },
        {
          name: 'industry_sector',
          label: 'Industry Sector',
          type: 'text',
          placeholder: 'e.g. Aerospace, Nuclear, Steel, Semiconductor',
        },
        {
          name: 'location',
          label: 'Location',
          type: 'text',
          placeholder: 'e.g. Cambridge, MA, USA',
        },
        { name: 'quote_testimonial', label: 'Quote / Testimonial', type: 'textarea' },
        { name: 'linkedin_url', label: 'LinkedIn Profile URL', type: 'text' },
        { name: 'photo_url', label: 'Photo URL', type: 'text', upload: true },
      ],
    },
    projects: {
      title: 'FYP & Student Projects',
      api: '/api/admin/projects',
      singular: 'Project',
      columns: ['id', 'title', 'project_type', 'batch', 'supervisors'],
      canCreate: true,
      fields: [
        { name: 'title', label: 'Project Title *', type: 'text', required: true },
        {
          name: 'project_type',
          label: 'Project Type',
          type: 'select',
          options: ['FYP', 'Graduate Research', 'Term Project', 'Competition'],
        },
        {
          name: 'batch',
          label: 'Batch *',
          type: 'text',
          required: true,
          placeholder: 'e.g. 2022-2026',
        },
        {
          name: 'students',
          label: 'Student Investigators',
          type: 'text',
          placeholder: 'Names separated by commas',
        },
        {
          name: 'supervisors',
          label: 'Faculty Supervisors',
          type: 'text',
          placeholder: 'Dr. ..., Engr. ...',
        },
        { name: 'abstract', label: 'Abstract / Technical Summary', type: 'textarea' },
        {
          name: 'tags',
          label: 'Tags / Keywords',
          type: 'text',
          placeholder: 'e.g. Titanium, SLM, Microstructure',
        },
        { name: 'image_url', label: 'Project Image URL', type: 'text', upload: true },
        { name: 'paper_or_repo_url', label: 'Paper / Code Repository URL', type: 'text' },
      ],
    },
    gallery: {
      title: 'Gallery & Micrographs',
      api: '/api/admin/gallery',
      singular: 'Gallery Item',
      columns: ['id', 'image_url', 'caption', 'event_name', 'year'],
      canCreate: true,
      fields: [
        {
          name: 'image_url',
          label: 'Image URL *',
          type: 'text',
          upload: true,
          required: true,
          placeholder: 'https:// or /uploads/...',
        },
        {
          name: 'caption',
          label: 'Caption / Micrograph Spec',
          type: 'text',
          placeholder: 'e.g. Ti-6Al-4V SLM Microstructure (1000x)',
        },
        { name: 'event_name', label: 'Event / Context', type: 'text' },
        { name: 'event_id', label: 'Linked Event ID (Optional)', type: 'number' },
        {
          name: 'year',
          label: 'Year *',
          type: 'text',
          required: true,
          default: String(new Date().getFullYear()),
        },
      ],
    },
    sponsors: {
      title: 'Industry Partners & Sponsors',
      api: '/api/admin/sponsors',
      singular: 'Sponsor',
      columns: ['id', 'name', 'tier', 'since_year', 'website_url'],
      canCreate: true,
      fields: [
        { name: 'name', label: 'Partner / Company Name *', type: 'text', required: true },
        {
          name: 'tier',
          label: 'Partnership Tier',
          type: 'select',
          options: ['Platinum', 'Gold', 'Silver', 'Bronze', 'Partner'],
        },
        { name: 'logo_url', label: 'Logo URL', type: 'text', upload: true, required: false },
        { name: 'website_url', label: 'Website URL', type: 'text' },
        {
          name: 'since_year',
          label: 'Partner Since (Year)',
          type: 'text',
          placeholder: 'e.g. 2024',
        },
        { name: 'display_order', label: 'Display Order', type: 'number', default: 0 },
      ],
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
        {
          name: 'category',
          label: 'Category',
          type: 'select',
          options: [
            'General',
            'Membership',
            'Events',
            'Academic',
            'Careers',
            'Council',
            'Certificates',
            'Collaboration',
          ],
        },
        { name: 'display_order', label: 'Display Order', type: 'number', default: 0 },
      ],
    },
    sections: {
      title: 'Dynamic Site Sections',
      api: '/api/admin/sections',
      singular: 'Site Section',
      columns: ['id', 'section_key', 'title', 'badge', 'updated_at'],
      canCreate: true,
      fields: [
        {
          name: 'section_key',
          label: 'Section Key * (Unique Identifier)',
          type: 'text',
          required: true,
          placeholder: 'e.g. home_hero, home_activities',
        },
        { name: 'title', label: 'Section Headline *', type: 'text', required: true },
        { name: 'subtitle', label: 'Subtitle', type: 'text' },
        { name: 'badge', label: 'Badge Text', type: 'text' },
        { name: 'content', label: 'Main Content Body *', type: 'textarea', required: true },
        {
          name: 'extra_json',
          label: 'Extra JSON (Cards / Links config)',
          type: 'textarea',
          rows: 4,
          placeholder: 'Valid JSON array or object',
        },
      ],
    },
    membership: {
      title: 'Membership Applications',
      api: '/api/admin/membership',
      singular: 'Application',
      columns: [
        'id',
        'full_name',
        'roll_number',
        'batch',
        'department',
        'email',
        'status',
        'created_at',
      ],
      canCreate: false,
      canEdit: false,
    },
    contacts: {
      title: 'Contact Messages',
      api: '/api/admin/contacts',
      singular: 'Message',
      columns: ['id', 'name', 'email', 'subject', 'message', 'created_at'],
      canCreate: false,
      canEdit: false,
    },
    newsletter: {
      title: 'Newsletter Subscribers',
      api: '/api/admin/newsletter',
      singular: 'Subscriber',
      columns: ['id', 'email', 'subscribed_at'],
      canCreate: false,
      canEdit: false,
    },
  };

  /** Every admin collection is paginated so no record is invisible past row 50. */
  const PAGINATED_TABS = new Set(Object.keys(SCHEMAS));

  window.PMSAdmin = Object.assign(window.PMSAdmin || {}, {
    currentTab: 'announcements',
    currentCache: [],
    currentEditingId: null,
    loadSeq: 0,
    eventsMap: {},
    PAGINATED_TABS,
    isUnauthorized,
    currentAcademicYear,
    fetchAllRows,
    SCHEMAS,
  });
})();
