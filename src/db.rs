use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand_core::OsRng;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Initialize the SQLite connection pool with WAL mode and foreign keys.
pub async fn init_pool() -> SqlitePool {
    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://pms.db?mode=rwc".to_string());
    let options = SqliteConnectOptions::from_str(&db_url)
        .expect("Invalid DB URL")
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create pool");

    pool
}

/// Run all CREATE TABLE IF NOT EXISTS migrations and track version.
pub async fn run_migrations(pool: &SqlitePool) {
    let schema = include_str!("../migrations/0001_initial_schema.sql");
    sqlx::raw_sql(schema)
        .execute(pool)
        .await
        .expect("Failed to run migrations");

    // Track migration version in _pms_migrations
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO _pms_migrations (version) VALUES ('0001_initial_schema')",
    )
    .execute(pool)
    .await;

    // Ensure site_sections default entries exist (runs for new and existing databases)
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO site_sections (section_key, title, subtitle, content, badge, extra_json) VALUES
        ('home_hero', 'Advancing Materials Science & Metallurgy at PIEAS', 'The official academic society of the Department of Metallurgy & Materials Engineering (DMME), PIEAS. Connecting students with research, practical engineering, and career development.', 'Advancing materials science and metallurgy at PIEAS.', 'DMME • PIEAS Islamabad', '{"cta_primary_text": "Join PMS", "cta_primary_url": "/join", "cta_secondary_text": "Student Resources", "cta_secondary_url": "/resources"}'),
        ('home_activities', 'Bridging Theory and Materials Engineering', 'PMS organizes initiatives designed to support metallurgy students in academic research and industrial engineering.', 'Society Portfolios', 'Society Portfolios', '[{"title":"Technical Seminars","desc":"Technical lectures by researchers and engineers on advanced materials, characterization, and metallurgical processing.","icon":"Atom"},{"title":"Hands-On Workshops","desc":"Practical training on metallurgical tools including Thermo-Calc, optical metallography, hardness testing, and computational modeling.","icon":"Cpu"},{"title":"Competitions & Olympiads","desc":"Materials design competitions, metallography contests, technical poster sessions, and engineering quizzes.","icon":"Award"},{"title":"Industrial & Lab Visits","desc":"Organized educational visits to metallurgical manufacturing units, testing laboratories, and research facilities.","icon":"Building"}]'),
        ('about_mission', 'Our Mission', 'Theory into Metallurgical Practice', 'To bridge academic metallurgical theory with practical industry applications. PMS equips students with hands-on exposure to phase diagrams, materials characterization, failure analysis, and professional engineering standards.', 'Mission', NULL),
        ('about_vision', 'Our Vision', 'Collaborative Community', 'To cultivate a collaborative community of materials engineers and researchers at PIEAS who contribute meaningfully to academia, strategic industries, and advanced manufacturing.', 'Vision', NULL),
        ('about_story', 'About PMS and DMME', 'Department & Society', 'The Department of Metallurgy & Materials Engineering (DMME) at PIEAS is dedicated to education and research in metallurgy, materials characterization, and advanced manufacturing. PMS was founded by students and faculty to provide an active extracurricular platform for technical workshops, guest lectures, industrial tours, and peer mentorship.', 'Department & Society', NULL),
        ('team_guidance', 'Council Portfolios & Responsibilities', 'Overview of responsibilities for executive positions within the PIEAS Materials Society.', 'Portfolios', 'Portfolios', '[{"title":"President","scope":"Executive Leadership & External Representation","desc":"Liaises with DMME faculty, administration, and partner institutions. Sets the society roadmap and coordinates executive initiatives."},{"title":"General Secretary","scope":"Operational Coordination & Administration","desc":"Manages society communications, official documentation, event venue permissions, and meeting agendas."},{"title":"Treasurer","scope":"Financial Governance & Budgeting","desc":"Maintains society accounts, event budgets, equipment disbursements, and financial records."},{"title":"Technical & Academic Head","scope":"Workshops & Academic Support","desc":"Coordinates technical workshops, guest lectures, and student academic resources for DMME coursework."},{"title":"Media & Publications Head","scope":"Visual Communication & Publications","desc":"Oversees event announcements, photography, technical articles, and society social media channels."},{"title":"Event Coordinator & Logistics Head","scope":"Event Management & Logistics","desc":"Coordinates logistics, equipment, venue setups, and industrial visits for society events."}]'),
        ('resources_standards', 'Materials Testing & Cheatsheets', 'ASTM & Metallography Protocols', 'Essential testing protocols and lab safety procedures referenced across DMME undergraduate coursework.', 'Standards', '[{"code":"ASTM E8 / E8M","title":"Tension Testing of Metallic Materials","detail":"Yield strength, UTS & elongation protocol"},{"code":"ASTM E112","title":"Determining Average Grain Size","detail":"Heyn lineal intercept & comparison charts"},{"code":"ASTM E23","title":"Charpy V-Notch Impact Testing","detail":"Ductile-to-brittle transition behavior (DBTT)"},{"code":"Acid Safety","title":"Nital & Kroll''s Reagents","detail":"HNO3 in ethanol (≤5%) & HF fume hood rules"}]'),
        ('sponsors_pitch', 'Partner with PIEAS Materials Society', 'Collaboration', 'Connect with materials engineering students at PIEAS for technical seminars, research projects, and campus engagement.', 'Collaboration', '[{"title":"Direct Talent Linkages","desc":"Engage directly with graduating metallurgy and materials engineering students for internships, trainee programs, and recruitment."},{"title":"Technical Event Visibility","desc":"Prominent organizational presence at society workshops, annual symposia, conference proceedings, and student competitions."},{"title":"Collaborative Projects","desc":"Support student research initiatives, industrial testing challenges, and metallurgical case studies with DMME faculty guidance."}]')
        "#
    )
    .execute(pool)
    .await
    .expect("Failed to seed default site_sections");

    // Normalize existing emails to lowercase safely, skipping and logging collisions to preserve data
    let colliding_admins: Vec<(i64, String)> = sqlx::query_as(
        r#"
        SELECT id, email FROM admin_users
        WHERE email != lower(email)
          AND lower(email) IN (
            SELECT lower(email) FROM admin_users GROUP BY lower(email) HAVING count(*) > 1
          )
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (id, email) in colliding_admins {
        tracing::warn!(
            "Skipping email lowercasing for admin_users id={} email='{}': collision with existing email",
            id, email
        );
    }

    let _ = sqlx::query(
        r#"
        UPDATE admin_users
        SET email = lower(email)
        WHERE email != lower(email)
          AND lower(email) NOT IN (
            SELECT lower(email) FROM admin_users GROUP BY lower(email) HAVING count(*) > 1
          )
        "#,
    )
    .execute(pool)
    .await;

    let colliding_subscribers: Vec<(i64, String)> = sqlx::query_as(
        r#"
        SELECT id, email FROM newsletter_subscribers
        WHERE email != lower(email)
          AND lower(email) IN (
            SELECT lower(email) FROM newsletter_subscribers GROUP BY lower(email) HAVING count(*) > 1
          )
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (id, email) in colliding_subscribers {
        tracing::warn!(
            "Skipping email lowercasing for newsletter_subscribers id={} email='{}': collision with existing email",
            id, email
        );
    }

    let _ = sqlx::query(
        r#"
        UPDATE newsletter_subscribers
        SET email = lower(email)
        WHERE email != lower(email)
          AND lower(email) NOT IN (
            SELECT lower(email) FROM newsletter_subscribers GROUP BY lower(email) HAVING count(*) > 1
          )
        "#,
    )
    .execute(pool)
    .await;

    tracing::info!("✅ Database migrations complete");
}

pub async fn seed_data(pool: &SqlitePool) {
    // --- Admin user seeding is EXPLICITLY GATED behind SEED_ADMIN=true ---
    let seed_admin = std::env::var("SEED_ADMIN")
        .unwrap_or_default()
        .to_lowercase()
        == "true";

    if seed_admin {
        let admin_email = match std::env::var("ADMIN_INITIAL_EMAIL") {
            Ok(e) if !e.trim().is_empty() => e.trim().to_lowercase(),
            _ => {
                tracing::error!(
                    "SEED_ADMIN=true but ADMIN_INITIAL_EMAIL is not set. \
                     Skipping admin seeding."
                );
                String::new()
            }
        };

        let admin_password = match std::env::var("ADMIN_INITIAL_PASSWORD") {
            Ok(p) if p.len() >= 8 => p,
            _ => {
                tracing::error!(
                    "SEED_ADMIN=true but ADMIN_INITIAL_PASSWORD is missing or too short. \
                     Skipping admin seeding."
                );
                String::new()
            }
        };

        if !admin_email.is_empty() && !admin_password.is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2
                .hash_password(admin_password.as_bytes(), &salt)
                .expect("Failed to hash password")
                .to_string();

            sqlx::query(
                "INSERT INTO admin_users (email, password_hash, name) VALUES (?1, ?2, ?3) ON CONFLICT(email) DO UPDATE SET password_hash = excluded.password_hash, name = excluded.name",
            )
            .bind(&admin_email)
            .bind(&password_hash)
            .bind("PMS Admin")
            .execute(pool)
            .await
            .expect("Failed to seed admin user");

            tracing::info!(email = %admin_email, "✅ Admin user seeded (SEED_ADMIN=true)");
        }
    } else {
        tracing::info!(
            "SEED_ADMIN is not set to 'true'. \
             No admin user will be created automatically. \
             Set SEED_ADMIN=true with credentials to create one."
        );
    }

    let seed_data = std::env::var("SEED_DATA")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true);

    if !seed_data {
        tracing::info!("SEED_DATA=false, skipping database seeding.");
        return;
    }

    tracing::info!("Checking and seeding database with initial academic society records...");

    sqlx::query(
        "INSERT OR IGNORE INTO site_stats (id, active_members, events_held, years_active, industrial_partners, constitution_pdf_url, office_location, contact_email) VALUES (1, 85, 42, 6, 12, '/uploads/pms-constitution.pdf', 'Department of Metallurgy & Materials Engineering (DMME) Block, PIEAS, Nilore, Islamabad', 'pms@pieas.edu.pk')"
    )
    .execute(pool)
    .await
    .expect("Failed to seed site stats");

    // Guard announcement seeding against duplicates
    let ann_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM announcements")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if ann_count.0 == 0 {
        sqlx::query(
            "INSERT INTO announcements (message, link_text, link_url, priority, is_active) VALUES
            ('PMS Annual Materials Symposium 2026 — Registrations Now Open', 'Register Now', '/events', 'event', 1),
            ('Applications for PMS Executive Council 2026-2027 are open until Nov 15', 'Apply Here', '/join', 'urgent', 1)"
        )
        .execute(pool)
        .await
        .expect("Failed to seed announcements");
    }

    // Guard events seeding against duplicates
    let evt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if evt_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO events (title, slug, date, time_range, venue, category, short_description, full_summary, speaker_or_organizer, is_past, is_featured) VALUES
            ('Annual Pakistan Materials Symposium (PMS ''26)', 'pms-symposium-2026', '2026-11-15', '09:00 AM - 05:00 PM', 'PIEAS Auditorium, Nilore Campus', 'Seminar', 'The flagship annual symposium bringing together researchers, students, and industry experts to discuss cutting-edge developments in materials science, metallurgy, and nuclear materials.', 'A full-day academic symposium featuring keynote lectures by distinguished researchers from PAEC, NESCOM, and international universities. Topics include advanced alloys for nuclear reactors, additive manufacturing of titanium alloys, and computational thermodynamics. Includes a poster competition and networking lunch with industry partners.', 'Dr. Amjad Hussain (Chief Scientist, PINSTECH) & PMS Council', 0, 1),
            ('Hands-on Workshop: Quantitative Metallography with ImageJ & Python', 'imagej-python-workshop', '2026-11-28', '02:00 PM - 05:30 PM', 'DMME Computer Lab, PIEAS', 'Workshop', 'Learn to quantify grain sizes (ASTM E112), phase fractions, and porosity from optical micrographs using ImageJ/Fiji and Python scripts with scikit-image.', 'A hands-on workshop where students will bring their own micrographs or use provided SEM/optical images of dual-phase steels and titanium alloys. Covers thresholding, watershed segmentation, grain boundary detection, and automated ASTM E112 grain size measurement. Python scripting with matplotlib for publication-quality figures.', 'Ahmed Raza (Technical Head, PMS) & Dr. Sarah Khan', 0, 1),
            ('Industrial Visit: Heavy Mechanical Complex (HMC), Taxila', 'hmc-taxila-visit', '2026-12-05', '07:00 AM - 06:00 PM', 'HMC Taxila (Bus from PIEAS Gate)', 'Lab Visit', 'An exclusive guided tour of Pakistan''s premier heavy engineering facility, witnessing large-scale forging, casting, and heat treatment of steel components for defense and energy sectors.', 'Students will visit the forging shop, foundry, heat treatment furnaces, and quality control/NDT labs at HMC Taxila. Observe how large steel ingots are processed into turbine shafts, pressure vessels, and armored vehicle components. Includes a Q&A with HMC metallurgists.', 'PMS Council & HMC Industry Relations', 0, 0),
            ('Technical Seminar: Radiation Damage in Nuclear Reactor Cladding Alloys', 'radiation-damage-seminar', '2026-09-20', '03:00 PM - 04:30 PM', 'DMME Seminar Hall, PIEAS', 'Seminar', 'Expert lecture on how neutron irradiation affects the microstructure and mechanical properties of Zircaloy-4 and stainless steel cladding in PWR/CANDU reactors.', 'Dr. Khalid Mahmood from PINSTECH delivered a comprehensive lecture on irradiation-induced defects (voids, dislocation loops), irradiation hardening, and embrittlement in reactor structural materials. Covered recent TEM analysis of irradiated Zircaloy samples from PARR-1. Over 60 students and faculty attended.', 'Dr. Khalid Mahmood (Senior Scientist, PINSTECH)', 1, 0),
            ('Metallographic Etching Competition — PMS Olympiad 2026', 'etching-competition-2026', '2026-08-10', '10:00 AM - 01:00 PM', 'DMME Metallography Lab, PIEAS', 'Competition', 'Teams of 3 students competed to prepare, polish, etch, and photograph the best optical micrograph of a provided steel sample within 2 hours.', 'The annual PMS Metallographic Etching Competition challenged 12 teams to reveal the microstructure of a mystery steel sample. Teams used Nital (2% HNO3 in ethanol) and Picral etchants, followed by optical microscopy. Judged on preparation quality, etching uniformity, and micrograph clarity. Winners received certificates and materials science textbooks.', 'PMS Technical Committee', 1, 0),
            ('Alumni Talk: From PIEAS to MIT — A Materials Engineer''s Journey', 'alumni-talk-mit', '2026-07-15', '04:00 PM - 05:30 PM', 'PIEAS Video Conference Room', 'Alumni Talk', 'DMME alumnus Dr. Fatima Zahra (MIT, Batch 2018) shared her journey from PIEAS undergraduate to PhD researcher at MIT, working on high-entropy alloys.', 'An inspiring virtual session where Dr. Fatima Zahra discussed her research on CoCrFeMnNi high-entropy alloys, grad school applications, GRE/TOEFL preparation tips, and the importance of undergraduate research. Over 80 students joined the session with an engaging 40-minute Q&A.', 'Dr. Fatima Zahra (PhD Candidate, MIT DMSE, Batch 2018)', 1, 0)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed events");
    }

    let team_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM team_members")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if team_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO team_members (name, role, role_responsibility, batch, tenure_year, is_current, is_faculty_advisor, bio, linkedin_url, email, display_order) VALUES
            ('Dr. Muhammad Asghar Hashmi', 'Faculty Advisor', 'Provides academic guidance, approves event proposals, and ensures PMS activities align with DMME departmental goals. Facilitates industry connections and mentors the executive council.', NULL, '2025-2026', 1, 1, 'Associate Professor at DMME, PIEAS. Research interests include physical metallurgy of advanced steels, phase transformations, and computational thermodynamics using CALPHAD methods. 15+ years of teaching and research experience.', 'https://linkedin.com/in/example', 'hashmi@pieas.edu.pk', 0),
            ('Bilal Ahmed Khan', 'President', 'Leads the PMS executive council, chairs all general body meetings, represents PMS at university and inter-university forums, and oversees strategic planning for the academic year.', '2022', '2025-2026', 1, 0, 'Final year BS Metallurgy & Materials Engineering student. Research interest in additive manufacturing of titanium alloys. Interned at PINSTECH in summer 2025.', 'https://linkedin.com/in/example', 'bilal.ahmed@pieas.edu.pk', 1),
            ('Ayesha Siddiqui', 'General Secretary', 'Manages all administrative operations, coordinates between council portfolios, maintains official records, and handles correspondence with PIEAS administration and external organizations.', '2023', '2025-2026', 1, 0, 'Third year student passionate about nuclear materials and corrosion engineering. Active member of the Pakistan Nuclear Society student chapter.', 'https://linkedin.com/in/example', 'ayesha.s@pieas.edu.pk', 2),
            ('Usman Tariq', 'Treasurer', 'Manages PMS finances, prepares event budgets, handles sponsorship funds, maintains transparent financial records, and coordinates with PIEAS accounts office.', '2023', '2025-2026', 1, 0, 'Third year student with interest in materials characterization (XRD, SEM, TEM). Won best poster award at PMS Symposium 2025.', 'https://linkedin.com/in/example', 'usman.t@pieas.edu.pk', 3),
            ('Hira Fatima', 'Technical Head', 'Plans and executes all technical events including seminars, workshops, and lab demonstrations. Curates the Material of the Month blog series and student resource toolkit.', '2023', '2025-2026', 1, 0, 'Third year student researching computational materials science using DFT and molecular dynamics. Proficient in VASP, Thermo-Calc, and Python for materials informatics.', 'https://linkedin.com/in/example', 'hira.f@pieas.edu.pk', 4),
            ('Hamza Ali Qureshi', 'Media & Publications Head', 'Manages PMS social media, designs event posters and promotional material, handles photography/videography at events, and edits the PMS newsletter.', '2024', '2025-2026', 1, 0, 'Second year student skilled in graphic design (Adobe Illustrator, Figma) and scientific photography. Documents all PMS events and manages the society Instagram.', 'https://linkedin.com/in/example', 'hamza.ali@pieas.edu.pk', 5),
            ('Zainab Malik', 'Event Coordinator', 'Handles logistics for all PMS events — venue booking, equipment setup, refreshments, participant registration, certificate generation, and post-event feedback collection.', '2024', '2025-2026', 1, 0, 'Second year student interested in welding metallurgy and non-destructive testing (NDT). Organized the successful PMS Metallographic Etching Competition 2026.', 'https://linkedin.com/in/example', 'zainab.m@pieas.edu.pk', 6)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed team members");

        sqlx::query(
            r#"INSERT INTO team_members (name, role, role_responsibility, batch, tenure_year, is_current, is_faculty_advisor, bio, display_order) VALUES
            ('Ali Hassan Rizvi', 'President', 'Led the PMS council during the 2024-2025 tenure. Organized the first-ever PMS Annual Symposium and established industry partnerships.', '2021', '2024-2025', 0, 0, 'BS Metallurgy & Materials Engineering graduate (2025). Currently pursuing MS at NUST in advanced ceramics.', 1),
            ('Mahnoor Iqbal', 'General Secretary', 'Managed administrative operations and coordinated the society''s first industry visit program to HMC Taxila and Pakistan Steel Mills.', '2022', '2024-2025', 0, 0, 'Final year student. Research focus on bio-compatible metallic implants and surface coatings.', 2),
            ('Saad Ullah', 'Technical Head', 'Initiated the Material of the Month blog series and organized 8 technical seminars during the 2024-2025 academic year.', '2022', '2024-2025', 0, 0, 'Final year student. Specialized in failure analysis and fractography. Intern at NDT lab, PAEC.', 3)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed past council");
    }

    let post_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if post_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO posts (title, slug, excerpt, body_markdown, author_name, author_role, author_batch, category, material_formula, crystal_structure, read_time_minutes, is_featured, published_at) VALUES
            ('Material of the Month: Zircaloy-4 & Inconel 718 — Superalloys Powering Extreme Environments', 'material-of-the-month-zircaloy-inconel', 'Exploring two critical engineering alloys that operate at the extremes — from nuclear reactor cores to jet engine turbines.', '## Zircaloy-4: The Guardian of Nuclear Fuel

Zircaloy-4 (Zr-1.5Sn-0.2Fe-0.1Cr) is the workhorse cladding material in Pressurized Water Reactors (PWRs) worldwide, including Pakistan''s KANUPP and Chashma nuclear power plants.

### Why Zircaloy?
- **Ultra-low neutron absorption cross-section** (0.18 barns) — essential for not wasting precious neutrons
- **Excellent corrosion resistance** in high-temperature water (300-350°C)
- **Adequate mechanical strength** up to 400°C
- **HCP crystal structure** that can be textured for optimal properties

### The Challenge: Hydrogen Pickup
During reactor operation, waterside corrosion produces hydrogen that diffuses into the cladding, forming brittle zirconium hydride platelets (ZrH₁.₆). This ''hydrogen embrittlement'' is a life-limiting degradation mechanism.

---

## Inconel 718: The Jet Engine Marvel

Inconel 718 (Ni-19Cr-18Fe-5Nb-3Mo-1Ti-0.5Al) is a precipitation-hardened nickel-base superalloy that maintains extraordinary strength up to 700°C.

### Strengthening Mechanisms
- **γ′′ (Ni₃Nb, BCT)** — primary strengthening precipitate
- **γ′ (Ni₃(Al,Ti), FCC)** — secondary strengthener
- **δ phase (Ni₃Nb, orthorhombic)** — grain boundary control

### Applications
Gas turbine discs, rocket engine components, nuclear reactor springs, and cryogenic storage tanks. One of the most versatile superalloys ever developed.', 'Hira Fatima', 'Technical Head (DMME ''23)', '2023', 'Material of the Month', 'Zr-1.5Sn-0.2Fe-0.1Cr / Ni-19Cr-18Fe-5Nb', 'HCP (Zircaloy) / FCC+BCT precipitates (Inconel)', 6, 1, '2026-10-01'),

            ('Why Do Welds Crack? A Visual Guide to Heat-Affected Zones and Hydrogen Embrittlement', 'why-welds-crack-haz-guide', 'Understanding the metallurgy behind weld failures — from HAZ microstructures to hydrogen-induced cold cracking.', '## The Heat-Affected Zone (HAZ)

When a steel plate is welded, the base metal adjacent to the weld pool experiences a complex thermal cycle. This region — the Heat-Affected Zone — undergoes phase transformations that can dramatically change its properties.

### HAZ Sub-Zones (from weld interface outward):
1. **Coarse-Grained HAZ (CGHAZ):** T > 1100°C. Prior austenite grains grow very large (>100 μm). Transforms to coarse martensite or bainite on cooling. **Most susceptible to cracking.**
2. **Fine-Grained HAZ (FGHAZ):** T ≈ 900-1100°C. Complete reaustenitization with fine grain size. Generally has good toughness.
3. **Inter-Critical HAZ (ICHAZ):** T ≈ 720-900°C. Partial transformation — a mix of fresh martensite islands in a ferrite matrix.
4. **Sub-Critical HAZ (SCHAZ):** T < 720°C. No phase transformation, but tempering/aging may occur.

## Hydrogen Embrittlement: The Silent Killer

Hydrogen atoms (from moisture, contaminated electrodes, or improper shielding gas) diffuse into the weld and HAZ during cooling. They accumulate at:
- Prior austenite grain boundaries
- Martensite lath boundaries
- Ahead of crack tips (stress-driven diffusion)

### Prevention Strategies:
- **Preheat** the workpiece (150-300°C) to slow cooling and allow hydrogen to escape
- **Use low-hydrogen electrodes** (E7018, baked at 350°C before use)
- **Post-Weld Heat Treatment (PWHT)** at 600-650°C to relieve residual stresses
- **Control carbon equivalent** CE(IIW) < 0.45 for weldable steels', 'Saad Ullah', 'Alumnus & Failure Analyst', '2022', 'Explainer', NULL, NULL, 5, 0, '2026-09-15'),

            ('How Steel Microstructures Transform: Pearlite, Bainite, and Martensite Explained', 'steel-microstructure-transformations', 'A visual journey through the CCT diagram — understanding how cooling rate determines whether you get soft pearlite or ultra-hard martensite.', '## The Austenite Decomposition Story

When austenite (γ-Fe, FCC) in a plain carbon steel (say 0.4% C) is cooled from above the A₃ temperature (~850°C), its fate depends entirely on the cooling rate.

### Slow Cooling → Pearlite
- **Structure:** Alternating lamellae of ferrite (α-Fe, BCC) and cementite (Fe₃C)
- **Hardness:** ~200 HV (coarse pearlite) to ~350 HV (fine pearlite)
- **Formation:** Diffusion-controlled. Carbon atoms must redistribute between ferrite and cementite.
- **Appearance:** Beautiful ''fingerprint'' pattern under optical microscope after Nital etch.

### Intermediate Cooling → Bainite
- **Upper Bainite (350-550°C):** Lath-shaped ferrite with cementite particles along lath boundaries. Moderate toughness.
- **Lower Bainite (250-350°C):** Plate-shaped ferrite with fine carbide precipitates within the plates. **Excellent combination of strength and toughness.**
- **Hardness:** 350-450 HV

### Rapid Quenching → Martensite
- **Structure:** Body-Centered Tetragonal (BCT) — a supersaturated solid solution of carbon in iron
- **Hardness:** 600-800 HV (depending on carbon content)
- **Formation:** Diffusionless, shear transformation. Austenite → Martensite almost instantaneously
- **The Problem:** Extremely hard but brittle. Must be **tempered** (heated to 150-650°C) to restore ductility.

### The CCT Diagram
The Continuous Cooling Transformation diagram maps these transformations as a function of cooling rate and temperature. It is the metallurgist''s most essential tool for heat treatment design.', 'Ahmed Raza', 'Metallurgy Scholar', '2023', 'Explainer', 'Fe-0.4C', 'FCC → BCC/BCT', 7, 0, '2026-08-20')"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed posts");
    }

    let res_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM resources")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if res_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO resources (title, url, category, software_tag, description, is_pinned) VALUES
            ('Thermo-Calc — Installation & CALPHAD Basics for PIEAS Students', 'https://thermocalc.com/academia/', 'Software', 'Thermo-Calc', 'Step-by-step guide to installing Thermo-Calc academic license, setting up TCFE12 steel database, and calculating your first phase diagram. Essential for Physical Metallurgy coursework.', 1),
            ('JMatPro — Predicting CCT/TTT Diagrams & Mechanical Properties', 'https://www.sentesoftware.co.uk/jmatpro', 'Software', 'JMatPro', 'Tutorial on using JMatPro to predict continuous cooling transformation diagrams, TTT diagrams, and mechanical properties of steels and nickel alloys. Includes example calculations for AISI 4340.', 1),
            ('ImageJ/Fiji — Grain Size Measurement (ASTM E112) Tutorial', 'https://imagej.net/software/fiji/', 'Software', 'ImageJ', 'Complete workflow for quantitative metallography: image calibration, thresholding, watershed segmentation, and automated grain size measurement per ASTM E112. Includes practice micrographs.', 1),
            ('OriginPro — Publication-Quality Plots for Materials Data', 'https://www.originlab.com/origin', 'Software', 'Origin', 'Guide to creating XRD patterns, stress-strain curves, and phase diagrams in OriginPro with proper axis labels, fonts, and export settings for journal submission.', 0),
            ('Python for Materials Science — pymatgen & matplotlib Crash Course', 'https://pymatgen.org/', 'Software', 'Python', 'Introduction to pymatgen for crystal structure analysis, phase diagram computation, and materials database queries. Includes Jupyter notebooks with worked examples for DMME coursework.', 1),
            ('Materials Project — Free Online Materials Database', 'https://materialsproject.org/', 'Databases', NULL, 'Access computed phase diagrams, crystal structures, band structures, and elastic properties for 150,000+ inorganic materials. Essential for computational materials research and FYP projects.', 1),
            ('ASM International Alloy Center & Handbooks', 'https://www.asminternational.org/', 'Databases', NULL, 'Access ASM Alloy Phase Diagram Database, ASM Handbooks (Vol. 1-23), and Materials Property Data. PIEAS library provides institutional access — see librarian for credentials.', 0),
            ('ASTM E8/E8M — Standard Test Methods for Tension Testing of Metallic Materials', 'https://www.astm.org/e0008_e0008m-22.html', 'Standards', NULL, 'The definitive standard for uniaxial tensile testing. Covers specimen geometry, strain rate, extensometer usage, and calculation of yield strength, UTS, elongation, and reduction in area.', 1),
            ('ASTM E112 — Standard Test Methods for Determining Average Grain Size', 'https://www.astm.org/e0112-13r21.html', 'Standards', NULL, 'Standard methods for measuring grain size in metals: comparison method, planimetric (Jeffries) method, and intercept method. Essential reference for metallography lab reports.', 1),
            ('Metallographic Etching Safety Guide — Nital, Picral & Kroll''s Reagent', 'https://www.buehler.com/solutions-guide/etching', 'Safety', NULL, 'Safety protocols for handling common metallographic etchants: 2% Nital (HNO3 + ethanol) for steels, Picral for pearlitic steels, and Kroll''s reagent (HF + HNO3) for titanium alloys. Includes PPE requirements and emergency procedures.', 1)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed resources");
    }

    let opp_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM opportunities")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if opp_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO opportunities (title, organization, opportunity_type, location, deadline, apply_url, description, posted_by_alumnus, is_active) VALUES
            ('Summer Research Internship — Nuclear Materials Division', 'Pakistan Institute of Nuclear Science & Technology (PINSTECH)', 'Internship', 'Islamabad, Pakistan', '2027-02-28', 'https://pinstech.org.pk/careers', 'PINSTECH offers 8-week summer internships for BS/MS students in their Nuclear Materials, Corrosion, and Materials Characterization divisions. Work on real research projects involving reactor materials, SEM/TEM analysis, and mechanical testing. Stipend provided.', NULL, 1),
            ('DAAD Research Fellowship for MS/PhD in Materials Science', 'German Academic Exchange Service (DAAD)', 'Scholarship', 'Germany', '2027-03-15', 'https://www.daad.de/en/studying-in-germany/scholarships/', 'Fully-funded fellowship for Pakistani students to pursue MS or PhD research at German universities. Covers tuition, monthly stipend (€934-1300), health insurance, and travel. Strong materials science departments at RWTH Aachen, TU Munich, and Max Planck Institutes.', 'Dr. Fatima Zahra (Batch 2018)', 1),
            ('Graduate Research Assistant — High-Entropy Alloys Lab', 'Massachusetts Institute of Technology (MIT DMSE)', 'Research', 'Cambridge, MA, USA', '2027-01-15', 'https://dmse.mit.edu/graduate/admissions', 'Prof. Li''s group at MIT DMSE is seeking motivated PhD students to work on the design and characterization of refractory high-entropy alloys for extreme temperature applications. Full tuition + stipend. GRE required.', 'Dr. Fatima Zahra (Batch 2018)', 1),
            ('4th International Conference on Advanced Materials & Manufacturing (ICAMM 2027)', 'Pakistan Engineering Council (PEC) & HEC', 'Competition', 'Lahore, Pakistan', '2027-04-30', 'https://example.com/icamm2027', 'Submit abstracts in: Advanced Steels, Additive Manufacturing, Biomaterials, Nuclear Materials, Computational Materials Science, Nanomaterials. Student registration discount available. Best paper awards in each category.', NULL, 1)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed opportunities");
    }

    let alm_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alumni")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if alm_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO alumni (name, graduating_batch, current_role, current_organization, industry_sector, location, quote_testimonial, linkedin_url) VALUES
            ('Dr. Fatima Zahra', '2014-2018', 'PhD Candidate (Final Year)', 'Massachusetts Institute of Technology (MIT DMSE)', 'Aerospace & Nuclear', 'Cambridge, MA, USA', 'PIEAS gave me the fundamentals and the confidence to compete globally. The rigorous curriculum at DMME, especially the hands-on metallography and characterization labs, prepared me for research at MIT. My advice: publish early, attend conferences, and never stop asking questions.', 'https://linkedin.com/in/example'),
            ('Engr. Ahmad Faraz', '2015-2019', 'Materials Engineer — Turbine Blade Coatings', 'Rolls-Royce plc', 'Aviation & Superalloys', 'Derby, United Kingdom', 'The Physical Metallurgy course at PIEAS was a game-changer. Understanding phase transformations at a deep level gave me an edge when I joined Rolls-Royce to work on thermal barrier coatings for jet engine turbine blades.', 'https://linkedin.com/in/example'),
            ('Dr. Sana Riaz', '2013-2017', 'Postdoctoral Researcher', 'Max Planck Institute für Eisenforschung', 'Steels & Hydrogen Metallurgy', 'Düsseldorf, Germany', 'From PIEAS to the Max Planck Institute — it was the passion for understanding ''why materials fail'' that drove my journey. The failure analysis training at DMME was invaluable. To current students: embrace computational tools early, they are the future.', 'https://linkedin.com/in/example'),
            ('Engr. Tariq Mehmood', '2016-2020', 'Senior Metallurgist — Quality Assurance', 'Heavy Mechanical Complex (HMC)', 'Defense & Heavy Manufacturing', 'Taxila, Pakistan', 'HMC was my dream workplace since the PMS industrial visit during my second year. The NDT and welding metallurgy knowledge from DMME directly applies to my daily work inspecting forgings and castings for defense applications.', 'https://linkedin.com/in/example')"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed alumni");
    }

    let prj_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if prj_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO projects (title, students, batch, supervisors, project_type, abstract, tags) VALUES
            ('Effect of Heat Treatment Parameters on Microstructure and Mechanical Properties of Dual-Phase (DP) Steel', 'Bilal Ahmed Khan, Mahnoor Iqbal', '2022-2026', 'Dr. Muhammad Asghar Hashmi', 'FYP', 'Investigated the influence of intercritical annealing temperature (740-820°C) and cooling rate on the volume fraction of martensite and resulting tensile properties in a 0.15C-1.5Mn steel. Achieved a DP steel with 980 MPa UTS and 15% elongation through optimized processing. Characterized using optical microscopy, SEM, XRD, and uniaxial tensile testing per ASTM E8.', 'Dual-Phase Steel, Heat Treatment, Mechanical Properties, Metallography'),
            ('Computational Prediction of Phase Stability in CoCrFeNiMn High-Entropy Alloy using CALPHAD', 'Hira Fatima, Usman Tariq', '2022-2026', 'Dr. Sarah Khan', 'FYP', 'Used Thermo-Calc with TCHEA5 database to predict equilibrium and Scheil solidification phase diagrams of the Cantor alloy (CoCrFeNiMn). Compared computational predictions with experimental XRD and DSC data from arc-melted samples. Identified σ-phase formation at intermediate temperatures (600-900°C) as a key embrittlement concern.', 'High-Entropy Alloys, CALPHAD, Thermo-Calc, Phase Diagrams'),
            ('Corrosion Behavior of Zircaloy-4 in Simulated PWR Primary Water Chemistry', 'Ahmed Raza, Zainab Malik', '2023-2027', 'Dr. Khalid Mahmood (PINSTECH)', 'FYP', 'Studied the aqueous corrosion and hydrogen pickup of Zircaloy-4 coupons in an autoclave simulating PWR conditions (360°C, 18.5 MPa, lithiated boric acid water). Characterized oxide layer thickness via SEM cross-sections and hydrogen content via vacuum hot extraction. Found that pre-oxidation treatment reduced hydrogen pickup by 30%.', 'Zircaloy-4, Nuclear Materials, Corrosion, PWR, Hydrogen Pickup'),
            ('Design and 3D Printing of Ti-6Al-4V Lattice Structures for Biomedical Implants', 'Hamza Ali Qureshi, Saad Ullah', '2022-2026', 'Dr. Amina Rashid', 'FYP', 'Designed BCC and TPMS (gyroid) lattice structures using nTopology and manufactured them via Selective Laser Melting (SLM) of Ti-6Al-4V powder. Evaluated compressive strength, elastic modulus, and biocompatibility. Gyroid structures with 60% porosity achieved elastic modulus matching cancellous bone (~3 GPa), reducing stress shielding.', 'Additive Manufacturing, Titanium, Biomedical, Lattice Structures, SLM')"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed projects");
    }

    let spon_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sponsors")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if spon_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO sponsors (name, website_url, tier, since_year, display_order) VALUES
            ('Pakistan Atomic Energy Commission (PAEC)', 'https://www.paec.gov.pk/', 'Platinum', '2020', 1),
            ('Heavy Mechanical Complex (HMC) Taxila', 'https://www.hmc.com.pk/', 'Gold', '2022', 2),
            ('Pakistan Steel Mills Corporation', 'https://www.paksteel.com.pk/', 'Silver', '2023', 3),
            ('ASM International — Pakistan Chapter', 'https://www.asminternational.org/', 'Partner', '2024', 4)"#,
        )
        .execute(pool)
        .await
        .expect("Failed to seed sponsors");
    }

    let faq_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM faqs")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if faq_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO faqs (question, answer, category, display_order) VALUES
            ('Who can join PMS?', 'PMS is primarily for students of the Department of Metallurgy & Materials Engineering (DMME) at PIEAS. However, we welcome students from related departments — Mechanical Engineering (DME), Physics & Applied Mathematics (DPAM), and Chemical Engineering (DCHE) — who share an interest in materials science.', 'Membership', 1),
            ('Is there a membership fee?', 'PMS membership is completely free. We believe in making technical knowledge and networking opportunities accessible to all PIEAS students without financial barriers.', 'Membership', 2),
            ('How do I apply to become a PMS council member?', 'Council elections are held annually, typically in September. Interested students must submit a candidacy form, deliver a short presentation to the general body outlining their vision for their desired portfolio, and win a majority vote. Watch for announcements at the start of each academic year.', 'Council', 3),
            ('Can non-DMME students attend PMS events?', 'Absolutely! All PMS technical seminars, workshops, and industrial visits are open to the entire PIEAS student community. Some hands-on workshops may have limited seats due to lab capacity, so early registration is recommended.', 'Events', 4),
            ('Does PMS provide certificates for events?', 'Yes, PMS issues verifiable participation certificates for all workshops, competitions, and seminars. Each certificate has a unique verification code that can be validated on our website under the Events page.', 'Certificates', 5),
            ('How can industry professionals or alumni collaborate with PMS?', 'We welcome collaborations! Industry partners can sponsor events, deliver guest lectures, offer internship positions, or co-organize competitions. Alumni can contribute by sharing career experiences, mentoring students, or posting opportunities. Contact us at pms@pieas.edu.pk or use the Contact form on our website.', 'Collaboration', 6)"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed FAQs");
    }

    let gal_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM gallery_images")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if gal_count.0 == 0 {
        sqlx::query(
            r#"INSERT INTO gallery_images (image_url, caption, event_name, year) VALUES
            ('https://images.unsplash.com/photo-1579783900882-c0d3dad7b119?auto=format&fit=crop&w=1000&q=80', 'Optical Micrograph of Dual-Phase Steel (Ferrite matrix with Martensite islands, 2% Nital Etch)', 'Annual Pakistan Materials Symposium (PMS ''26)', '2026'),
            ('https://images.unsplash.com/photo-1581092160607-ee22621dd758?auto=format&fit=crop&w=1000&q=80', 'SEM Micrograph of Ti-6Al-4V Selective Laser Melted (SLM) Gyroid Lattice', 'Hands-on Workshop: Quantitative Metallography with ImageJ & Python', '2026'),
            ('https://images.unsplash.com/photo-1581092335397-9583fe92d232?auto=format&fit=crop&w=1000&q=80', 'Heavy Mechanical Complex (HMC) Taxila — 3,000-ton Hydraulic Open-Die Forging Press', 'Industrial Visit: Heavy Mechanical Complex (HMC), Taxila', '2025'),
            ('https://images.unsplash.com/photo-1532094349884-543bc11b234d?auto=format&fit=crop&w=1000&q=80', 'High-Resolution TEM Micrograph showing γ′′ (Ni₃Nb) Precipitates in Inconel 718', 'Technical Seminar: Radiation Damage in Nuclear Reactor Cladding Alloys', '2026'),
            ('https://images.unsplash.com/photo-1507668077129-56e32842fceb?auto=format&fit=crop&w=1000&q=80', 'PMS Metallographic Etching Olympiad — Polishing & Etching Bench Session', 'Metallographic Etching Competition — PMS Olympiad 2026', '2026'),
            ('https://images.unsplash.com/photo-1517420704952-d9f39e95b43e?auto=format&fit=crop&w=1000&q=80', 'Optical Micrograph of Coarse Lamellar Pearlite in AISI 1080 Eutectoid Steel', 'Hands-on Workshop: Quantitative Metallography with ImageJ & Python', '2025'),
            ('https://images.unsplash.com/photo-1526374965328-7f61d4dc18c5?auto=format&fit=crop&w=1000&q=80', 'DMME Metallography Laboratory — Olympus BX53M Metallurgical Microscope Suite', 'Annual Pakistan Materials Symposium (PMS ''26)', '2025'),
            ('https://images.unsplash.com/photo-1516321318423-f06f85e504b3?auto=format&fit=crop&w=1000&q=80', 'Charpy V-Notch Impact Specimen Fracture Surface showing Cleavage River Patterns (SEM)', 'Technical Seminar: Radiation Damage in Nuclear Reactor Cladding Alloys', '2026')"#
        )
        .execute(pool)
        .await
        .expect("Failed to seed gallery images");
    }

    // Mark initial seed as complete in _db_metadata
    let _ = sqlx::query(
        "INSERT INTO _db_metadata (key, value) VALUES ('initial_seed_done', '1') ON CONFLICT(key) DO UPDATE SET value = '1'"
    )
    .execute(pool)
    .await;

    tracing::info!("Database seeding complete.");
}
