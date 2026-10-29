use anyhow::Result;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use totp_rs::{Algorithm, Builder, Secret, Totp};

/// Resolve effective Unix timestamp from optional explicit timestamp and relative skew in seconds
pub fn resolve_effective_time(at: Option<u64>, skew: Option<i64>) -> Result<Option<u64>> {
    if at.is_none() && skew.is_none() {
        return Ok(None);
    }
    let base = match at {
        Some(ts) => ts,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("System clock error: {e}"))?
            .as_secs(),
    };
    let effective = match skew {
        Some(offset) => base
            .checked_add_signed(offset)
            .ok_or_else(|| anyhow::anyhow!("Resulting timestamp cannot be negative"))?,
        None => base,
    };
    Ok(Some(effective))
}

/// Compute RFC 4226 HOTP code
pub fn generate_hotp(
    secret: &[u8],
    counter: u64,
    digits: u8,
    algorithm: Algorithm,
) -> Result<String> {
    if !(6..=8).contains(&digits) {
        anyhow::bail!("HOTP digits must be 6, 7, or 8");
    }

    let counter_bytes = counter.to_be_bytes();
    let hash: Vec<u8> = match algorithm {
        Algorithm::SHA1 | Algorithm::Steam => {
            let mut mac = <Hmac<Sha1>>::new_from_slice(secret)
                .map_err(|e| anyhow::anyhow!("HMAC-SHA1 init error: {e}"))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::SHA256 => {
            let mut mac = <Hmac<Sha256>>::new_from_slice(secret)
                .map_err(|e| anyhow::anyhow!("HMAC-SHA256 init error: {e}"))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::SHA512 => {
            let mut mac = <Hmac<Sha512>>::new_from_slice(secret)
                .map_err(|e| anyhow::anyhow!("HMAC-SHA512 init error: {e}"))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        _ => {
            let mut mac = <Hmac<Sha1>>::new_from_slice(secret)
                .map_err(|e| anyhow::anyhow!("HMAC-SHA1 init error: {e}"))?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
    };

    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    if offset + 4 > hash.len() {
        anyhow::bail!("Dynamic truncation offset out of bounds");
    }

    let binary = ((hash[offset] & 0x7f) as u32) << 24
        | ((hash[offset + 1] as u32) & 0xff) << 16
        | ((hash[offset + 2] as u32) & 0xff) << 8
        | ((hash[offset + 3] as u32) & 0xff);

    let modulo = 10u32.pow(digits as u32);
    let code = binary % modulo;
    Ok(format!("{:0width$}", code, width = digits as usize))
}

#[derive(Debug, Clone)]
pub struct TfaHotp {
    pub account: String,
    pub secret: Secret,
    pub counter: u64,
    pub digits: u8,
    pub algorithm: Algorithm,
    pub issuer: Option<String>,
}

impl TfaHotp {
    pub fn from_url(url_str: &str) -> Result<Self> {
        let u =
            url::Url::parse(url_str).map_err(|e| anyhow::anyhow!("Invalid OTPAuth URL: {e}"))?;
        if u.scheme() != "otpauth" {
            anyhow::bail!("Invalid scheme in URL: expected otpauth");
        }
        if u.host_str() != Some("hotp") {
            anyhow::bail!("Expected host 'hotp' in OTPAuth URL");
        }

        let mut secret_opt = None;
        let mut counter = 0u64;
        let mut digits = 6u8;
        let mut algorithm = Algorithm::SHA1;
        let mut issuer_query = None;

        for (k, v) in u.query_pairs() {
            match k.as_ref() {
                "secret" => secret_opt = Some(v.to_string()),
                "counter" => {
                    counter = v
                        .parse::<u64>()
                        .map_err(|e| anyhow::anyhow!("Invalid counter in URL: {e}"))?;
                }
                "digits" => {
                    digits = v
                        .parse::<u8>()
                        .map_err(|e| anyhow::anyhow!("Invalid digits in URL: {e}"))?;
                }
                "algorithm" => {
                    algorithm = match v.to_uppercase().as_str() {
                        "SHA1" => Algorithm::SHA1,
                        "SHA256" => Algorithm::SHA256,
                        "SHA512" => Algorithm::SHA512,
                        "STEAM" => Algorithm::Steam,
                        _ => anyhow::bail!("Unsupported algorithm: {v}"),
                    };
                }
                "issuer" => issuer_query = Some(v.to_string()),
                _ => {}
            }
        }

        let raw_secret =
            secret_opt.ok_or_else(|| anyhow::anyhow!("Missing secret parameter in OTPAuth URL"))?;
        let clean_secret: String = raw_secret
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
            .collect::<String>()
            .to_uppercase();
        let secret = Secret::try_from_base32(clean_secret)
            .map_err(|e| anyhow::anyhow!("Invalid base32 secret: {e}"))?;

        let path = u.path().trim_start_matches('/');
        let (label_issuer, account) = if let Some((iss, acc)) = path.split_once(':') {
            (Some(iss.to_string()), acc.to_string())
        } else {
            (None, path.to_string())
        };

        let final_issuer = issuer_query.or(label_issuer);

        Ok(Self {
            account: if account.is_empty() {
                "account".to_string()
            } else {
                account
            },
            secret,
            counter,
            digits,
            algorithm,
            issuer: final_issuer,
        })
    }

    pub fn to_url(&self) -> Result<String> {
        let label = match &self.issuer {
            Some(iss) => format!("{}:{}", iss, self.account),
            None => self.account.clone(),
        };
        let mut u = url::Url::parse(&format!("otpauth://hotp/{label}"))
            .map_err(|e| anyhow::anyhow!("Failed constructing OTPAuth URL: {e}"))?;

        let alg_str = match self.algorithm {
            Algorithm::SHA1 => "SHA1",
            Algorithm::SHA256 => "SHA256",
            Algorithm::SHA512 => "SHA512",
            Algorithm::Steam => "STEAM",
            _ => "SHA1",
        };

        u.query_pairs_mut()
            .append_pair("secret", &self.secret.to_base32())
            .append_pair("counter", &self.counter.to_string())
            .append_pair("digits", &self.digits.to_string())
            .append_pair("algorithm", alg_str);

        if let Some(ref iss) = self.issuer {
            u.query_pairs_mut().append_pair("issuer", iss);
        }

        Ok(u.to_string())
    }
}

pub enum TfaEngine {
    Totp(Totp),
    Hotp(TfaHotp),
}

pub struct TfaTotp {
    inner: TfaEngine,
    prefix: Option<String>,
}

fn extract_prefix_from_url(url_str: &str) -> Option<String> {
    let u = url::Url::parse(url_str).ok()?;
    for (k, v) in u.query_pairs() {
        if k == "prefix" && !v.is_empty() {
            return Some(v.into_owned());
        }
    }
    None
}

fn normalize_otpauth_url(url_str: &str) -> String {
    let Ok(mut u) = url::Url::parse(url_str) else {
        return url_str.to_string();
    };
    let mut is_steam = false;
    let pairs: Vec<(String, String)> = u
        .query_pairs()
        .map(|(k, v)| {
            if k == "secret" {
                let clean: String = v
                    .chars()
                    .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
                    .collect::<String>()
                    .to_uppercase();
                (k.into_owned(), clean)
            } else {
                if k == "encoder" && v.eq_ignore_ascii_case("steam") {
                    is_steam = true;
                }
                (k.into_owned(), v.into_owned())
            }
        })
        .collect();
    if is_steam {
        let _ = u.set_host(Some("steam"));
    }
    if !pairs.is_empty() {
        u.query_pairs_mut().clear().extend_pairs(pairs);
    }
    u.to_string()
}

impl TfaTotp {
    /// Create TOTP or HOTP instance with defaults from raw base32 secret or otpauth:// URI
    pub fn from_input(account: &str, input: &str) -> Result<Self> {
        Self::build(account, input, None, None, None, None)
    }

    /// Build TOTP or HOTP instance with optional custom digits, period, algorithm, and counter
    pub fn build(
        account: &str,
        input: &str,
        digits: Option<u8>,
        period: Option<u64>,
        algorithm: Option<Algorithm>,
        counter: Option<u64>,
    ) -> Result<Self> {
        let input = input.trim();
        let prefix = if input.starts_with("otpauth://") {
            extract_prefix_from_url(input)
        } else {
            None
        };

        // 1. If counter is explicitly provided or URL is otpauth://hotp/, build HOTP
        if counter.is_some() || input.starts_with("otpauth://hotp/") {
            let mut hotp = if input.starts_with("otpauth://hotp/") {
                let mut h = TfaHotp::from_url(input)?;
                if !account.is_empty() {
                    h.account = account.to_string();
                }
                h
            } else {
                let clean_secret: String = input
                    .chars()
                    .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
                    .collect::<String>()
                    .to_uppercase();

                let secret = Secret::try_from_base32(clean_secret)
                    .map_err(|e| anyhow::anyhow!("Invalid base32 secret: {e}"))?;

                TfaHotp {
                    account: account.to_string(),
                    secret,
                    counter: 0,
                    digits: 6,
                    algorithm: Algorithm::SHA1,
                    issuer: Some("tfa".to_string()),
                }
            };

            if let Some(c) = counter {
                hotp.counter = c;
            }
            if let Some(d) = digits {
                hotp.digits = d;
            }
            if let Some(a) = algorithm {
                hotp.algorithm = a;
            }

            return Ok(Self {
                inner: TfaEngine::Hotp(hotp),
                prefix,
            });
        }

        // 2. Otherwise build TOTP (supports standard RFC 6238 and Steam Guard)
        let base_totp = if input.starts_with("otpauth://") {
            let normalized = normalize_otpauth_url(input);
            Totp::from_url_unchecked(&normalized)
                .map_err(|e| anyhow::anyhow!("Invalid OTPAuth URL: {e}"))?
        } else {
            let clean_secret: String = input
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
                .collect::<String>()
                .to_uppercase();

            let secret = Secret::try_from_base32(clean_secret)
                .map_err(|e| anyhow::anyhow!("Invalid base32 secret: {e}"))?;

            Builder::new()
                .with_algorithm(Algorithm::SHA1)
                .with_digits(6)
                .with_step_duration(30)
                .with_secret(secret)
                .with_account_name(account)
                .with_issuer(Some("tfa"))
                .build_noncompliant()
        };

        let final_alg = algorithm.unwrap_or_else(|| base_totp.algorithm());
        let final_digits = digits.unwrap_or_else(|| {
            if final_alg == Algorithm::Steam {
                5
            } else {
                base_totp.digits()
            }
        });

        if final_alg == Algorithm::Steam && final_digits != 5 {
            anyhow::bail!("Steam algorithm requires exactly 5 digits");
        }

        let final_step = period.unwrap_or_else(|| base_totp.step());
        let final_issuer = if final_alg == Algorithm::Steam
            && (base_totp.issuer().is_none() || base_totp.issuer() == Some("tfa"))
        {
            "Steam"
        } else {
            base_totp.issuer().unwrap_or("tfa")
        };
        let final_account = if base_totp.account_name().is_empty() {
            account
        } else {
            base_totp.account_name()
        };

        let totp = Builder::new()
            .with_algorithm(final_alg)
            .with_digits(final_digits)
            .with_step_duration(final_step)
            .with_secret(base_totp.secret().clone())
            .with_account_name(final_account)
            .with_issuer(Some(final_issuer))
            .build_noncompliant();

        Ok(Self {
            inner: TfaEngine::Totp(totp),
            prefix,
        })
    }

    /// Optional static prefix prepended to generated OTP tokens
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Set or clear token prefix
    pub fn set_prefix(&mut self, prefix: Option<String>) {
        self.prefix = prefix.filter(|p| !p.is_empty());
    }

    fn apply_prefix(&self, code: String) -> String {
        match &self.prefix {
            Some(pfx) if !pfx.is_empty() => format!("{pfx}{code}"),
            _ => code,
        }
    }

    /// True if account is counter-based (HOTP)
    pub fn is_hotp(&self) -> bool {
        matches!(self.inner, TfaEngine::Hotp(_))
    }

    /// Account type description ("HOTP", "Steam", or "TOTP")
    pub fn account_type(&self) -> &str {
        match &self.inner {
            TfaEngine::Hotp(_) => "HOTP",
            TfaEngine::Totp(totp) => {
                if totp.algorithm() == Algorithm::Steam {
                    "Steam"
                } else {
                    "TOTP"
                }
            }
        }
    }

    /// Current counter if account is HOTP
    pub fn counter(&self) -> Option<u64> {
        match &self.inner {
            TfaEngine::Hotp(h) => Some(h.counter),
            TfaEngine::Totp(_) => None,
        }
    }

    /// Update stored counter for HOTP account
    pub fn set_counter(&mut self, counter: u64) {
        if let TfaEngine::Hotp(ref mut h) = self.inner {
            h.counter = counter;
        }
    }

    /// Generate code for a specific counter value (HOTP only)
    pub fn generate_at_counter(&self, counter: u64) -> Result<String> {
        match &self.inner {
            TfaEngine::Hotp(h) => {
                let code = generate_hotp(h.secret.as_bytes(), counter, h.digits, h.algorithm)?;
                Ok(self.apply_prefix(code))
            }
            TfaEngine::Totp(_) => {
                anyhow::bail!("Counter generation only supported for HOTP accounts")
            }
        }
    }

    /// Generate current TOTP code or current-counter HOTP code
    pub fn generate_current(&self) -> String {
        let code = match &self.inner {
            TfaEngine::Totp(totp) => totp.generate_current().to_string(),
            TfaEngine::Hotp(h) => {
                generate_hotp(h.secret.as_bytes(), h.counter, h.digits, h.algorithm)
                    .unwrap_or_default()
            }
        };
        self.apply_prefix(code)
    }

    /// Generate TOTP code for the next time step
    pub fn generate_next(&self) -> Result<String> {
        match &self.inner {
            TfaEngine::Totp(totp) => {
                Ok(self.apply_prefix(totp.generate(totp.next_step_current()).to_string()))
            }
            TfaEngine::Hotp(_) => {
                anyhow::bail!(
                    "Next-token preview is not supported for counter-based (HOTP) accounts"
                )
            }
        }
    }

    /// Generate TOTP code for a specific timestamp (TOTP only)
    pub fn generate_at_time(&self, time: u64) -> Result<String> {
        match &self.inner {
            TfaEngine::Totp(totp) => Ok(self.apply_prefix(totp.generate(time).to_string())),
            TfaEngine::Hotp(_) => {
                anyhow::bail!("Time-based generation only supported for TOTP accounts")
            }
        }
    }

    /// Generate TOTP code for the next time step from a specific timestamp (TOTP only)
    pub fn generate_next_at_time(&self, time: u64) -> Result<String> {
        match &self.inner {
            TfaEngine::Totp(totp) => {
                Ok(self.apply_prefix(totp.generate(totp.next_step(time)).to_string()))
            }
            TfaEngine::Hotp(_) => {
                anyhow::bail!(
                    "Next-token preview is not supported for counter-based (HOTP) accounts"
                )
            }
        }
    }

    /// Seconds remaining until current code expires (0 for HOTP)
    pub fn ttl(&self) -> u64 {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.ttl(),
            TfaEngine::Hotp(_) => 0,
        }
    }

    /// Seconds remaining until code expires from a specific timestamp (TOTP only)
    pub fn ttl_at_time(&self, time: u64) -> u64 {
        match &self.inner {
            TfaEngine::Totp(totp) => {
                let step = totp.step();
                if step == 0 {
                    0
                } else {
                    step - (time % step)
                }
            }
            TfaEngine::Hotp(_) => 0,
        }
    }

    /// Number of digits in generated code
    pub fn digits(&self) -> u8 {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.digits(),
            TfaEngine::Hotp(h) => h.digits,
        }
    }

    /// Time step period in seconds (0 for HOTP)
    pub fn step(&self) -> u64 {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.step(),
            TfaEngine::Hotp(_) => 0,
        }
    }

    /// Hash algorithm
    pub fn algorithm(&self) -> Algorithm {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.algorithm(),
            TfaEngine::Hotp(h) => h.algorithm,
        }
    }

    /// Base32 representation of the secret
    pub fn secret_base32(&self) -> String {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.secret().to_base32(),
            TfaEngine::Hotp(h) => h.secret.to_base32(),
        }
    }

    /// Raw secret bytes
    pub fn secret_bytes(&self) -> Vec<u8> {
        match &self.inner {
            TfaEngine::Totp(totp) => totp.secret().as_bytes().to_vec(),
            TfaEngine::Hotp(h) => h.secret.as_bytes().to_vec(),
        }
    }

    /// Canonical OTPAuth URL representation
    pub fn to_url(&self) -> Result<String> {
        let base = match &self.inner {
            TfaEngine::Totp(totp) => totp
                .to_url()
                .map_err(|e| anyhow::anyhow!("Failed exporting OTPAuth URL: {e}"))?,
            TfaEngine::Hotp(h) => h.to_url()?,
        };
        if let Some(ref pfx) = self.prefix {
            if !pfx.is_empty() {
                if let Ok(mut u) = url::Url::parse(&base) {
                    u.query_pairs_mut().append_pair("prefix", pfx);
                    return Ok(u.to_string());
                }
            }
        }
        Ok(base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_from_valid_base32() {
        // Standard RFC 6238 test secret "12345678901234567890" in base32
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let tfa = TfaTotp::from_input("test", secret);
        assert!(tfa.is_ok());

        let totp = tfa.unwrap();
        let code_str = totp.generate_current();
        assert_eq!(code_str.len(), 6);
        assert!(code_str.chars().all(|c| c.is_ascii_digit()));

        let ttl = totp.ttl();
        assert!(ttl <= 30);
        assert_eq!(totp.account_type(), "TOTP");
        assert_eq!(totp.secret_base32(), secret);
    }

    #[test]
    fn test_totp_from_formatted_base32() {
        let secret = "gezd-gnbv gy3t-qojq gez-dgnbv gy3t-qojq";
        let tfa = TfaTotp::from_input("test", secret).unwrap();
        let code = tfa.generate_current();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_totp_from_otpauth_url() {
        let url = "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example";
        let tfa = TfaTotp::from_input("alice", url).unwrap();
        let code = tfa.generate_current();
        assert_eq!(code.len(), 6);
        assert_eq!(tfa.account_type(), "TOTP");
    }

    #[test]
    fn test_totp_invalid_secret_fails() {
        let invalid = "not-a-valid-base32-secret-999!!!";
        let tfa = TfaTotp::from_input("test", invalid);
        assert!(tfa.is_err());
    }

    #[test]
    fn test_totp_custom_digits_period_algorithm() {
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let tfa = TfaTotp::build(
            "custom_acc",
            secret,
            Some(8),
            Some(60),
            Some(Algorithm::SHA256),
            None,
        )
        .unwrap();

        assert_eq!(tfa.digits(), 8);
        assert_eq!(tfa.step(), 60);
        assert_eq!(tfa.algorithm(), Algorithm::SHA256);

        let code = tfa.generate_current();
        assert_eq!(code.len(), 8);
        assert!(tfa.ttl() <= 60);

        let url = tfa.to_url().unwrap();
        assert!(url.contains("digits=8"));
        assert!(url.contains("period=60"));
        assert!(url.contains("algorithm=SHA256"));
    }

    #[test]
    fn test_steam_guard_token_generation() {
        // Steam secret in base32
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let steam = TfaTotp::build(
            "steam_user",
            secret,
            None,
            None,
            Some(Algorithm::Steam),
            None,
        )
        .unwrap();

        assert_eq!(steam.digits(), 5);
        assert_eq!(steam.step(), 30);
        assert_eq!(steam.algorithm(), Algorithm::Steam);
        assert_eq!(steam.account_type(), "Steam");

        let code = steam.generate_current();
        assert_eq!(code.len(), 5);
        // Steam alphabet: 23456789BCDFGHJKMNPQRTVWXY
        const STEAM_ALPHABET: &[u8] = b"23456789BCDFGHJKMNPQRTVWXY";
        for byte in code.bytes() {
            assert!(
                STEAM_ALPHABET.contains(&byte),
                "Unexpected Steam char: {}",
                byte as char
            );
        }

        let url = steam.to_url().unwrap();
        assert!(url.starts_with("otpauth://steam/") || url.contains("algorithm=STEAM"));

        // Load from exported steam url
        let loaded = TfaTotp::from_input("steam_user", &url).unwrap();
        assert_eq!(loaded.algorithm(), Algorithm::Steam);
        assert_eq!(loaded.digits(), 5);
    }

    #[test]
    fn test_rfc4226_hotp_appendix_d_vectors() {
        // RFC 4226 Appendix D: secret is ASCII "12345678901234567890"
        let secret_bytes = b"12345678901234567890";
        let expected_codes = [
            "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583",
            "399871", "520489",
        ];

        for (counter, expected) in expected_codes.iter().enumerate() {
            let code = generate_hotp(secret_bytes, counter as u64, 6, Algorithm::SHA1).unwrap();
            assert_eq!(&code, expected, "Mismatch at counter {}", counter);
        }
    }

    #[test]
    fn test_hotp_build_and_counter_increment() {
        // Base32 for "12345678901234567890" is "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        let secret_b32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let mut hotp = TfaTotp::build(
            "alice",
            secret_b32,
            Some(6),
            None,
            Some(Algorithm::SHA1),
            Some(0),
        )
        .unwrap();

        assert!(hotp.is_hotp());
        assert_eq!(hotp.counter(), Some(0));
        assert_eq!(hotp.account_type(), "HOTP");
        assert_eq!(hotp.ttl(), 0);

        // Counter 0 code: 755224
        assert_eq!(hotp.generate_current(), "755224");

        // Increment counter
        hotp.set_counter(1);
        assert_eq!(hotp.generate_current(), "287082");

        // Counter 5 via generate_at_counter
        assert_eq!(hotp.generate_at_counter(5).unwrap(), "254676");

        // URL export and reload
        let url = hotp.to_url().unwrap();
        assert!(url.starts_with("otpauth://hotp/"));
        assert!(url.contains("counter=1"));

        let loaded = TfaTotp::from_input("alice", &url).unwrap();
        assert!(loaded.is_hotp());
        assert_eq!(loaded.counter(), Some(1));
        assert_eq!(loaded.generate_current(), "287082");
    }

    #[test]
    fn test_totp_and_hotp_edge_cases() {
        let secret = b"12345678901234567890";
        // Invalid digits
        assert!(generate_hotp(secret, 0, 5, Algorithm::SHA1).is_err());
        assert!(generate_hotp(secret, 0, 9, Algorithm::SHA1).is_err());

        // SHA256 and SHA512 HOTP
        let c256 = generate_hotp(secret, 0, 6, Algorithm::SHA256).unwrap();
        assert_eq!(c256.len(), 6);
        let c512 = generate_hotp(secret, 0, 6, Algorithm::SHA512).unwrap();
        assert_eq!(c512.len(), 6);

        // URL parsing errors & algorithms
        assert!(TfaHotp::from_url("https://example.com").is_err());
        assert!(TfaHotp::from_url("otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP").is_err());

        let url_sha256 = "otpauth://hotp/alice?secret=JBSWY3DPEHPK3PXP&counter=0&algorithm=SHA256";
        let h256 = TfaHotp::from_url(url_sha256).unwrap();
        assert_eq!(h256.algorithm, Algorithm::SHA256);
        let exp256 = h256.to_url().unwrap();
        assert!(exp256.contains("algorithm=SHA256"));

        let url_sha512 = "otpauth://hotp/alice?secret=JBSWY3DPEHPK3PXP&counter=0&algorithm=SHA512";
        let h512 = TfaHotp::from_url(url_sha512).unwrap();
        assert_eq!(h512.algorithm, Algorithm::SHA512);
        let exp512 = h512.to_url().unwrap();
        assert!(exp512.contains("algorithm=SHA512"));

        let url_steam = "otpauth://hotp/alice?secret=JBSWY3DPEHPK3PXP&counter=0&algorithm=STEAM";
        let hsteam = TfaHotp::from_url(url_steam).unwrap();
        assert_eq!(hsteam.algorithm, Algorithm::Steam);
        let expsteam = hsteam.to_url().unwrap();
        assert!(expsteam.contains("algorithm=STEAM"));

        // HOTP step is 0
        let hotp_totp = TfaTotp::from_input("alice", url_sha256).unwrap();
        assert_eq!(hotp_totp.step(), 0);

        // Steam with 6 digits fails
        assert!(TfaTotp::build(
            "steam_acc",
            "JBSWY3DPEHPK3PXP",
            Some(6),
            None,
            Some(Algorithm::Steam),
            None
        )
        .is_err());

        // Standard TOTP counter and generate_at_counter
        let totp = TfaTotp::build(
            "totp_acc",
            "JBSWY3DPEHPK3PXP",
            Some(6),
            Some(30),
            Some(Algorithm::SHA1),
            None,
        )
        .unwrap();
        assert_eq!(totp.counter(), None);
        assert!(totp.generate_at_counter(1).is_err());
    }

    #[test]
    fn test_totp_generate_next_token() {
        let totp = TfaTotp::build(
            "test_totp",
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ",
            Some(6),
            Some(30),
            Some(Algorithm::SHA1),
            None,
        )
        .unwrap();

        // Check at fixed timestamps
        let t1 = 1_000_000;
        let ttl = totp.ttl_at_time(t1);
        assert_eq!(ttl, 30 - (t1 % 30));
        let code_now = totp.generate_at_time(t1).unwrap();
        let code_next = totp.generate_next_at_time(t1).unwrap();
        let code_expected_next = totp.generate_at_time(t1 + ttl).unwrap();
        assert_eq!(code_next, code_expected_next);
        assert_ne!(code_now, code_next);

        // Check generate_next() live returns a 6-digit code
        let live_next = totp.generate_next().unwrap();
        assert_eq!(live_next.len(), 6);
        assert!(live_next.chars().all(|c| c.is_ascii_digit()));

        // HOTP accounts fail generate_next and generate_next_at_time
        let hotp = TfaHotp::from_url(
            "otpauth://hotp/alice?secret=JBSWY3DPEHPK3PXP&counter=0&algorithm=SHA1",
        )
        .unwrap();
        let hotp_tfa = TfaTotp {
            inner: TfaEngine::Hotp(hotp),
            prefix: None,
        };
        assert!(hotp_tfa.generate_next().is_err());
        assert!(hotp_tfa.generate_next_at_time(t1).is_err());
        assert!(hotp_tfa.generate_at_time(t1).is_err());
        assert_eq!(hotp_tfa.ttl_at_time(t1), 0);
    }
}
