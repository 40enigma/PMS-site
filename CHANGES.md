# Changes & Release Notes — PIEAS Materials Society (PMS) Platform

This document details all bugs fixed, architectural improvements, files touched, new environment variables, and operational configurations applied to the PMS platform codebase across all phases.

---

## 📋 Summary of Fixed Issues by Identifier

### Phase 1: Critical Production Blockers
- **B-01 (`src/auth.rs`, `.env.example`)**: In production mode (`RUST_ENV=production`), application terminates with a fatal exit message if `JWT_SECRET` is unset, shorter than 32 characters, or contains placeholder text (`replace-with`, `placeholder`). Commented out `JWT_SECRET` in `.env.example` with cryptographic generation instructions.
- **B-02 (`src/db.rs`, `tests/smoke_test.rs`)**: In `run_migrations`, always executes `INSERT OR IGNORE INTO site_stats (id) VALUES (1)` so `/api/stats` and admin stats editor function properly when `SEED_DATA=false`. Added `test_migrations_only_stats_returns_200`.
- **B-03 (`src/rate_limit.rs`, `README.md`, `.env.example`)**: Ceased trusting `CF-Connecting-IP` / `True-Client-IP` under plain `TRUST_PROXY`. Added separate `TRUST_CLOUDFLARE` flag (default `false`); reads Cloudflare headers only when both `TRUST_PROXY=true` and `TRUST_CLOUDFLARE=true`. With `TRUST_PROXY=true`, prefers `X-Real-IP`, then the rightmost `X-Forwarded-For` hop. Startup warns in production if `TRUST_PROXY` is unset. Updated Nginx proxy configuration in `README.md`.
- **B-04 (`src/routes/admin.rs`)**: `logout()` only records tokens into `revoked_tokens` if `verify_token` succeeds (using its decoded `exp` timestamp). Applied `check_submit` rate limiting to logout. Added background task `spawn_revoked_token_pruner` pruning expired entries every 10 minutes, plus a startup prune in `run_migrations`.
- **B-06 (`src/routes/admin.rs`, `src/lib.rs`, `tests/smoke_test.rs`)**: File uploads ignore client-supplied filenames. Magic MIME sniffing verifies type (`image/png`, `image/jpeg`, `image/webp`, `application/pdf`, `text/plain`), saving files as `{uuid}.{ext}`. Wrapped `/uploads` in security middleware adding `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; sandbox`, `X-Content-Type-Options: nosniff`, and `Cache-Control: public, max-age=2592000`.
- **C-01 (`.env.example`, `.env.production.example`)**: Set `HOST=127.0.0.1`, removed default admin password in example, removed stale `:5173` origin, defaulted `SEED_DATA=false`. Created `.env.production.example`.

