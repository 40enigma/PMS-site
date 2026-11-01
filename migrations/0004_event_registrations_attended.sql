-- 0004_event_registrations_attended.sql
-- Track whether a registered student attended the event (required for certificate verification)

ALTER TABLE event_registrations ADD COLUMN attended BOOLEAN NOT NULL DEFAULT 0;

