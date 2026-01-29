use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, serde::Deserialize)]
struct Claims {
    iss: String,
    iat: u64,
    exp: u64,
    aud: String,
}

pub fn generate_token(
    key_id: &str,
    issuer_id: &str,
    key_pem: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        iss: issuer_id.to_string(),
        iat: now,
        exp: now + 20 * 60, // 20 minutes
        aud: "appstoreconnect-v1".to_string(),
    };
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id.to_string());
    header.typ = Some("JWT".to_string());
    let key = EncodingKey::from_ec_pem(key_pem)?;
    Ok(encode(&header, &claims, &key)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    const TEST_EC_PRIVATE_KEY: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgP1Aoh3bRVcJFYVgd
77v6MnHD4pCtoPwxSlj/fpgJiKqhRANCAATSn1ZpcLRHMEcZSJrv/LpG7IfDTe49
g5oWeSRv7yNOxuquI8UBjC9E6beP/57Rvrjimf2xW3Iw3UedqMBEe+Pw
-----END PRIVATE KEY-----";

    const TEST_EC_PUBLIC_KEY: &[u8] = b"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE0p9WaXC0RzBHGUia7/y6RuyHw03u
PYOaFnkkb+8jTsbqriPFAYwvROm3j/+e0b644pn9sVtyMN1HnajARHvj8A==
-----END PUBLIC KEY-----";

    fn generate_test_token() -> String {
        generate_token("TEST_KEY_ID", "TEST_ISSUER_ID", TEST_EC_PRIVATE_KEY).unwrap()
    }

    #[test]
    fn token_is_generated_successfully() {
        let token = generate_test_token();
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn token_header_has_correct_algorithm_and_kid() {
        let token = generate_test_token();
        let header = jsonwebtoken::decode_header(&token).unwrap();
        assert_eq!(header.alg, Algorithm::ES256);
        assert_eq!(header.kid, Some("TEST_KEY_ID".to_string()));
        assert_eq!(header.typ, Some("JWT".to_string()));
    }

    #[test]
    fn token_claims_contain_correct_audience() {
        let token = generate_test_token();
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&["appstoreconnect-v1"]);
        validation.set_required_spec_claims(&["iss", "aud", "iat", "exp"]);
        let key = DecodingKey::from_ec_pem(TEST_EC_PUBLIC_KEY).unwrap();
        let data = decode::<Claims>(&token, &key, &validation).unwrap();
        assert_eq!(data.claims.aud, "appstoreconnect-v1");
        assert_eq!(data.claims.iss, "TEST_ISSUER_ID");
    }

    #[test]
    fn token_expiry_is_20_minutes() {
        let token = generate_test_token();
        let mut validation = Validation::new(Algorithm::ES256);
        validation.insecure_disable_signature_validation();
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["iss", "aud", "iat", "exp"]);
        let key = DecodingKey::from_ec_pem(TEST_EC_PUBLIC_KEY).unwrap();
        let data = decode::<Claims>(&token, &key, &validation).unwrap();
        assert_eq!(data.claims.exp - data.claims.iat, 20 * 60);
    }
}