### Phase 2: Backend Correctness and Robustness
- **B-05 (`src/routes/admin.rs`)**: Wrapped Argon2 password verification in `tokio::task::spawn_blocking` inside `login()`. Maintained uniform timing dummy-hash verification for non-existent users.
- **B-07 (`src/routes/public.rs`, `src/routes/admin.rs`, `migrations/0004_event_registrations_attended.sql`, `tests/smoke_test.rs`)**: Normalized emails (trimmed and lowercased) on event registration and membership applications. Added `attended BOOLEAN NOT NULL DEFAULT 0` column via migration `0004`. Added `PUT /api/admin/registrations/{id}` to toggle attendance. Updated `/api/certificates/verify` to require `attended = 1`.
- **B-08 (`src/db.rs`)**: Table seeding gated on `_db_metadata.initial_seed_done` (executes only once). Changed `SEED_ADMIN` query to `ON CONFLICT(email) DO NOTHING`. Removed redundant `UPDATE site_sections` statements. Defaulted `SEED_DATA` to false in production.
- **B-09 (`src/db.rs`, `migrations/0003_email_lowercasing.sql`, `tests/smoke_test.rs`)**: Migration runner consults `_pms_migrations` table and applies only unapplied migration files sequentially in transactions. Moved email lowercasing to migration `0003`. Added migration test on pre-populated database (`test_migrations_on_copy_of_existing_db`).
- **B-10 (`src/rate_limit.rs`, `.env.example`, `README.md`)**: Configured independent rate limiters for registration (`RATE_LIMIT_REGISTER_RPM`, default 60), contact (`RATE_LIMIT_CONTACT_RPM`, default 10), newsletter (`RATE_LIMIT_NEWSLETTER_RPM`, default 10), and membership (`RATE_LIMIT_MEMBERSHIP_RPM`, default 15). Documented `RATE_LIMIT_CERT_VERIFY_RPM` and `UPLOAD_DIR`.
- **B-11 (`src/routes/public.rs`)**: Event past/upcoming status dynamically computed from date (`YYYY-MM-DD` compared to today) in addition to `is_past` flag (`is_past = flag || date < today`). Registration disabled if past. Upcoming events sorted date ASC; past events sorted date DESC. Opportunities hide expired postings (`deadline < today`) and sort NULL deadlines last.
- **B-12 (`src/routes/public.rs`, `src/routes/admin.rs`, `src/models.rs`)**: Excluded `body_markdown` from `/api/posts` list query (`CAST(NULL AS TEXT)`), returning it only on detail endpoint `/api/posts/{slug}`. Added `limit` (default 50, max 200) and `offset` query parameters to public list endpoints and admin lists.
- **B-13 (`src/util.rs`, `src/routes/admin.rs`, `src/routes/public.rs`)**: Consolidated `slugify` and SQLite `is_unique_violation` helper into shared `src/util.rs`. Mapped unique constraint violations to `AppError::Conflict` (409) across create and update endpoints.
- **B-14 (`Cargo.toml`, `tests/smoke_test.rs`)**: Added `serial_test = "3.2"` dev-dependency and marked env/global-limiter tests with `#[serial]`. Cleaned up files in upload tests.
- **B-15 (`src/lib.rs`)**: Added `x-requested-with` to CORS allowed headers.
- **B-16 (`src/lib.rs`, `src/error.rs`, `tests/smoke_test.rs`)**: Added JSON 404 fallback for unmatched `/api/*` routes. Converted CSRF header check failure to 403 Forbidden. Standardized 413/422/400 rejections to standard JSON error format (`{ "error": "...", "status": N }`). Added `Retry-After: 60` header on 429 Too Many Requests responses.
- **B-17 (`src/routes/admin.rs`, `tests/smoke_test.rs`)**: Used `COALESCE` in `update_stats` and `update_section` queries so omitted optional fields retain their existing values, while explicit empty strings clear the field.
- **B-18 (`src/rate_limit.rs`)**: Called `ip.to_canonical()` before masking IPv6 addresses to `/64` subnets.
- **B-19 (`src/auth.rs`)**: `AdminClaims` extractor fails closed with `AppError::Internal` if `SqlitePool` extension is missing.
- **B-20 (`src/models.rs`, `src/routes/admin.rs`, `tests/smoke_test.rs`)**: Set `LoginResponse.token` to `Option<String>`, returning token only if request contains header `X-Return-Token: 1`.
- **B-21 (`src/lib.rs`, `Cargo.toml`)**: Added `Cache-Control: no-store` on `/api/admin/*` and `/api/auth/*`, HSTS in production, `Cache-Control: public, max-age=60` on public announcements/stats/sections, and `Cache-Control: public, max-age=86400` on `/assets`. Added `compression-br` to `tower-http`.
- **B-22 (`src/main.rs`)**: Wired graceful shutdown signal (`shutdown_signal()` listening for Ctrl-C and SIGTERM). Removed `create_dir_all("frontend/static")` and fail fast if `frontend/static/index.html` is absent.
- **B-23 (`frontend/static/assets/pages/admin.js`)**: Delete event confirmation explicitly warns that registrations and certificates for the event will be deleted.
- **B-24 (`src/db.rs`)**: Seeded `constitution_pdf_url` as `NULL` instead of a non-existent file.
- **B-25 (`src/main.rs`)**: Initialized tracing subscriber before calling `jwt_secret()`.
- **B-26 (`src/lib.rs`, `tests/smoke_test.rs`)**: Added trailing slash tolerance for all pretty pages (`/about/`, etc.), and `/404` and `/404/` return HTTP status 404.
- **Health Check (`src/lib.rs`, `tests/smoke_test.rs`)**: Added `GET /healthz` endpoint returning 200 `{"status":"ok"}` upon `SELECT 1` DB ping (503 if unreachable).

