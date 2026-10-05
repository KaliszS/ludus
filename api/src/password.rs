//! Password hashing and the guard rails around it.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use tokio::sync::Semaphore;

/// OWASP's 7 MiB / 5-pass Argon2id: the same work as their 19 MiB / 2-pass default
/// at under half the memory per attempt, which matters on a small machine.
const MEMORY_KIB: u32 = 7 * 1024;
const PASSES: u32 = 5;
/// Each hash borrows MEMORY_KIB for its duration. Capping how many run at once
/// keeps a flood of sign-ins from turning into a flood of memory.
const CONCURRENT: usize = 4;

pub const MIN_LENGTH: usize = 10;
/// Long enough for any passphrase, short enough that nobody can make us hash a novel.
pub const MAX_LENGTH: usize = 256;

pub struct Hasher {
    permits: Semaphore,
    /// Checked against when an email has no account, so a miss costs as long as a
    /// wrong password and timing cannot tell the two apart.
    decoy: String,
}

fn argon2() -> Argon2<'static> {
    let params = Params::new(MEMORY_KIB, PASSES, 1, None).expect("constant parameters are valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

impl Hasher {
    pub fn new() -> anyhow::Result<Self> {
        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).context("os random generator")?;
        Ok(Self {
            permits: Semaphore::new(CONCURRENT),
            decoy: hash_now(&seed)?,
        })
    }

    pub async fn hash(&self, password: String) -> anyhow::Result<String> {
        let _permit = self.permits.acquire().await?;
        tokio::task::spawn_blocking(move || hash_now(password.as_bytes())).await?
    }

    /// `stored` is None when there is no such account; the work is done regardless.
    pub async fn verify(&self, password: String, stored: Option<String>) -> anyhow::Result<bool> {
        let known = stored.is_some();
        let stored = stored.unwrap_or_else(|| self.decoy.clone());
        let _permit = self.permits.acquire().await?;
        let matches = tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&stored).context("stored hash")?;
            anyhow::Ok(
                argon2()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok(),
            )
        })
        .await??;
        Ok(known && matches)
    }
}

fn hash_now(password: &[u8]) -> anyhow::Result<String> {
    Ok(argon2()
        .hash_password(password)
        .map_err(|err| anyhow::anyhow!("argon2: {err}"))?
        .to_string())
}

const ATTEMPTS: u32 = 10;
const WINDOW: Duration = Duration::from_secs(15 * 60);
/// Past this many tracked emails, expired windows are swept before adding another.
const TRACKED: usize = 10_000;

/// Failed sign-ins per email, kept in memory: one daemon, one map. A restart
/// forgets them, which only ever errs toward letting someone try again.
///
/// Counting per email rather than per address means guessing at one account is
/// capped however many machines do it. The price is that someone can lock an
/// account for a window by failing on purpose; it reopens on its own.
#[derive(Default)]
pub struct Attempts {
    windows: Mutex<HashMap<String, (Instant, u32)>>,
}

impl Attempts {
    pub fn blocked(&self, email: &str) -> bool {
        let windows = self.windows.lock().expect("attempts lock");
        windows
            .get(email)
            .is_some_and(|(started, failures)| started.elapsed() < WINDOW && *failures >= ATTEMPTS)
    }

    pub fn failed(&self, email: &str) {
        let mut windows = self.windows.lock().expect("attempts lock");
        if windows.len() >= TRACKED {
            windows.retain(|_, (started, _)| started.elapsed() < WINDOW);
        }
        let entry = windows
            .entry(email.to_owned())
            .or_insert((Instant::now(), 0));
        if entry.0.elapsed() >= WINDOW {
            *entry = (Instant::now(), 0);
        }
        entry.1 += 1;
    }

    pub fn succeeded(&self, email: &str) {
        self.windows.lock().expect("attempts lock").remove(email);
    }
}

/// One spelling per address, so `Jan@X.com` and `jan@x.com` are the same login.
pub fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    // No dot required in the domain: a self-hosted instance may well live on `localhost`.
    let valid =
        !local.is_empty() && !domain.is_empty() && !domain.contains('@') && email.len() <= 254;
    valid.then_some(email)
}

/// NIST 800-63B: a length floor and nothing else. Composition rules only breed `Haslo123!`.
pub fn acceptable(password: &str) -> bool {
    (MIN_LENGTH..=MAX_LENGTH).contains(&password.chars().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verifies_its_own_hash_and_nothing_else() {
        let hasher = Hasher::new().unwrap();
        let stored = hasher.hash("correct horse battery".into()).await.unwrap();
        assert!(stored.starts_with("$argon2id$v=19$m=7168,t=5,p=1$"));
        assert!(
            hasher
                .verify("correct horse battery".into(), Some(stored.clone()))
                .await
                .unwrap()
        );
        assert!(
            !hasher
                .verify("correct horse batterY".into(), Some(stored))
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn an_unknown_account_never_verifies() {
        let hasher = Hasher::new().unwrap();
        assert!(!hasher.verify("anything at all".into(), None).await.unwrap());
    }

    #[tokio::test]
    async fn the_same_password_hashes_differently() {
        let hasher = Hasher::new().unwrap();
        let a = hasher.hash("same password".into()).await.unwrap();
        let b = hasher.hash("same password".into()).await.unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn attempts_block_after_the_limit_and_clear_on_success() {
        let attempts = Attempts::default();
        for _ in 0..ATTEMPTS - 1 {
            attempts.failed("jan@x.com");
        }
        assert!(!attempts.blocked("jan@x.com"));
        attempts.failed("jan@x.com");
        assert!(attempts.blocked("jan@x.com"));
        assert!(!attempts.blocked("ola@x.com"));
        attempts.succeeded("jan@x.com");
        assert!(!attempts.blocked("jan@x.com"));
    }

    #[test]
    fn emails_are_normalised_or_rejected() {
        assert_eq!(
            normalize_email("  Jan@Example.COM ").as_deref(),
            Some("jan@example.com")
        );
        assert_eq!(
            normalize_email("dev@localhost").as_deref(),
            Some("dev@localhost")
        );
        for bad in ["", "jan", "@example.com", "jan@", "a@b@c.com"] {
            assert_eq!(normalize_email(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn length_is_counted_in_characters() {
        assert!(!acceptable("krótkie"));
        assert!(acceptable("źdźbłoźdźbło"));
        assert!(!acceptable(&"a".repeat(MAX_LENGTH + 1)));
    }
}
