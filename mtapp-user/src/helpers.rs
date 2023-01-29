// Credit: https://blue42.net/code/rust/examples/sodiumoxide-password-hashing/post/
use sodiumoxide::crypto::pwhash::argon2id13;

pub(crate) fn hash(password: &str) -> String {
    let hash = argon2id13::pwhash(
        password.as_bytes(),
        argon2id13::OPSLIMIT_MODERATE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    // TODO: can it somehow error in runtime?
    .expect("Should not error");
    std::str::from_utf8(&hash.0)
        .expect("Hash is valid utf8")
        .trim_end_matches('\u{0}')
        .to_owned()
}

pub(crate) fn verify(password: &str, hash: &str) -> bool {
    let mut padded_hash = [0_u8; 128];
    hash.as_bytes().iter().enumerate().for_each(|(i, val)| {
        padded_hash[i] = *val;
    });
    match argon2id13::HashedPassword::from_slice(&padded_hash) {
        Some(hp) => argon2id13::pwhash_verify(&hp, password.as_bytes()),
        None => false,
    }
}
