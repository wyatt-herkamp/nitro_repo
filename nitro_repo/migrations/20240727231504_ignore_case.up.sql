-- Add up migration script here
CREATE COLLATION IF NOT EXISTS ignoreCase (
  provider = 'icu',
  locale = '@colStrength=secondary',
  deterministic = false
);