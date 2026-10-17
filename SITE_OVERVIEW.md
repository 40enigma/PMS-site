# PIEAS Materials Society (PMS) — Platform Overview & Codebase Guide

This document provides a comprehensive technical overview of the PIEAS Materials Society (PMS) web platform, including system architecture, database design, API endpoints, frontend organization, and instructions for managing dynamic site sections via the Admin Panel.

---

## 1. System Architecture

The PMS platform is built as a monolithic, single-binary full-stack application:

```
┌─────────────────────────────────────────────────────────────┐
│                      Client Browser                         │
│   React 19 + Vite SPA (Tailwind CSS, Glassmorphic UI)       │
└──────────────────────────────▲──────────────────────────────┘
                               │ HTTP / JSON API
┌──────────────────────────────▼──────────────────────────────┐
│                  Rust Axum Web Server (Port 3000)           │
│                                                             │
│   ├── /api/auth/*         (JWT Bearer Authentication)       │
│   ├── /api/admin/*        (Secured CRUD Management)         │
│   ├── /api/*              (Public Endpoints & Search)       │
│   ├── /uploads/*          (Static PDF & Media Assets)       │
│   ├── /swagger-ui         (Interactive OpenAPI Docs)        │
│   └── /*                  (SPA Fallback -> frontend/dist)   │
└──────────────────────────────▲──────────────────────────────┘
                               │ SQLite Connection Pool (sqlx)
┌──────────────────────────────▼──────────────────────────────┐
│                      SQLite Database                        │
│                         (pms.db)                            │
└─────────────────────────────────────────────────────────────┘
```

- **Backend**: Rust 2021 edition using **Axum 0.8**, **SQLx 0.8 (SQLite)**, **Argon2** password hashing, **jsonwebtoken (JWT)** for admin sessions, **tower-http** for CORS, tracing, and static file serving, and **utoipa / utoipa-swagger-ui** for OpenAPI documentation.
- **Frontend**: **React 19**, **Vite 6**, **React Router v7**, **Tailwind CSS**, and **Lucide React** icons.
- **Single Deployment Artifact**: When running `cargo run`, Axum serves both the REST API and the compiled React production bundle from `frontend/dist/` with automated SPA routing fallback.

---

## 2. Directory Structure

```
PMS-site/
├── src/
│   ├── main.rs              # Application entrypoint, server routing, and OpenAPI config
│   ├── models.rs            # Data structures, serialization schemas, and request payloads
│   ├── db.rs                # SQLite migration runner, default seeds, and pool init
│   ├── auth.rs              # JWT token generation, verification, and AuthUser extractor
│   └── routes/
│       ├── public.rs        # Public read endpoints, searches, and form submissions
│       ├── admin.rs         # Protected admin CRUD, stats editor, and file uploader
│       └── auth.rs          # Admin login and session verification
├── frontend/
│   ├── src/
│   │   ├── api.js           # Central API client and fetch wrapper
│   │   ├── hooks/
│   │   │   └── index.js     # Custom hooks: useSections, useFetch, useCountUp, useTheme
│   │   ├── components/      # Reusable components (Navbar, Footer, Modal, Visualizer)
│   │   ├── pages/           # Page views (Home, About, Team, Events, Admin, etc.)
│   │   ├── App.jsx          # Top-level router with 404 fallback
│   │   └── index.css        # Tailwind directives and custom dark glassmorphism styling
│   ├── package.json         # Frontend dependencies and Vite scripts
│   └── vite.config.js       # Vite configuration with proxy to localhost:3000
├── uploads/                 # Static user uploads and constitution PDFs
├── pms.db                   # SQLite database file
├── Cargo.toml               # Rust dependencies and compiler configuration
└── SITE_OVERVIEW.md         # This technical documentation
```

---

## 3. Database Schema

The SQLite database (`pms.db`) consists of 14 core relational tables:

| Table Name | Description | Key Columns |
| :--- | :--- | :--- |
| `admin_users` | Administrators authorized to access `/admin` | `id, email, password_hash, name, created_at` |
| `events` | Workshops, technical seminars, and symposia | `id, title, date, venue, category, is_past, image_url` |
| `event_registrations` | Student attendees registered for events | `id, event_id, student_name, roll_number, certificate_code` |
| `team_members` | Council officers and faculty patrons | `id, name, role, batch, tenure_year, is_faculty_advisor` |
| `gallery_images` | High-res photos of workshops and tours | `id, image_url, caption, category, event_id` |
| `blog_posts` | Technical articles & Material of the Month | `id, title, slug, content, material_formula, published` |
| `faqs` | Frequently asked questions | `id, question, answer, category, display_order` |
| `resources` | Student study guides, software, ASTM notes | `id, title, category, software_tag, url, is_pinned` |
| `announcements` | Urgent banners and recruitment notices | `id, title, content, is_active, priority` |
| `opportunities` | Internships, scholarships, and jobs | `id, title, organization, type, deadline, is_active` |
| `projects` | Departmental research and FYP showcases | `id, title, abstract, supervisor, student_lead, year` |
| `sponsors` | Strategic partners (PAEC, NESCOM, HMC, etc.) | `id, name, tier, website_url, is_active` |
| `site_stats` | Singleton table for homepage impact counters | `id, active_members, events_held, industrial_partners` |
| `site_sections` | Dynamic text content for frontend pages | `id, section_key, title, subtitle, content, badge, extra_json` |

