use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[tracing::instrument(name = "Hashing user password", skip(password))]
pub async fn hash(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .expect("Unable to hash password.")
        .to_string()
}

#[tracing::instrument(name = "Verifying user password", skip(password, hash))]
pub fn verify_password(hash: &str, password: &[u8]) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Argon2::default().verify_password(password, &parsed_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_web::test]
    #[ignore]
    async fn hash() -> Result<(), String> {
        todo!();
    }

    #[actix_web::test]
    async fn test_verify_password() -> Result<(), String> {
        let password = "password";
        let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";

        let res = verify_password(hashed_password, password.as_bytes());

        assert!(res.is_ok());
        Ok(())
    }

    #[actix_web::test]
    async fn test_verify_incorrect_password() -> Result<(), String> {
        let incorrect_password = "passworda";
        let hashed_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";

        let res = verify_password(hashed_password, incorrect_password.as_bytes());

        assert!(res.is_err());
        Ok(())
    }
}
