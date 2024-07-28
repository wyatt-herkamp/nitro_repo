-- Add up migration script here
CREATE COLLATION ignoreCase (
  provider = 'icu',
  locale = '@colStrength=secondary',
  deterministic = false
);