---

## 4. Dynamic Content Management (Site Sections)

Every primary text block, mission statement, portfolio strip, and collaborative pitch on the frontend is completely dynamic and editable via the Admin Panel without modifying code.

### Registered Section Keys

| Section Key | Frontend Page | Purpose |
| :--- | :--- | :--- |
| `home_hero` | `/` (Home) | Hero main headline, subtitle, and department badge |
| `home_activities` | `/` (Home) | Society portfolios grid (title, subtitle, and 4 activity cards) |
| `about_mission` | `/about` | Society mission card |
| `about_vision` | `/about` | Society vision card |
| `about_story` | `/about` | Detailed narrative on PMS founding & DMME heritage |
| `team_guidance` | `/team` | Executive council portfolios explainer & duties |
| `resources_standards`| `/resources` | Metallurgy ASTM testing cheatsheet strip |
| `sponsors_pitch` | `/sponsors` | Industry partnership benefits & collaboration proposal |

### How It Works in React
Pages use the `useSections()` hook from `src/hooks/index.js`:
```javascript
const { getSection } = useSections();
const hero = getSection('home_hero', {
  badge: 'DMME • PIEAS Islamabad',
  title: 'Engineering Matter from Atomic Lattices to Extreme Frontiers.',
  subtitle: 'The official student academic platform at DMME, PIEAS.'
});
```
- If the section exists in the backend database, its database values render seamlessly.
- If the network request is loading or the database is uninitialized, graceful hardcoded metallurgical fallbacks are displayed.

---

## 5. API Reference

### Public API Endpoints (`/api/*`)
All public endpoints support SQL-parameterized filtering:
- `GET /api/sections` — List all dynamic text sections
- `GET /api/stats` — Retrieve society impact metrics
- `GET /api/events?search=&category=&is_past=` — Filter events
- `POST /api/events/{id}/register` — Register for an event & receive instant certificate code
- `GET /api/certificates/verify?code=PMS-1-XXXXXX` — Case-insensitive certificate verification
- `GET /api/team?tenure_year=&batch=&search=` — Council and advisor list
- `GET /api/posts?category=&search=` — Technical blog articles & Material of the Month
- `GET /api/resources?category=&search=` — Software guides, phase diagrams, and ASTM standards
- `GET /api/opportunities?type=` — Scholarships and internships
- `GET /api/projects?search=&year=` — Department research projects
- `GET /api/sponsors` — Active industry and strategic partners
- `GET /api/gallery?category=` — Event photos
- `POST /api/membership` — Submit undergraduate society membership application
- `POST /api/contact` — Submit general inquiry

### Admin API Endpoints (`/api/admin/*`)
Requires HTTP Header: `Authorization: Bearer <JWT_TOKEN>`
- `GET / POST /api/admin/sections` — List or create dynamic content sections
- `GET / PUT / DELETE /api/admin/sections/{id}` — Retrieve, update, or remove a section
- `GET / PUT /api/admin/stats` — Update homepage metrics
- `POST /api/admin/upload` — Secure file uploader (Path-traversal sanitized, 20MB limit)
- Full CRUD endpoints for `/api/admin/events`, `/api/admin/team`, `/api/admin/posts`, `/api/admin/resources`, `/api/admin/opportunities`, `/api/admin/projects`, `/api/admin/sponsors`, `/api/admin/announcements`, `/api/admin/faqs`, `/api/admin/gallery`, `/api/admin/membership`.

---

## 6. Admin Credentials & Access

To access the administrative dashboard:
1. Navigate to: **`http://localhost:3000/admin`**
2. Login with credentials:
   - **Email**: `admin@pms.pieas.edu.pk`
   - **Password**: `PMS@Admin2026`
3. Use the navigation sidebar to:
   - **Sections**: Edit any headline, subtitle, or card across the entire website.
   - **Site Stats**: Update live numbers for active members, workshops held, and partners.
   - **Events & Registrations**: Manage events, view registered attendees, and review verification codes.
   - **Content Hub**: Create blog posts, student toolkit resources, opportunities, and sponsors.

---

## 7. Development & Production Commands

### Prerequisites
- **Rust & Cargo** (1.75+ or 1.99.1+)
- **Node.js** (v18+ or v24.22.0) and **npm**

### Building the Frontend
```bash
cd frontend
npm install
npm run build
```
*(Compiles the optimized React bundle into `frontend/dist/`)*

### Running the Backend Server
```bash
cargo run
```
The Axum server starts on `http://0.0.0.0:3000`:
- Website: `http://localhost:3000/`
- Admin Dashboard: `http://localhost:3000/admin`
- Swagger UI (OpenAPI Docs): `http://localhost:3000/swagger-ui`

### Running Full Automated Quality Verification
```bash
# Verify Rust Backend
cargo check
cargo clippy -- -D warnings

# Verify Frontend Build
cd frontend
npm run build
```
