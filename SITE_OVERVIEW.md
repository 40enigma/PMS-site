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
│   ├── 0001_initial_schema.sql              # Core schema for application tables
│   ├── 0002_token_revocation.sql            # Token revocation denylist for admin sessions
│   ├── 0003_email_lowercasing.sql           # Lowercase email normalization
│   └── 0004_event_registrations_attended.sql # Participant attendance tracking column
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
│           ├── style.css    # Design system tokens (dark & light themes), components, responsive layout
│           ├── theme.js     # Early theme initialization
│           ├── layout.js    # Shared public/admin navbar & footer injection
│           ├── app.js       # PMS namespace, safeUrl XSS sanitizer, API clients, helpers
│           └── pages/       # Modular deferred page scripts & split admin/*.js console modules
├── uploads/                 # Storage for uploaded media assets (non-executable permissions)
├── tests/
│   └── smoke_test.rs        # Integration test suite (auth, rate limits, upload sniff, CRUD, 404)
├── Cargo.toml               # Rust dependencies and package configuration
├── .env.example             # Documented environment variables template
└── README.md                # Developer onboarding and production deployment manual
```

---

## 3. Database Schema (19 Core Tables + Migration Metadata)

The database schema is managed via sequential, numbered migrations tracked in `_pms_migrations`:
- `migrations/0001_initial_schema.sql` — Initial schema establishing all core tables
- `migrations/0002_token_revocation.sql` — Created `revoked_tokens` table and expiration index for admin logout revocation
- `migrations/0003_email_lowercasing.sql` — Normalized emails to lowercase across all account and submission tables
- `migrations/0004_event_registrations_attended.sql` — Added `attended` BOOLEAN column to `event_registrations`

### Core Application Tables:

1. **`admin_users`**: Administrative credentials (`id`, `email` UNIQUE, `password_hash` via Argon2, `name`).
2. **`announcements`**: Top notification banners (`id`, `message`, `link_text`, `link_url`, `priority`, `is_active`, `created_at`).
3. **`site_stats`**: Departmental metrics (`id` PRIMARY KEY, `active_members`, `events_held`, `years_active`, `industrial_partners`, `constitution_pdf_url`, `office_location`, `contact_email`).
4. **`events`**: Technical workshops and seminars (`id`, `title`, `slug`, `date`, `time_range`, `venue`, `category`, `short_description`, `full_summary`, `poster_url`, `registration_link`, `speaker_or_organizer`, `is_past`, `is_featured`).
5. **`event_registrations`**: Participant registrations (`id`, `event_id` REFERENCES events, `student_name`, `roll_number`, `batch`, `department`, `email`, `phone`, `certificate_code`, `attended` DEFAULT 0, `created_at`).
6. **`team_members`**: Executive council & faculty advisors (`id`, `name`, `role`, `role_responsibility`, `photo_url`, `batch`, `tenure_year`, `is_current`, `is_faculty_advisor`, `bio`, `linkedin_url`, `email`, `display_order`).
7. **`gallery_images`**: Micrograph repository and event photos (`id`, `image_url`, `caption`, `event_name`, `year`, `event_id`, `created_at`).
8. **`posts`**: Technical explainers and blog articles (`id`, `title`, `slug` UNIQUE, `category`, `excerpt`, `body_markdown`, `material_formula`, `crystal_structure`, `cover_image_url`, `author_name`, `author_role`, `author_batch`, `published_at`, `read_time_minutes`, `is_featured`).
9. **`resources`**: Undergraduate toolkit, ASTM standards, software links (`id`, `title`, `category`, `description`, `url`, `software_tag`, `is_pinned`, `created_at`).
10. **`opportunities`**: Internships, scholarships, research postings (`id`, `title`, `organization`, `opportunity_type`, `location`, `deadline`, `description`, `apply_url`, `is_active`, `posted_by_alumnus`, `created_at`).
11. **`alumni`**: Departmental alumni directory (`id`, `name`, `graduating_batch`, `current_role`, `current_organization`, `industry_sector`, `location`, `quote_testimonial`, `linkedin_url`, `photo_url`).
12. **`projects`**: FYP and student research archive (`id`, `title`, `project_type`, `batch`, `supervisors`, `abstract`, `students`, `image_url`, `tags`, `paper_or_repo_url`, `created_at`).
13. **`sponsors`**: Corporate and industrial partners (`id`, `name`, `tier`, `logo_url`, `website_url`, `since_year`, `display_order`).
14. **`faqs`**: Frequently asked questions (`id`, `question`, `answer`, `category`, `display_order`).
15. **`membership_applications`**: Student society membership submissions (`id`, `full_name`, `roll_number`, `batch`, `department`, `email`, `phone`, `interests`, `why_join`, `status`, `created_at`).
16. **`contact_messages`**: Direct inquiries dispatched to the council (`id`, `name`, `email`, `subject`, `message`, `created_at`).
17. **`newsletter_subscribers`**: Email subscriptions for society bulletins (`id`, `email` UNIQUE, `subscribed_at`).
18. **`site_sections`**: Dynamic editable page content and portfolio blocks (`id`, `section_key` UNIQUE, `title`, `subtitle`, `content`, `badge`, `extra_json`, `updated_at`).
19. **`revoked_tokens`**: Token revocation denylist for invalidated administrator sessions (`token_hash` PRIMARY KEY, `expires_at`).

*(System metadata tables: `_pms_migrations` tracks applied schema versions; `_db_metadata` gates initial demo seeding).*

---

## 4. Security & Hardening Architecture

1. **IP Rate Limiting & Anti-Spoofing**:
   - `X-Forwarded-For` and `X-Real-IP` are strictly untrusted unless `TRUST_PROXY=true` is explicitly set in production behind Nginx/Caddy.
   - Cloudflare headers (`CF-Connecting-IP`, `True-Client-IP`) are only parsed when both `TRUST_PROXY=true` and `TRUST_CLOUDFLARE=true`.
   - Separate rate limiting buckets are enforced for login, generic submit, registrations (60 RPM), contacts (10 RPM), newsletter (10 RPM), and membership (15 RPM).
   - IPv6 addresses are canonicalized and masked to `/64` subnets before bucket lookup.
   - Governor's in-memory `DashMapStateStore` is automatically pruned every 5 minutes (`retain_recent()` and `shrink_to_fit()`).
2. **CORS & Origin Isolation**:
   - Allowed origins are enforced via `ALLOWED_ORIGINS`. In production mode (`RUST_ENV=production`), missing origins or weak configurations fail fast.
3. **Modern Security Headers**:
   - `Content-Security-Policy`: Scripts restricted to `'self'` (all page scripts are modular and externalized, zero `'unsafe-inline'`), styles (`'self' 'unsafe-inline' https://fonts.googleapis.com`), Google Fonts (`https://fonts.gstatic.com`), and approved image hosts.
   - `Strict-Transport-Security: max-age=31536000; includeSubDomains` in production.
   - `Referrer-Policy: strict-origin-when-cross-origin`.
   - `X-Content-Type-Options: nosniff`.
   - `X-Frame-Options: SAMEORIGIN`.