### Phase 3: Frontend Fixes
- **F-01 (`frontend/static/assets/app.js`)**: Split URL handling into `safeUrlRaw(url)` (un-escaped URL string validation) and `safeUrl(url)` (escaped for HTML attribute interpolation). Used `safeUrlRaw` for property assignments (`el.href`, `img.src`). Exported both on `window.PMS`.
- **F-02 (`frontend/static/assets/pages/admin.js`)**: Neutralized CSV formula injection by prefixing `'` to any cell starting with `= + - @ \t \r`.
- **F-03 (`frontend/static/assets/app.js`)**: Enhanced `renderMarkdown` with ordered list support (`1. `), horizontal rules (`---`), and sanitized `![alt](url)` rendering via `safeUrlRaw`. Guarded link placeholder restoration against throwing on literal matches.
- **F-04 (`frontend/static/assets/pages/admin.js`)**: Added `loadSeq` request counter discarding stale async responses.
- **F-05 (`frontend/static/assets/pages/admin.js`)**: Select field renderer prepends current value if missing from options. Added seeded FAQ categories (Council, Certificates, Collaboration).
- **F-06 (`frontend/static/assets/app.js`)**: `apiFetch` properly merges headers and throws descriptive errors on invalid JSON.
- **F-07 (`frontend/static/assets/app.js`)**: `openRegistrationModal` persists issued certificate code in `localStorage` (`pms_cert_<id>`), provides a "Copy Code" button with clipboard integration, sets `aria-labelledby`, and closes on backdrop click.
- **F-08 (`frontend/static/assets/app.js`, `frontend/static/assets/style.css`, page scripts)**: Replaced `animation-delay` with `transition-delay` on staggered cards. Added `.no-js .fade-up` CSS fallback.
- **F-09 (`frontend/static/assets/app.js`, `frontend/static/assets/pages/gallery.js`)**: Micrograph lightbox closes on backdrop click, sets `aria-labelledby`, handles null captions, and triggers on Enter/Space keypress. Added `tabindex="0"`, `role="button"`, and `aria-label`.
- **F-10 (`frontend/static/assets/pages/*.js`, HTML pages)**: Subtitle elements added and rendered for `home_hero`, `home_activities`, `team_guidance`, and `sponsors_pitch`.
- **F-11 (`frontend/static/assets/app.js`)**: Article modal closes using `history.replaceState` preserving scroll position, and wired `popstate` event listener.
- **F-12 (`frontend/static/assets/pages/admin.js`)**: Made `section_key` readonly during site section editing.
- **F-13 (`frontend/static/assets/pages/team.js`)**: Faculty tab displays all faculty advisors (current first, former second with "Former" badge). Past tab displays past council members.
- **F-14 (`frontend/static/assets/pages/admin.js`)**: Automatically catches 401 Unauthorized and redirects to `showLogin()`.
- **F-15 (`frontend/static/assets/pages/resources.js`)**: Replaced loose substring category matching with an exact category alias map.
- **F-16 (`frontend/static/assets/pages/*.js`, `src/lib.rs`)**: Extracted all inline scripts from HTML pages into modular `assets/pages/*.js` loaded with `defer`. Removed `'unsafe-inline'` from `script-src` in CSP. Added `<meta name="robots" content="noindex,nofollow">` to `admin.html`.

### Phase 4: Dead Code and Cleanup
- Removed dead `fade-up-delay-*` classes from all HTML files.
- Removed unused `cleanup()` method from `RateLimitState` in `src/rate_limit.rs`.
- Removed unused `tower` dependency from `[dependencies]` in `Cargo.toml`.
- Configured release profile optimizations (`lto = true`, `codegen-units = 1`, `strip = true`) in `Cargo.toml`.
- Verified absence of stale `:5173` references across project files.

### Phase 5: Docs, CI, Missing Essentials
- **Documentation**: Updated `README.md` and `SITE_OVERVIEW.md` (removed deprecated `X-XSS-Protection` claim, corrected table count to 19 application + 2 metadata tables, documented migration tracking, runtime static files requirement, REST API reference, and SQLite backup procedure).
- **License**: Created `LICENSE` (MIT License).
- **CI Workflow**: Created `.github/workflows/ci.yml` running format check, clippy, unit/smoke tests, and security audit.
- **Containerization**: Created multi-stage non-root `Dockerfile` running as unprivileged `pmsuser:pms` on port 3000, along with `.dockerignore`.
- **Robots & Sitemap**: Created `frontend/static/robots.txt` and dynamic `/sitemap.xml` route in Axum app.
- **SEO & Accessibility**: Added `<meta property="og:*">` tags and canonical links on public pages. Added "Skip to main content" link and `<main id="main-content">` landmark across all HTML pages with `:focus-visible` styles in `style.css`.

