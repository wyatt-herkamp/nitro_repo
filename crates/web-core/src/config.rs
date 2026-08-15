//! Configuration types shared between the server and the code it hands a request to.
//!
//! Only the pieces something below the application needs: the security settings a repository reads
//! to decide whether to trust `X-Forwarded-Host`, the password rules the API reports, and the two
//! odds and ends the session store wants. The rest of the configuration stays in the server, which
//! is the only thing that parses it.

use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};
use strum::EnumIs;
use utoipa::ToSchema;

/// Whether this build is a debug build, which relaxes a few things that would be unsafe in
/// production — notably the `Secure` flag on the session cookie.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, EnumIs, ToSchema)]
pub enum Mode {
    Debug,
    Release,
}

impl Default for Mode {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return Mode::Debug;
        #[cfg(not(debug_assertions))]
        return Mode::Release;
    }
}

pub fn get_current_directory() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::new())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecuritySettings {
    pub allow_basic_without_tokens: bool,
    pub password_rules: Option<PasswordRules>,
    /// Resolve a request's host from `X-Forwarded-Host` before `Host`.
    ///
    /// Off by default. Any client can send that header, so trusting it without a reverse proxy in
    /// front that overwrites it would let a caller pick which repository's custom domain it lands
    /// on. Turn it on only if your proxy sets it and strips whatever the client sent.
    ///
    /// `#[serde(default)]` on the field rather than the struct: `SecuritySettings` has no
    /// container-level default, so without it every existing config with a `[security]` section
    /// would stop parsing.
    #[serde(default)]
    pub trust_forwarded_host: bool,
}
impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            allow_basic_without_tokens: false,
            password_rules: Some(PasswordRules::default()),
            trust_forwarded_host: false,
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct PasswordRules {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_symbol: bool,
}
impl PasswordRules {
    pub fn validate(&self, password: &str) -> bool {
        if password.len() < self.min_length {
            return false;
        }
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return false;
        }
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return false;
        }
        if self.require_number && !password.chars().any(|c| c.is_numeric()) {
            return false;
        }
        if self.require_symbol && !password.chars().any(|c| c.is_ascii_punctuation()) {
            return false;
        }
        true
    }
}
impl Default for PasswordRules {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_symbol: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}
