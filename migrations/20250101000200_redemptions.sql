CREATE TABLE IF NOT EXISTS redemptions (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  coupon_id   INTEGER NOT NULL,
  user_id     TEXT NOT NULL,
  redeemed_at INTEGER NOT NULL,
  UNIQUE(coupon_id)
);
CREATE INDEX IF NOT EXISTS idx_redemptions_user ON redemptions(user_id);
