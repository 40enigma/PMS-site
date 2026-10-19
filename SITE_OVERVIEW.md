# PIEAS Materials Society (PMS) — Platform Overview & Architecture Guide

This document provides a comprehensive technical guide to the PIEAS Materials Society (PMS) web platform, including system architecture, database schema, security mechanisms, API endpoints, frontend organization, and administration workflows.

---

## 1. System Architecture

The PMS platform is engineered as a self-contained, single-binary full-stack service in Rust (Axum 0.8) with an embedded SQLite 3 database and zero-bundler static frontend:

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Browser                         │
│     Vanilla HTML5 / Modern CSS / ES6 JavaScript Static UI   │
│             (Served directly from frontend/static/)         │
└──────────────────────────────▲──────────────────────────────┘
                               │ HTTP / REST / JSON (httpOnly Cookie or Bearer)
┌──────────────────────────────▼──────────────────────────────┐
│                  Rust Axum Web Server (Port 3000)           │
│                                                             │
│   ├── /api/auth/*         (Rate-limited login, logout, me)  │
│   ├── /api/admin/*        (Secured CRUD Management)         │
│   ├── /api/*              (Public Endpoints & Forms)        │
│   ├── /uploads/*          (Non-executable user media)       │
│   ├── /assets/*           (Static CSS & JS stylesheets)     │
│   └── /*                  (Static HTML pages: /about, etc.) │
│       └── Fallback        (HTTP 404 with 404.html)          │
└──────────────────────────────▲──────────────────────────────┘
                               │ SQLite Connection Pool (sqlx WAL Mode)
┌──────────────────────────────▼──────────────────────────────┐
│                    SQLite 3 Embedded Engine                 │
│                 (pms.db with foreign keys enabled)          │
└─────────────────────────────────────────────────────────────┘
```

- **Backend**: Rust 2021 edition using **Axum 0.8**, **SQLx 0.8 (SQLite)**, **Argon2** password hashing, **jsonwebtoken (JWT)** for administrative sessions, **tower-http** for CORS, tracing, compression, and static file serving, and **governor** for IP-safe rate limiting.
- **Frontend**: High-performance, responsive static HTML5, vanilla modern CSS with design tokens, glassmorphism, responsive navigation, and lightweight modular ES6 JavaScript (`frontend/static/assets/app.js`). **Zero Node.js build step, bundler, or npm dependencies required.**
- **Single Deployment Artifact**: Axum serves the REST API and the static HTML/CSS/JS frontend with automated routing and 404 fallback.

---

## 2. Directory Structure

```
PMS-site/
├── src/
│   ├── main.rs              # Entrypoint, TCP listener, ConnectInfo injection, tracing init
│   ├── lib.rs               # Application router, CORS, modern security headers, 404 fallback
│   ├── models.rs            # Data structures, serialization schemas
│   ├── db.rs                # SQLite connection pool, single-source migration runner, seed logic
│   ├── auth.rs              # Cached JWT secret/cookie OnceLock, token generation & extractor
│   ├── rate_limit.rs        # In-memory IP rate limiter (TRUST_PROXY aware, DashMap pruning)
│   └── routes/
│       ├── mod.rs           # Module exports (public, admin)
│       ├── public.rs        # Public read endpoints, filter queries, rate-limited submissions
│       └── admin.rs         # Protected admin CRUD, stats editor, file uploader, session verification
├── migrations/
│   └── 0001_initial_schema.sql # Canonical single source of truth for all 16 SQLite tables
├── frontend/
│   └── static/
│       ├── index.html       # Homepage with dynamic hero & activities
│       ├── about.html       # Society mission, vision, history, and FAQs
│       ├── team.html        # Executive council, faculty advisors, past councils
│       ├── events.html      # Technical seminars, workshops, registration modal
│       ├── resources.html   # ASTM standards, metallurgy software guides, categorized toolkit
│       ├── blog.html        # Materials explainers with full technical article reader modal
│       ├── opportunities.html # Internships, graduate scholarships, industry jobs
│       ├── alumni.html      # DMME alumni network and career profiles
│       ├── projects.html    # Final Year Projects (FYP) and student research archive
│       ├── gallery.html     # High-resolution micrographs and event photography
│       ├── sponsors.html    # Industrial sponsors and collaboration pitch
│       ├── join.html        # Membership application form
│       ├── admin.html       # Full Admin Console with Create/Edit modal, tabs, status toggle
│       ├── 404.html         # Responsive 404 page with complete navbar and footer
│       └── assets/
│           ├── style.css    # Premium design system tokens, components, responsive layout
│           └── app.js       # PMS namespace, safeUrl XSS sanitizer, API clients, helpers
├── uploads/                 # Storage for uploaded media assets (non-executable permissions)
├── tests/
│   └── smoke_test.rs        # Integration test suite (auth, rate limits, upload sniff, CRUD, 404)
├── Cargo.toml               # Rust dependencies and package configuration
├── .env.example             # Documented environment variables template
└── README.md                # Developer onboarding and production deployment manual
```

---

## 3. Database Schema (16 SQLite Tables)

The database schema is defined in `migrations/0001_initial_schema.sql` and initialized automatically on startup:

1. **`admin_users`**: Administrative credentials (`id`, `email` UNIQUE, `password_hash` via Argon2, `name`).
2. **`announcements`**: Top notification banners (`id`, `message`, `link_text`, `link_url`, `priority`, `is_active`, `created_at`).
3. **`site_stats`**: Departmental metrics (`id` PRIMARY KEY, `active_members`, `events_held`, `years_active`, `industrial_partners`, `constitution_pdf_url`, `office_location`, `contact_email`).
4. **`events`**: Technical workshops and seminars (`id`, `title`, `slug`, `date`, `time_range`, `venue`, `category`, `short_description`, `full_summary`, `poster_url`, `registration_link`, `speaker_or_organizer`, `is_past`, `is_featured`).
5. **`event_registrations`**: Participant registrations (`id`, `event_id` REFERENCES events, `student_name`, `roll_number`, `batch`, `department`, `email`, `phone`, `certificate_code`, `created_at`).
6. **`team_members`**: Executive council & faculty advisors (`id`, `name`, `role`, `role_responsibility`, `photo_url`, `batch`, `tenure_year`, `is_current`, `is_faculty_advisor`, `bio`, `linkedin_url`, `display_order`).
7. **`gallery_images`**: Micrograph repository and event photos (`id`, `image_url`, `caption`, `event_name`, `year`, `event_id`, `created_at`).
8. **`posts`**: Technical explainers and blog articles (`id`, `title`, `slug` UNIQUE, `category`, `excerpt`, `body_markdown`, `material_formula`, `crystal_structure`, `cover_image_url`, `author_name`, `author_role`, `published_at`, `read_time_minutes`, `is_featured`).
9. **`resources`**: Undergraduate toolkit, ASTM standards, software links (`id`, `title`, `category`, `description`, `url`, `software_tag`, `is_pinned`, `created_at`).
10. **`opportunities`**: Internships, scholarships, research postings (`id`, `title`, `organization`, `opportunity_type`, `location`, `deadline`, `description`, `apply_url`, `is_active`, `posted_by_alumnus`, `created_at`).
11. **`alumni`**: Departmental alumni directory (`id`, `name`, `graduating_batch`, `current_role`, `current_organization`, `industry_sector`, `location`, `quote_testimonial`, `linkedin_url`, `photo_url`).
12. **`projects`**: FYP and student research archive (`id`, `title`, `project_type`, `batch`, `supervisors`, `abstract`, `students`, `image_url`, `tags`, `paper_or_repo_url`, `created_at`).
13. **`sponsors`**: Corporate and industrial partners (`id`, `name`, `tier`, `logo_url`, `website_url`, `since_year`, `display_order`).
14. **`faqs`**: Frequently asked questions (`id`, `question`, `answer`, `category`, `display_order`).
15. **`membership_applications`**: Student society membership submissions (`id`, `full_name`, `roll_number`, `batch`, `department`, `email`, `phone`, `interests`, `why_join`, `status`, `created_at`).
16. **`site_sections`**: Dynamic editable page content and portfolio blocks (`id`, `section_key` UNIQUE, `title`, `subtitle`, `content`, `badge`, `extra_json`, `updated_at`).

*(Plus auxiliary tracking tables: `contact_messages`, `newsletter_subscribers`).*

---

## 4. Security & Hardening Architecture

1. **IP Rate Limiting & Anti-Spoofing**:
   - `X-Forwarded-For` and `X-Real-IP` are strictly untrusted unless `TRUST_PROXY=true` is explicitly set in production behind Nginx/Caddy.
   - When `TRUST_PROXY=true`, the rightmost proxy hop is extracted.
   - For direct TCP connections, `ConnectInfo<SocketAddr>` is extracted from the socket.
   - Governor's in-memory `DashMapStateStore` is automatically pruned every 5 minutes (`retain_recent()` and `shrink_to_fit()`) to prevent memory exhaustion attacks.
2. **CORS & Origin Isolation**:
   - Allowed origins are enforced via `ALLOWED_ORIGINS`. In production mode (`RUST_ENV=production`), missing origins result in an immediate fail-fast exit.
   - `x-forwarded-for` and `x-real-ip` headers are stripped from CORS `allow_headers`.
3. **Modern Security Headers**:
   - `Content-Security-Policy`: Restricts scripts, styles, Google Fonts (`https://fonts.googleapis.com`, `https://fonts.gstatic.com`), and approved image hosts (`https://images.unsplash.com`, `'self'`, `data:`).
   - `Referrer-Policy: strict-origin-when-cross-origin`.
   - `X-Content-Type-Options: nosniff`.
   - `X-Frame-Options: SAMEORIGIN`.
4. **XSS URL Sanitization**:
   - Dynamic URLs placed into HTML `href` or `src` attributes are sanitized with `PMS.safeUrl(url)` which strictly permits `https://`, `http://`, `mailto:`, and relative `/` paths, disarming `javascript:` and malicious `data:` URIs.
5. **Admin Seeding Security**:
   - Default credentials are never committed or seeded by default. An admin user is only created when `SEED_ADMIN=true` is explicitly passed along with `ADMIN_INITIAL_EMAIL` and `ADMIN_INITIAL_PASSWORD`.

---

## 5. API Reference

All public, authentication, and administrative endpoints are served under `/api/` with JSON payloads and response definitions.