### Phase 6 & Audit Remediation (Phases 1–6)
- **Data Integrity & API Correctness**:
  - Added `fetchAllRows` pagination (200 rows/page, up to 500 pages) for `/api/admin/registrations`, `/api/admin/membership`, `/api/admin/contacts`, and `/api/admin/newsletter` so CSV exports and counts include all rows.
  - Preserved `savedRegFilter` before clearing the admin toolbar, cleared optional stats/section strings with `''` while keeping `null` on create, replaced brittle 401 string matching with `isUnauthorized(err)` (`err.status === 401`), and added a sequence guard in `events.js` to discard stale async filter responses.
  - Escaped gallery `data-img-url` with `PMS.safeUrl`, normalized `'Partner'` tier labels, stopped HTML-escaping `mailto:` `href` attributes in `loadStats` (and removed dead stat selectors), formatted date-only and SQLite UTC timestamps consistently in `fmtDate`, joined only non-empty alumni subtitle parts, grouped past team members under `tenure_year`, and updated `renderMarkdown` to emit `<h2>`–`<h4>` with collision-proof per-call NUL-delimited tokens.
  - Added `CertificateVerification.reason` (`"not_found"` / `"not_attended"`), fixed `sitemap_xml` to emit `/blog/{slug}` and `/events`, and replaced hardcoded default years with computed academic tenure year (Aug–Jul rollover) and current calendar year.
- **Modals, Scroll Lock & Shared UX**:
  - Replaced click-to-close backdrop listeners with drag-safe `enableBackdropClose(dialog)` requiring `pointerdown` and `pointerup` outside dialog bounds.
  - Unified `.modal` / `.modal--sm|--md|--lg` styles and outline Close buttons in `style.css`, added `body:has(dialog[open])` scroll lock, wired `aria-labelledby` and focus restoration across public and admin dialogs, enforced `defer` ordering (`layout.js` -> `app.js` -> page script), switched announcement bar to slide/pause/dismiss, and rendered `"PIEAS Materials Society"` on a single navbar row on desktop.
- **Design Tokens, Light Theme, Accessibility & Performance**:
  - Expanded `:root` and `[data-theme="light"]` tokens (`color-scheme`, `--fs-*`, `--sp-*`, `--dur-*`, `--z-*`, `--tap-min`, `--overlay`, `--code-*`, `--tag-bg`, `--tier-*`), verified WCAG AA 4.5:1 text contrast on surface tokens, prevented grid overflow down to 320px, aligned `.fade-up` transitions with `.card`, enforced minimum 44px touch targets on coarse pointers, and converted clickable `<figure>` items, FAQ disclosures, form `aria-invalid`, and admin `role="tab"` keyboard navigation to accessible semantics.
  - Aligned static `index.html` hero markup with seeded database copy (`home_hero`) to prevent first-paint flash.
- **Routing, Asset Versioning, Shared Layout, Admin Split & Dev Tooling**:
  - Added `/blog/{slug}` and `/blog/{slug}/` Axum routes plus History API URL updates and `?article=<slug>` deep-link normalization in `app.js`.
  - Added server-side HTML template substitution (`render_html`) for `{{ASSET_V}}` (from `ASSET_V` env var or 8-char SHA-256 hash of `frontend/static/assets`) and `{{APP_URL}}` across all HTML templates.
  - Extracted duplicated navbar/footer HTML into `frontend/static/assets/layout.js` (`#site-header` / `#site-footer`) and split monolithic `pages/admin.js` into `pages/admin/{schemas,auth,modals,tools,table}.js`.
  - Added `PMS.isEmail`, `PMS.emptyState`, `PMS.errorState`, removed unused SVG icons, added explicit image `width`/`height`/`decoding="async"`/`loading="lazy"`, and configured ESLint v8 (`.eslintrc.json`), Prettier (`.prettierrc`), TypeScript `--checkJs` (`@ts-check` in `app.js`/`layout.js`), and the CI `frontend-lint` job.

---

## 📂 Files Touched & Created

### Modified Files:
- `.env.example`
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `SITE_OVERVIEW.md`
- `src/auth.rs`
- `src/db.rs`
- `src/error.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/models.rs`
- `src/rate_limit.rs`
- `src/routes/admin.rs`
- `src/routes/public.rs`
- `tests/smoke_test.rs`
- `frontend/static/assets/app.js`
- `frontend/static/assets/style.css`
- `frontend/static/index.html`
- `frontend/static/about.html`
- `frontend/static/team.html`
- `frontend/static/events.html`
- `frontend/static/resources.html`
- `frontend/static/blog.html`
- `frontend/static/opportunities.html`
- `frontend/static/alumni.html`
- `frontend/static/projects.html`
- `frontend/static/gallery.html`
- `frontend/static/sponsors.html`
- `frontend/static/join.html`
- `frontend/static/admin.html`
- `frontend/static/404.html`

