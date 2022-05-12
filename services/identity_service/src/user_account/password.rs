use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

/// Produces a hashed value of the given password to be stored in a persistent storage. The algorithm
/// used for hashing the password is Argon2id.
pub fn hash_password(val: &String) -> argon2::password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    Ok(argon2.hash_password(val.as_bytes(), &salt)?.to_string())
}

/// Verifies the given password `sub` against a hashed value stored in a persistent storage. If the
/// passwords match, then an `Ok(())` is returned, otherwise an error is returned.
///
/// # Errors
///
/// In case `sub` does not match the hashed value `actual_hashed`, `Error::Password` is returned.
/// However, the underlying password hash system may return other errors.
pub fn verify_password(sub: &String, actual_hashed: &String) -> argon2::password_hash::Result<()> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(actual_hashed.as_ref())?;

    argon2.verify_password(sub.as_bytes(), &parsed_hash)
}
