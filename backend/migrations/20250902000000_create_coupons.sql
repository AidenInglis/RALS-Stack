CREATE TABLE IF NOT EXISTS coupons (
  id          TEXT PRIMARY KEY,           -- uuid v4
  code        TEXT UNIQUE NOT NULL,       -- human-facing code
  description TEXT NOT NULL DEFAULT '',
  service     TEXT NOT NULL DEFAULT '',
  expires_at  INTEGER NOT NULL,           -- unix seconds
  owner_id    TEXT,                       -- optional user id (NULL = unassigned)
  created_at  INTEGER NOT NULL,           -- unix seconds
  FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_coupons_code ON coupons(code);
CREATE INDEX IF NOT EXISTS idx_coupons_owner ON coupons(owner_id);
CREATE INDEX IF NOT EXISTS idx_coupons_expires ON coupons(expires_at);