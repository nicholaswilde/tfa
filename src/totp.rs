use anyhow::Result;
use totp_rs::{Algorithm, Builder, Secret, Totp};

pub struct TfaTotp {
    inner: Totp,
}

impl TfaTotp {
    /// Create TOTP instance with defaults from raw base32 secret or otpauth:// URI
    pub fn from_input(account: &str, input: &str) -> Result<Self> {
        Self::build(account, input, None, None, None)
    }

    /// Build TOTP instance with optional custom digits, period, and algorithm
    pub fn build(
        account: &str,
        input: &str,
        digits: Option<u8>,
        period: Option<u64>,
        algorithm: Option<Algorithm>,
    ) -> Result<Self> {
        let input = input.trim();
        let base_totp = if input.starts_with("otpauth://") {
            Totp::from_url_unchecked(input)
                .map_err(|e| anyhow::anyhow!("Invalid OTPAuth URL: {e}"))?
        } else {
            let clean_secret: String = input
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-')
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
        let final_digits = digits.unwrap_or_else(|| base_totp.digits());
        let final_step = period.unwrap_or_else(|| base_totp.step());
        let final_issuer = base_totp.issuer().unwrap_or("tfa");
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

        Ok(Self { inner: totp })
    }

    /// Generate current TOTP code
    pub fn generate_current(&self) -> String {
        self.inner.generate_current().to_string()
    }

    /// Seconds remaining until current code expires
    pub fn ttl(&self) -> u64 {
        self.inner.ttl()
    }

    /// Number of digits in generated code
    pub fn digits(&self) -> u8 {
        self.inner.digits()
    }

    /// Time step period in seconds
    pub fn step(&self) -> u64 {
        self.inner.step()
    }

    /// Hash algorithm
    pub fn algorithm(&self) -> Algorithm {
        self.inner.algorithm()
    }

    /// Canonical OTPAuth URL representation
    pub fn to_url(&self) -> Result<String> {
        self.inner
            .to_url()
            .map_err(|e| anyhow::anyhow!("Failed exporting OTPAuth URL: {e}"))
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

    #[test]
    fn test_totp_custom_digits_period_algorithm() {
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let tfa = TfaTotp::build(
            "custom_acc",
            secret,
            Some(8),
            Some(60),
            Some(Algorithm::SHA256),
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
}
