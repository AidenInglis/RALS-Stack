CREATE TABLE IF NOT EXISTS users (
  id            TEXT PRIMARY KEY,         -- uuid v4
  email         TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  is_admin      INTEGER NOT NULL DEFAULT 0, -- 0=false, 1=true
  created_at    INTEGER NOT NULL            -- unix seconds
);
