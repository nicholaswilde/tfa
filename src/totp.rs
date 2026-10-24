use anyhow::{Context, Result};
use totp_rs::{Algorithm, Builder, Secret, Totp};

pub struct TfaTotp {
    inner: Totp,
}

impl TfaTotp {
    /// Create TOTP instance from raw base32 secret or otpauth:// URI
    pub fn from_input(account: &str, input: &str) -> Result<Self> {
        let input = input.trim();
        if input.starts_with("otpauth://") {
            let totp = Totp::from_url_unchecked(input)
                .map_err(|e| anyhow::anyhow!("Invalid OTPAuth URL: {e}"))?;
            return Ok(Self { inner: totp });
        }

        // Clean base32 secret (remove spaces/dashes, uppercase)
        let clean_secret: String = input
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect::<String>()
            .to_uppercase();

        let secret = Secret::try_from_base32(clean_secret)
            .map_err(|e| anyhow::anyhow!("Invalid base32 secret: {e}"))?;

        let totp = Builder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_step_duration(30)
            .with_secret(secret)
            .with_account_name(account)
            .with_issuer(Some("tfa"))
            .build()
            .context("Failed building TOTP engine")?;

        Ok(Self { inner: totp })
    }

    /// Generate current 6-digit TOTP code
    pub fn generate_current(&self) -> String {
        self.inner.generate_current().to_string()
    }

    /// Seconds remaining until current code expires
    pub fn ttl(&self) -> u64 {
        self.inner.ttl()
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
    }

    #[test]
    fn test_totp_from_formatted_base32() {
        // Space and dash separated base32 secret
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
    }

    #[test]
    fn test_totp_invalid_secret_fails() {
        let invalid = "not-a-valid-base32-secret-999!!!";
        let tfa = TfaTotp::from_input("test", invalid);
        assert!(tfa.is_err());
    }
}
