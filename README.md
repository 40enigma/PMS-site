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
│   ├── /swagger-ui         (Interactive OpenAPI Docs)        │
│   └── /*                  (Static HTML: index, about, etc.) │
└──────────────────────────────▲──────────────────────────────┘
                               │ SQLite Connection Pool (sqlx)
┌──────────────────────────────▼──────────────────────────────┐
│                    SQLite 3 Embedded Engine                 │
│                 (pms.db with WAL mode enabled)              │
└─────────────────────────────────────────────────────────────┘
```

- **Backend**: Rust using [Axum 0.8](https://github.com/tokio-rs/axum), [SQLx 0.8](https://github.com/launchbadge/sqlx) (SQLite), [Argon2](https://github.com/RustCrypto/password-hashes), [jsonwebtoken](https://github.com/Keats/jsonwebtoken), and [governor](https://github.com/antifuchs/governor) for in-memory rate limiting.
- **Frontend**: Clean, responsive static HTML5, CSS3, and vanilla modern JavaScript (`frontend/static/`). **No Node.js build step or bundler is required!**
- **Authentication**: `httpOnly`, `Secure`, `SameSite=Lax` cookie flow for browser clients, with backward-compatible `Authorization: Bearer <token>` support for programmatic API consumers.
- **Security Headers**: Automatic `X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, and `X-XSS-Protection` headers on all responses.

---

## 📁 Project Structure

```
PMS-site/
├── .env                       # Local environment variables (git-ignored)
├── .env.example               # Template environment configuration
├── Cargo.toml                 # Rust dependencies & package metadata
├── Cargo.lock                 # Deterministic dependency lockfile
├── pms.db                     # SQLite database (auto-created on first run)
├── migrations/                # Database schema migrations
│   └── 0001_initial_schema.sql# Idempotent SQLite initial schema
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
│           ├── style.css      # Shared design tokens & responsive CSS
│           └── app.js         # Lightweight API client & UI helpers
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
| `RATE_LIMIT_SUBMIT_RPM`| Max form submissions per minute per IP | `30` | `30` |
| `TRUST_PROXY` | Trust proxy headers (`X-Forwarded-For`/`X-Real-IP`) (`true`/`false`) | `false` | Set `true` behind reverse proxy (Nginx/Caddy) to prevent all users sharing `127.0.0.1` |

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

Check code formatting and linter:
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
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
    }

    # Static uploads with caching
    location /uploads/ {
        alias /var/www/pms-platform/uploads/;
        expires 30d;
        add_header Cache-Control "public, no-transform";
    }
}
```

---

## 📡 REST API Reference

The platform features an automated interactive OpenAPI Swagger specification at:

```
http://localhost:3000/swagger-ui
```

### Summary of Core Endpoints

- `GET /api/stats` — Society metrics (active members, events held, years active)
- `GET /api/announcements` — Active ticker announcements
- `GET /api/events` — Upcoming and past symposiums & workshops
- `POST /api/events/{id}/register` — Register for an event (rate-limited)
- `GET /api/certificates/verify?code=...` — Verify digital certificate validity
- `GET /api/team` — Current executive council and faculty advisor profiles
- `GET /api/resources` — ASTM testing guides and computational tools
- `GET /api/posts` — Technical articles and explainers
- `POST /api/membership` — Submit student membership application (rate-limited)
- `POST /api/contact` — Send society inquiry message (rate-limited)
- `POST /api/newsletter` — Subscribe to newsletter (rate-limited)
- `POST /api/auth/login` — Administrator authentication & httpOnly cookie issuance (rate-limited)
- `POST /api/auth/logout` — Admin logout & cookie clearing
- `POST /api/admin/upload` — Multipart file & image uploader (hardened, authenticated)
- `GET/POST/PUT/DELETE /api/admin/*` — Protected CMS CRUD operations

---

## 📜 License

Developed for the **Department of Metallurgy & Materials Engineering (DMME)**, **Pakistan Institute of Engineering and Applied Sciences (PIEAS)**. All rights reserved. Available for academic and student society operations.