### Newly Created Files:
- `CHANGES.md` (This document)
- `LICENSE`
- `Dockerfile`
- `.dockerignore`
- `.env.production.example`
- `.github/workflows/ci.yml`
- `src/util.rs`
- `migrations/0003_email_lowercasing.sql`
- `migrations/0004_event_registrations_attended.sql`
- `frontend/static/robots.txt`
- `.eslintrc.json`
- `.prettierrc`
- `frontend/static/assets/theme.js`
- `frontend/static/assets/layout.js`
- `frontend/static/assets/pages/index.js`
- `frontend/static/assets/pages/about.js`
- `frontend/static/assets/pages/team.js`
- `frontend/static/assets/pages/events.js`
- `frontend/static/assets/pages/resources.js`
- `frontend/static/assets/pages/blog.js`
- `frontend/static/assets/pages/opportunities.js`
- `frontend/static/assets/pages/alumni.js`
- `frontend/static/assets/pages/projects.js`
- `frontend/static/assets/pages/gallery.js`
- `frontend/static/assets/pages/sponsors.js`
- `frontend/static/assets/pages/join.js`
- `frontend/static/assets/pages/admin/schemas.js` (replacing `frontend/static/assets/pages/admin.js`)
- `frontend/static/assets/pages/admin/auth.js`
- `frontend/static/assets/pages/admin/modals.js`
- `frontend/static/assets/pages/admin/tools.js`
- `frontend/static/assets/pages/admin/table.js`

---

## 🔑 New & Updated Environment Variables

| Variable | Type / Default | Description |
| :--- | :--- | :--- |
| `TRUST_CLOUDFLARE` | Boolean (`false`) | Reads `CF-Connecting-IP` / `True-Client-IP` only when both `TRUST_PROXY=true` and `TRUST_CLOUDFLARE=true`. |
| `UPLOAD_DIR` | String (`uploads`) | Filesystem directory path for storing validated static uploads. |
| `RATE_LIMIT_REGISTER_RPM` | Integer (`60`) | Per-IP requests per minute for `/api/events/{id}/register`. |
| `RATE_LIMIT_CONTACT_RPM` | Integer (`10`) | Per-IP requests per minute for `/api/contact`. |
| `RATE_LIMIT_NEWSLETTER_RPM` | Integer (`10`) | Per-IP requests per minute for `/api/newsletter`. |
| `RATE_LIMIT_MEMBERSHIP_RPM` | Integer (`15`) | Per-IP requests per minute for `/api/membership`. |
| `RATE_LIMIT_CERT_VERIFY_RPM` | Integer (`30`) | Per-IP requests per minute for `/api/certificates/verify`. |
| `APP_URL` | String (`https://pms.pieas.edu.pk`) | Canonical base URL for `/sitemap.xml` and `{{APP_URL}}` HTML canonical/OG placeholders. |
| `ASSET_V` | String (optional; default 8-char SHA-256 digest of `frontend/static/assets`) | Cache-busting query value substituted for `{{ASSET_V}}` in HTML `/assets/*` URLs. |

---

## ⚙️ Items Skipped or Architectural Decisions

1. **`RateLimitState::default`**:
   - Clippy rule `clippy::new_without_default` requires `impl Default for RateLimitState` when `pub fn new() -> Self` takes no parameters. The method was retained while the truly unused `cleanup()` method was removed.
2. **Migrations Ordering**:
   - `migrations/0002_token_revocation.sql` created the `revoked_tokens` table and expiration index. To ensure existing and clean databases upgrade identically without destructive changes, migration `0003_email_lowercasing.sql` was added to apply idempotent lowercase normalization across all tables, and migration `0004_event_registrations_attended.sql` added the `attended` column. Both upgrades are verified in the test suite.
3. **CSP Nonce vs External Page Scripts**:
   - Rather than introducing dynamic server-side HTML templating or script nonces into a pure static file server, all page-specific scripts were extracted into static deferred JavaScript files (`assets/pages/*.js`). This cleanly eliminated the need for `'unsafe-inline'` script execution under Content-Security-Policy.

