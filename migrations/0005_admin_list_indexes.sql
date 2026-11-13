-- 0005_admin_list_indexes.sql
-- Indexes for the admin list views that sort or filter on these columns.
--
-- CHECK constraints for enum-like columns (status, priority, tier, category) are
-- deliberately not added here: SQLite cannot attach them to an existing table
-- without a full table rebuild, which is tracked separately.

CREATE INDEX IF NOT EXISTS idx_membership_created_at ON membership_applications(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contacts_created_at ON contact_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_alumni_graduating_batch ON alumni(graduating_batch DESC);
CREATE INDEX IF NOT EXISTS idx_projects_batch_title ON projects(batch DESC, title ASC);
CREATE INDEX IF NOT EXISTS idx_faqs_display_order ON faqs(display_order ASC);
CREATE INDEX IF NOT EXISTS idx_sponsors_display_order ON sponsors(display_order ASC, id DESC);
CREATE INDEX IF NOT EXISTS idx_events_date ON events(date);
