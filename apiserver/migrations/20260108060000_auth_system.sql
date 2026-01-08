-- Thêm mật khẩu cho user (Idempotent)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'password_hash') THEN
        ALTER TABLE users ADD COLUMN password_hash TEXT;
    END IF;
END $$;

-- Bảng quản lý session (refresh tokens)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);

-- Cập nhật mật khẩu mặc định cho admin seeded (mật khẩu: 'admin123')
-- Hash: $2b$12$R9h/lIPzBZB9u34rn8qSze1FFE6zS/vU.YQe8V9M.Yq6.X9fP8P3C
UPDATE users SET password_hash = '$2b$12$R9h/lIPzBZB9u34rn8qSze1FFE6zS/vU.YQe8V9M.Yq6.X9fP8P3C' WHERE email = 'admin@leeattend.com';
