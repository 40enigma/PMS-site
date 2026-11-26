-- 0006_email_lowercasing_remaining.sql
-- Safely normalize emails in remaining tables without violating unique constraints

UPDATE event_registrations
SET email = lower(trim(email))
WHERE email != lower(trim(email))
  AND NOT EXISTS (
    SELECT 1 FROM event_registrations er2
    WHERE er2.event_id = event_registrations.event_id
      AND lower(trim(er2.email)) = lower(trim(event_registrations.email))
      AND er2.id != event_registrations.id
  );

UPDATE membership_applications
SET email = lower(trim(email))
WHERE email != lower(trim(email))
  AND lower(trim(email)) NOT IN (
    SELECT lower(trim(email)) FROM membership_applications GROUP BY lower(trim(email)) HAVING count(*) > 1
  );

UPDATE contact_messages
SET email = lower(trim(email))
WHERE email != lower(trim(email));

UPDATE team_members
SET email = lower(trim(email))
WHERE email IS NOT NULL AND email != lower(trim(email));