4. **Runtime Integrity**:
   - The application verifies that `frontend/static/index.html` exists at startup and halts with a fatal message if static assets are missing.
5. **XSS URL Sanitization**:
   - Dynamic URLs placed into HTML properties or attributes use `PMS.safeUrlRaw(url)` and `PMS.safeUrl(url)` which strictly validate permitted protocols (`https:`, `http:`, `mailto:`, `/`), preventing `javascript:` injection.

---

## 5. API Reference & Features

- **Health Check**: `GET /healthz` checks database connectivity with `SELECT 1` and returns 200 `{"status":"ok"}` (503 on error). Not rate limited.
- **Sitemap**: `GET /sitemap.xml` dynamically generates an XML sitemap using `APP_URL`, including static pages, article `/blog/{slug}` URLs, and `/events`.
- **Pagination**: All public and admin list endpoints support `limit` (default: 50, maximum: 200) and `offset` query parameters.
- **Certificate Verification**: `GET /api/certificates/verify?code=...` returns `valid: true` if `attended = 1`, or `valid: false` with `reason: "not_attended"` / `"not_found"`.
- **HTML Template Rendering**: Pretty pages and `/blog/{slug}` substitute `{{ASSET_V}}` (from `ASSET_V` env var or hashed assets digest) and `{{APP_URL}}` at serve time.
- **Attendance Toggling**: Admins can toggle event attendance via `PUT /api/admin/registrations/{id}` with `{"attended": true | false}`.
- **Programmatic Logins**: `POST /api/auth/login` supports optional header `X-Return-Token: 1` to receive the JWT token in the JSON body.
- **File Upload Security**: `/api/admin/upload` sniffs MIME magic bytes, generates safe UUID filenames `{uuid}.{ext}`, and wraps `/uploads` with strict sandbox and cache headers.
