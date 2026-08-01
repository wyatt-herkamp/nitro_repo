-- When a token was last used to authenticate.
--
-- The profile page shows tokens with no way to tell which are still in use, which is what makes
-- cleaning up old ones guesswork. Written at most once an hour per token so an authenticated read
-- does not become a write against a hot row.
ALTER TABLE user_auth_tokens
    ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMP WITH TIME ZONE;
