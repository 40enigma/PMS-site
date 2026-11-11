# PIEAS Materials Society (PMS) — Official Web Platform

[![Rust](https://img.shields.io/badge/Rust-Axum%200.8-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Frontend](https://img.shields.io/badge/Frontend-Static%20HTML%2FCSS%2FJS-E34F26?logo=html5&logoColor=white)](#-frontend-architecture)
[![SQLite](https://img.shields.io/badge/Database-SQLite%20WAL-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![License](https://img.shields.io/badge/License-Academic%20Open-orange)](#license)

The official web platform and digital headquarters of the **PIEAS Materials Society (PMS)**, representing the **Department of Metallurgy & Materials Engineering (DMME)** at the **Pakistan Institute of Engineering and Applied Sciences (PIEAS)**, Nilore, Islamabad.

---

## 📖 Table of Contents

- [Purpose of the Website](#-purpose-of-the-website)
- [System Architecture](#-system-architecture)
- [Project Structure](#-project-structure)
- [Frontend Architecture (Static HTML)](#-frontend-architecture)
- [Environment & Security Configuration](#-environment--security-configuration)
  - [Environment Variables](#environment-variables)
  - [Generating a Secure JWT Secret](#generating-a-secure-jwt-secret)
  - [Intentional Admin Seeding](#intentional-admin-seeding)
- [How to Run Locally](#-how-to-run-locally)
  - [Development Mode](#development-mode)
  - [Production Mode](#production-mode)
  - [Running Tests](#running-tests)
- [Production Deployment Guide](#-production-deployment-guide)
  - [Hardened File Uploads & Object Storage Recommendations](#hardened-file-uploads--object-storage-recommendations)
  - [Systemd Background Service](#systemd-background-service)
  - [Nginx / Reverse Proxy Configuration](#nginx--reverse-proxy-configuration)
- [REST API Reference](#-rest-api-reference)

---

## 🎯 Purpose of the Website

The PMS web platform serves as an all-in-one academic, logistical, and community portal for undergraduate and graduate materials engineering students, faculty, and industry partners:

1. **Academic & Research Portfolios**:
   - **Undergraduate Materials Toolkit**: Direct access to ASTM testing standards (*ASTM E8 tensile testing*, *ASTM E112 grain sizing*, *ASTM E23 impact testing*), metallographic etching safety protocols, and computational software guides (*Thermo-Calc*, *JMatPro*, *ImageJ*, *Python pymatgen*).
   - **Technical Articles & Blog**: Technical explainers on critical engineering alloys (e.g., *Zircaloy-4 nuclear cladding*, *Inconel 718 aerospace superalloys*, *High-Entropy Alloys*).
   - **Final Year Project (FYP) Showcase**: Archive of departmental student research in additive manufacturing, nuclear materials, and metallurgy.

2. **Society Operations & Logistics**:
   - **Event Registrations & Announcements**: Online registration for technical seminars, hands-on workshops, and industrial visits.
   - **Verifiable Digital Participation Certificates**: Real-time passcode verification for event participants and employers.
   - **Council Portfolios & Directory**: Executive council member directories, portfolios, and faculty advisors.
   - **Alumni Network & Career Board**: Professional directory of DMME alumni and active internship/scholarship listings.

3. **Dynamic Content Management**:
   - Administrative dashboard allowing executive council members to update homepage headlines, mission statements, event listings, and resources directly via secured APIs.

---

## 🏗 System Architecture

The application is engineered as a **self-contained, monolithic Rust service**:

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Browser                         │
│       Pure Responsive Static HTML/CSS/JS (Zero Bundler)     │
└──────────────────────────────▲──────────────────────────────┘
                               │ HTTP / REST / JSON (httpOnly Cookie or Bearer)
┌──────────────────────────────▼──────────────────────────────┐
│                  Rust Axum Web Server (Port 3000)           │
│                                                             │
│   ├── /api/auth/*         (Rate-limited login & logout)     │
│   ├── /api/admin/*        (Secured CRUD Database Ops)       │
│   ├── /api/*              (Public Endpoints & Forms)        │
│   ├── /uploads/*          (Validated non-executable media)  │
│   └── /*                  (Static HTML: index, about, etc.) │
└──────────────────────────────▲──────────────────────────────┘
                               │ SQLite Connection Pool (sqlx)
┌──────────────────────────────▼──────────────────────────────┐
│                    SQLite 3 Embedded Engine                 │
│                 (pms.db with WAL mode enabled)              │
└─────────────────────────────────────────────────────────────┘
```

- **Backend**: Rust using [Axum 0.8](https://github.com/tokio-rs/axum), [SQLx 0.8](https://github.com/launchbadge/sqlx) (SQLite), [Argon2](https://github.com/RustCrypto/password-hashes), [jsonwebtoken](https://github.com/Keats/jsonwebtoken), and [governor](https://github.com/antifuchs/governor) for in-memory rate limiting.
- **Frontend**: Clean, responsive static HTML5, CSS3, and modular modern JavaScript (`frontend/static/`). **No Node.js build step or bundler is required!** Note: `frontend/static/index.html` is strictly required at runtime; the server verifies its presence at boot and will fail with a clear fatal error if absent.
- **Authentication**: `httpOnly`, `Secure`, `SameSite=Lax` cookie flow for browser clients, with backward-compatible `Authorization: Bearer <token>` support for programmatic API consumers (pass header `X-Return-Token: 1` during login to receive the token string in JSON).
- **Security Headers**: Strict `Content-Security-Policy` (with external deferred page scripts and no `'unsafe-inline'` script permissions), `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, and `Strict-Transport-Security` in production.

---

## 📁 Project Structure

```
PMS-site/
├── .env                       # Local environment variables (git-ignored)
├── .env.example               # Template environment configuration
├── Cargo.toml                 # Rust dependencies & package metadata
├── Cargo.lock                 # Deterministic dependency lockfile
├── pms.db                     # SQLite database (auto-created on first run)
├── migrations/                # Numbered database schema migrations (tracked in _pms_migrations)
│   ├── 0001_initial_schema.sql# Initial core database schema
│   ├── 0002_token_revocation.sql# Token revocation denylist for invalidated admin sessions
│   ├── 0003_email_lowercasing.sql# Lowercase email normalization
│   └── 0004_event_registrations_attended.sql# Added attended column to event_registrations
├── tests/                     # Integration tests
│   └── smoke_test.rs          # Smoke & security test suite
├── src/                       # Rust Backend
│   ├── lib.rs                 # Library exports, CORS, middleware, and create_app
│   ├── main.rs                # App entrypoint and HTTP listener
│   ├── models.rs              # SQLx structs and DTO schemas
│   ├── db.rs                  # Migrations, seed data, and SQLite connection pool
│   ├── auth.rs                # JWT token signing, verification, cookie extraction
│   ├── rate_limit.rs          # Governor per-IP rate limiter
│   ├── error.rs               # Safe uniform JSON error handling (AppError)
│   └── routes/
│       ├── mod.rs             # Module declarations
│       ├── public.rs          # Public endpoints (events, team, forms)
│       └── admin.rs           # Secured admin CRUD handlers and file uploaders
│
├── frontend/
│   └── static/                # Static HTML pages & assets served by Axum
│       ├── index.html         # Homepage
│       ├── about.html         # About & Society history
│       ├── team.html          # Executive council directory
│       ├── events.html        # Events list, registration, & certificate verify
│       ├── resources.html     # ASTM standards & software guides
│       ├── blog.html          # Technical articles
│       ├── opportunities.html # Internships & scholarships
│       ├── alumni.html        # Alumni directory
│       ├── projects.html      # Final Year Projects (FYP)
│       ├── gallery.html       # Laboratory & event photos
│       ├── sponsors.html      # Society sponsors & partners
│       ├── join.html          # Membership application & contact form
│       ├── admin.html         # Minimal administrative control console
│       ├── 404.html           # Not found page
│       ├── pms-logo.jpeg      # Society emblem
│       └── assets/
│           ├── style.css      # Shared design tokens, light/dark themes, responsive CSS
│           ├── theme.js       # Early dark/light theme initialization
│           ├── layout.js      # Shared navbar & footer injector (#site-header / #site-footer)
│           ├── app.js         # Lightweight API client, markdown renderer, & UI helpers
│           └── pages/         # Externalized deferred page scripts + admin/*.js modules
│
└── uploads/                   # Uploaded media and PDFs (validated & non-executable)
    └── .gitkeep
```

---

## 🌐 Frontend Architecture

The frontend is implemented with pure static HTML pages in `frontend/static/`.

### How Static Routing Works
The backend uses Axum's `ServeDir` and explicit route services:
- Visiting `/` serves `frontend/static/index.html`.
- Visiting `/about` serves `frontend/static/about.html`.
- Visiting `/team`, `/events`, `/resources`, `/blog`, `/opportunities`, `/alumni`, `/projects`, `/gallery`, `/sponsors`, `/join`, and `/admin` automatically resolve to their corresponding `.html` files.
- Visiting `/blog/{slug}` serves `frontend/static/blog.html` and opens the corresponding article directly, with canonical/OG tags rendered using `{{APP_URL}}` and static asset query strings versioned via `{{ASSET_V}}`.
- Unknown non-API paths return HTTP 404 with `frontend/static/404.html`.
- Static assets (stylesheets, scripts, images) are served directly under `/assets/*` or root.

### How to Edit Frontend Pages
- Simply edit the HTML files in `frontend/static/*.html`.
- Edit styles in `frontend/static/assets/style.css`.
- Edit API interaction logic in `frontend/static/assets/app.js`.
- **No compilation, npm install, or Vite build is needed.** Changes take effect immediately upon page refresh!

---

## 🔐 Environment & Security Configuration

The platform reads all security and networking configurations from `.env` or system environment variables at startup.

### Environment Variables

| Variable | Description | Default / Dev Fallback | Production Requirement |
| :--- | :--- | :--- | :--- |
| `RUST_ENV` | Application environment (`development` / `production`) | `development` | Set to `production` |
| `JWT_SECRET` | Cryptographic secret for signing tokens | Dev placeholder (warns) | **REQUIRED** (server fails fast if unset) |
| `ALLOWED_ORIGINS` | Comma-separated list of allowed CORS origins | `http://localhost:3000,http://127.0.0.1:3000` | **REQUIRED** (server fails fast if unset) |
| `HOST` | Interface IP to bind listener | `0.0.0.0` | `127.0.0.1` (behind reverse proxy) |
| `PORT` | Listening HTTP port | `3000` | `3000` |
| `DATABASE_URL` | SQLite connection string | `sqlite://pms.db?mode=rwc` | `sqlite://pms.db?mode=rwc` |
| `SEED_ADMIN` | Flag to authorize admin creation (`true`/`false`) | `false` | Set `true` only when creating initial admin |
| `ADMIN_INITIAL_EMAIL` | Initial admin account email | Unset | Required if `SEED_ADMIN=true` |
| `ADMIN_INITIAL_PASSWORD`| Initial admin account password (min 8 chars) | Unset | Required if `SEED_ADMIN=true` |
| `SEED_DATA` | Flag to enable demo data seeding (`true`/`false`) | `true` | `false` for clean production DB |
| `JWT_COOKIE_NAME` | Name of the authentication cookie | `pms_admin_token` | `pms_admin_token` |
| `JWT_COOKIE_SECURE` | Force Secure flag on auth cookie (`true`/`false`) | `false` in dev | `true` in production |
| `RATE_LIMIT_LOGIN_RPM` | Max login requests per minute per IP | `10` | `10` |
| `RATE_LIMIT_SUBMIT_RPM`| Max generic submissions per minute per IP | `30` | `30` |
| `RATE_LIMIT_REGISTER_RPM`| Max event registrations per minute per IP | `60` | `60` |
| `RATE_LIMIT_CONTACT_RPM` | Max contact form messages per minute per IP | `10` | `10` |
| `RATE_LIMIT_NEWSLETTER_RPM`| Max newsletter subscriptions per minute per IP | `10` | `10` |
| `RATE_LIMIT_MEMBERSHIP_RPM`| Max membership applications per minute per IP | `15` | `15` |
| `RATE_LIMIT_CERT_VERIFY_RPM`| Max certificate verification checks per minute per IP | `30` | `30` |
| `TRUST_PROXY` | Trust proxy headers (`X-Real-IP`, rightmost `X-Forwarded-For`) (`true`/`false`) | `false` | Set `true` behind reverse proxy (Nginx/Caddy) to prevent all users sharing `127.0.0.1` |
| `TRUST_CLOUDFLARE` | Trust `CF-Connecting-IP` / `True-Client-IP` when `TRUST_PROXY=true` | `false` | Set `true` only if deployed directly behind Cloudflare |
| `UPLOAD_DIR` | Directory to store uploaded media assets | `uploads` | `/var/www/pms-platform/uploads` |
| `APP_URL` | Canonical public base URL for `/sitemap.xml` and `{{APP_URL}}` HTML meta/canonical tags | `https://pms.pieas.edu.pk` | Your public site URL (e.g. `https://pms.pieas.edu.pk`) |
| `ASSET_V` | Override for `{{ASSET_V}}` cache-busting query string on `/assets/*` URLs | 8-char SHA-256 hash of `frontend/static/assets` | Optional explicit version override or leave unset for content hash |

### Generating a Secure JWT Secret
Run the following command on Linux/macOS to generate a cryptographically strong 256-bit hexadecimal key:

```bash
openssl rand -hex 32
```

### Intentional Admin Seeding
By default, the application **never** seeds a known admin password automatically. To create an initial admin intentionally, start the server once with explicit flags:

```bash
SEED_ADMIN=true \
ADMIN_INITIAL_EMAIL=admin@pieas.edu.pk \
ADMIN_INITIAL_PASSWORD='MySecretPassword123!' \
cargo run
```

Once created, future startups without `SEED_ADMIN=true` will leave the existing administrator account securely in place.

---

## 💻 How to Run Locally

### Development Mode

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Run the application:
   ```bash
   cargo run
   ```
   The platform will start at `http://localhost:3000`.

### Production Mode

In production, `RUST_ENV=production` enforces strict security validations (fails fast if `JWT_SECRET` or `ALLOWED_ORIGINS` is missing):

```bash
RUST_ENV=production \
JWT_SECRET=$(openssl rand -hex 32) \
ALLOWED_ORIGINS="https://pms.pieas.edu.pk" \
cargo run --release
```

### Running Tests

Run the test suite including smoke and integration tests:

```bash
cargo test
```

Check code formatting, linter, and optional zero-build frontend checks:
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
ESLINT_USE_FLAT_CONFIG=false npx --yes eslint@8 frontend/static/assets
npx --yes prettier --check "frontend/static/assets/**/*.js"
npx --yes -p typescript tsc --noEmit --allowJs --checkJs frontend/static/assets/app.js frontend/static/assets/layout.js
```

---

## 🚀 Production Deployment Guide

### Hardened File Uploads & Object Storage Recommendations

The backend upload handler (`/api/admin/upload`) includes comprehensive safeguards:
- **Magic Bytes Validation**: File contents are sniffed using the `infer` crate to verify actual file headers.
- **Whitelist Enforcement**: Only `image/png`, `image/jpeg`, `image/webp`, `application/pdf`, and `text/plain` are accepted.
- **Blocked Script Extensions**: Executables (`.exe`, `.sh`, `.php`, `.cgi`, `.py`, `.js`, etc.) are unconditionally rejected.
- **File System Permissions**: Saved files are given strict `0o644` (read-only, non-executable) permissions.
- **Safe Filenames**: Path traversal attacks are stripped and UUIDs are prepended.

> **Production Recommendation**:
> While local uploads are stored safely under `uploads/`, for large-scale production deployments we strongly recommend serving user uploads from a dedicated object store (such as Amazon S3, Cloudflare R2, or MinIO) backed by a CDN. This offloads static asset delivery from the application server and isolates user content from the host filesystem.

### Systemd Background Service

Create `/etc/systemd/system/pms.service`:

```ini
[Unit]
Description=PIEAS Materials Society (PMS) Web Application
After=network.target

[Service]
Type=simple
User=pms
Group=pms
WorkingDirectory=/var/www/pms-platform
ExecStart=/var/www/pms-platform/target/release/pms-platform
Restart=always
RestartSec=5
EnvironmentFile=/var/www/pms-platform/.env

# Security Sandbox
ProtectSystem=full
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pms
```

### Nginx / Reverse Proxy Configuration

```nginx
server {
    listen 80;
    server_name pms.pieas.edu.pk;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name pms.pieas.edu.pk;

    ssl_certificate /etc/letsencrypt/live/pms.pieas.edu.pk/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/pms.pieas.edu.pk/privkey.pem;

    client_max_body_size 25M;

    # Backend API and Static Pages
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        # Clear Cloudflare headers unless PMS is deployed directly behind Cloudflare
        proxy_set_header CF-Connecting-IP "";
        proxy_set_header True-Client-IP "";
    }

    # Static uploads with caching and security headers
    location /uploads/ {
        alias /var/www/pms-platform/uploads/;
        expires 30d;
        add_header Cache-Control "public, no-transform";
        add_header X-Content-Type-Options "nosniff" always;
        add_header Content-Security-Policy "default-src 'none'; style-src 'unsafe-inline'; sandbox" always;
        add_header X-Frame-Options "DENY" always;
    }
}
```

### SQLite Database Backup Command
Because SQLite is used in Write-Ahead Logging (WAL) mode, safe online backups can be performed without interrupting ongoing write transactions:

```bash
# Ad-hoc point-in-time database snapshot
sqlite3 pms.db ".backup 'pms-backup-$(date +%F).db'"
```

**Recommended Nightly Cron Task**:
```cron
0 2 * * * sqlite3 /var/www/pms-platform/pms.db ".backup '/var/backups/pms/pms-$(date +\%F).db'"
```

---

## 📡 REST API Reference

All list endpoints accept optional pagination query parameters: `limit` (default: 50, maximum: 200) and `offset` (default: 0). The response format is a direct JSON array.

### Public Endpoints

| Method | Endpoint | Description | Rate Limit |
| :--- | :--- | :--- | :--- |
| `GET` | `/healthz` | Service health status (`{"status":"ok"}`) after `SELECT 1` ping | Unthrottled |
| `GET` | `/sitemap.xml` | Dynamic XML sitemap listing canonical static and content URLs | Unthrottled |
| `GET` | `/api/stats` | Society metrics (active members, events held, years active, contact) | Cached 60s |
| `GET` | `/api/announcements` | Active announcement banners | Cached 60s |
| `GET` | `/api/events` | Event listings. Filters: `is_past=false` (sorted ASC) or `is_past=true` (sorted DESC), `is_featured=true` | Unthrottled |
| `GET` | `/api/events/{id}` | Detailed event information | Unthrottled |
| `POST` | `/api/events/{id}/register` | Event registration (trims and lowercases email; issues certificate code) | `RATE_LIMIT_REGISTER_RPM` (60) |
| `GET` | `/api/certificates/verify?code=...` | Verify participation certificate (`valid: true` if `attended = 1`; otherwise `valid: false` with `reason: "not_attended"` or `"not_found"`) | `RATE_LIMIT_CERT_VERIFY_RPM` (30) |
| `GET` | `/api/team` | Team directory and council members | Unthrottled |
| `GET` | `/api/resources` | Categorized ASTM standards and academic guides | Unthrottled |
| `GET` | `/api/posts` | Technical articles summary list (omits `body_markdown` for bandwidth efficiency) | Unthrottled |
| `GET` | `/api/posts/{slug}` | Full article details including `body_markdown` | Unthrottled |
| `GET` | `/api/opportunities` | Active opportunities (automatically hides opportunities with `deadline < today`) | Unthrottled |
| `GET` | `/api/alumni` | Alumni profiles and testimonials | Unthrottled |
| `GET` | `/api/projects` | Student FYPs and research projects | Unthrottled |
| `GET` | `/api/gallery` | Micrographs and gallery items | Unthrottled |
| `GET` | `/api/sponsors` | Active industry partners and sponsors | Unthrottled |
| `GET` | `/api/faqs` | Published frequently asked questions | Unthrottled |
| `GET` | `/api/sections` | Dynamic site content sections | Cached 60s |
| `POST` | `/api/membership` | Submit membership application (trims and lowercases email) | `RATE_LIMIT_MEMBERSHIP_RPM` (15) |
| `POST` | `/api/contact` | Submit inquiry message to the council | `RATE_LIMIT_CONTACT_RPM` (10) |
| `POST` | `/api/newsletter` | Subscribe to newsletter bulletin | `RATE_LIMIT_NEWSLETTER_RPM` (10) |

### Authentication & Admin Endpoints

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/api/auth/login` | Administrator login via Argon2 (`{"email", "password"}`). Issues httpOnly session cookie. Pass header `X-Return-Token: 1` to also receive JWT in JSON body for programmatic clients. |
| `POST` | `/api/auth/logout` | Revokes the current verified JWT session in `revoked_tokens` and clears session cookie. |
| `GET` | `/api/auth/me` | Returns current admin profile. |
| `POST` | `/api/admin/upload` | Multipart file upload (validates MIME bytes: PNG/JPG/WebP/PDF/TXT; stores as `{uuid}.{ext}`). |
| `GET` | `/api/admin/registrations` | Admin list of event registrations (supports `limit` and `offset`). |
| `PUT` | `/api/admin/registrations/{id}` | Toggle participant attendance (`{"attended": true \| false}`). |
| `GET` | `/api/admin/membership` | Admin list of membership applications (supports `limit` and `offset`). |
| `GET` | `/api/admin/contacts` | Admin list of contact messages (supports `limit` and `offset`). |
| `GET` | `/api/admin/newsletter` | Admin list of newsletter subscribers (supports `limit` and `offset`). |
| `GET/POST/PUT/DELETE`| `/api/admin/*` | Secured CMS CRUD endpoints for announcements, events, posts, team, resources, opportunities, alumni, projects, gallery, sponsors, faqs, sections, and stats. |

---

## 📜 License

Distributed under the [MIT License](LICENSE). Developed for the **Department of Metallurgy & Materials Engineering (DMME)**, **Pakistan Institute of Engineering and Applied Sciences (PIEAS)**.
