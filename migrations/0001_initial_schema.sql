-- 0001_initial_schema.sql
-- PIEAS Materials Society (PMS) Platform Initial Database Schema

CREATE TABLE IF NOT EXISTS admin_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS announcements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message TEXT NOT NULL,
    link_text TEXT,
    link_url TEXT,
    priority TEXT NOT NULL DEFAULT 'info',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS site_stats (
    id INTEGER PRIMARY KEY,
    active_members INTEGER NOT NULL DEFAULT 0,
    events_held INTEGER NOT NULL DEFAULT 0,
    years_active INTEGER NOT NULL DEFAULT 0,
    industrial_partners INTEGER NOT NULL DEFAULT 0,
    constitution_pdf_url TEXT,
    office_location TEXT,
    contact_email TEXT
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    date TEXT NOT NULL,
    time_range TEXT,
    venue TEXT,
    category TEXT NOT NULL DEFAULT 'Seminar',
    short_description TEXT,
    full_summary TEXT,
    poster_url TEXT,
    registration_link TEXT,
    speaker_or_organizer TEXT,
    is_past BOOLEAN NOT NULL DEFAULT 0,
    is_featured BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS event_registrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    student_name TEXT NOT NULL,
    roll_number TEXT NOT NULL,
    batch TEXT NOT NULL,
    department TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    certificate_code TEXT UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS team_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    role_responsibility TEXT,
    photo_url TEXT,
    batch TEXT,
    tenure_year TEXT NOT NULL DEFAULT '2025-2026',
    is_current BOOLEAN NOT NULL DEFAULT 1,
    is_faculty_advisor BOOLEAN NOT NULL DEFAULT 0,
    bio TEXT,
    linkedin_url TEXT,
    email TEXT,
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS gallery_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_url TEXT NOT NULL,
    caption TEXT,
    event_id INTEGER REFERENCES events(id),
    event_name TEXT,
    year TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    excerpt TEXT,
    body_markdown TEXT,
    author_name TEXT,
    author_batch TEXT,
    cover_image_url TEXT,
    category TEXT NOT NULL DEFAULT 'Explainer',
    material_formula TEXT,
    crystal_structure TEXT,
    published_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS resources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'Software Guides',
    software_tag TEXT,
    description TEXT,
    is_pinned BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS opportunities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    organization TEXT NOT NULL,
    opportunity_type TEXT NOT NULL DEFAULT 'Internship',
    location TEXT,
    deadline TEXT,
    apply_url TEXT,
    description TEXT,
    posted_by_alumnus TEXT
);

CREATE TABLE IF NOT EXISTS alumni (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    batch TEXT NOT NULL,
    current_position TEXT,
    organization TEXT,
    location TEXT,
    photo_url TEXT,
    quote_testimonial TEXT,
    linkedin_url TEXT,
    research_domain TEXT
);

CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    student_names TEXT NOT NULL,
    batch TEXT NOT NULL,
    supervisor_name TEXT,
    project_type TEXT NOT NULL DEFAULT 'FYP Highlight',
    abstract_summary TEXT,
    image_url TEXT,
    paper_or_repo_url TEXT,
    tags TEXT
);

CREATE TABLE IF NOT EXISTS sponsors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    logo_url TEXT,
    website_url TEXT,
    tier TEXT NOT NULL DEFAULT 'Partner'
);

CREATE TABLE IF NOT EXISTS faqs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS membership_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL,
    roll_number TEXT NOT NULL,
    batch TEXT NOT NULL,
    department TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    interests TEXT,
    why_join TEXT,
    status TEXT NOT NULL DEFAULT 'Pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contact_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS newsletter_subscribers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    subscribed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS site_sections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    section_key TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    subtitle TEXT,
    content TEXT NOT NULL,
    badge TEXT,
    extra_json TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
