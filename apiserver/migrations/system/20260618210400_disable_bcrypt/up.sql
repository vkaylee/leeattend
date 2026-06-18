-- Disable legacy bcrypt hashes
UPDATE users
SET password_hash = '*DISABLED_BCRYPT*'
WHERE password_hash LIKE '$2a$%'
   OR password_hash LIKE '$2b$%'
   OR password_hash LIKE '$2y$%';
