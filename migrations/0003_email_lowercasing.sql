-- 0003_email_lowercasing.sql
-- Safely lowercase admin and subscriber emails skipping any collision pairs

UPDATE admin_users
SET email = lower(email)
WHERE email != lower(email)
  AND lower(email) NOT IN (
    SELECT lower(email) FROM admin_users GROUP BY lower(email) HAVING count(*) > 1
  );

UPDATE newsletter_subscribers
SET email = lower(email)
WHERE email != lower(email)
  AND lower(email) NOT IN (
    SELECT lower(email) FROM newsletter_subscribers GROUP BY lower(email) HAVING count(*) > 1
  );

