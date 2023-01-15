use std::num::NonZeroU32;
use std::result;

use ring::aead::{BoundKey, Nonce, NonceSequence, NONCE_LEN, SealingKey, UnboundKey, AES_256_GCM, Aad, OpeningKey};
use ring::pbkdf2::{derive, PBKDF2_HMAC_SHA256};
use ring::rand::{Random, SecureRandom, SystemRandom};

use crate::{Error, Result};

pub trait Engine {
    fn encrypt_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>>;
    fn decrypt_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>>;
}

struct OneNonceSequence(Option<Nonce>);

impl OneNonceSequence {
    fn new(nonce: Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> result::Result<Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}

fn next_nonce_bytes() -> Result<[u8; NONCE_LEN]> {
    let mut nonce_buf = [0u8; NONCE_LEN];
    SystemRandom::new().fill(&mut nonce_buf)?;
    Ok(nonce_buf)
}

pub struct Aes256GcmEngine {
    key: [u8; 32],
}

impl Aes256GcmEngine {
    pub fn new(pass: String) -> Result<Aes256GcmEngine> {
        Ok(Aes256GcmEngine { key: derive_key_from_pass(pass)? })
    }
}

impl Engine for Aes256GcmEngine {
    fn encrypt_bytes(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let nonce_bytes = next_nonce_bytes()?;
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes.as_slice())?;

        let mut sealing_key = SealingKey::new(UnboundKey::new(&AES_256_GCM, &self.key)?, OneNonceSequence::new(nonce));
        let mut raw = payload.to_owned();
        sealing_key.seal_in_place_append_tag(Aad::empty(), &mut raw)?;

        // Append the nonce to the beginning of the encrypted bytes
        let mut data = nonce_bytes.to_vec();
        data.append(&mut raw);

        Ok(data)
    }

    fn decrypt_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        // Split the incoming bytes at the nonce length
        let (nonce_bytes, bytes) = bytes.split_at(NONCE_LEN);
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)?;

        let mut opening_key = OpeningKey::new(UnboundKey::new(&AES_256_GCM, &self.key)?, OneNonceSequence::new(nonce));
        let mut raw = bytes.to_owned();
        let plaintext = opening_key.open_in_place(Aad::empty(), &mut raw)?;

        // Remove tag length?
        Ok(plaintext.to_owned())
    }
}

pub fn derive_key_from_pass<'a>(pass: String) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];

    // TODO salt?
    let salt = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // Derive the key and store in `key`
    derive(PBKDF2_HMAC_SHA256, NonZeroU32::new(100000u32).unwrap(), &salt, &pass.as_bytes(), &mut key);

    Ok(key)
}

#[cfg(test)]
mod test {
    use crate::crypto::{Aes256GcmEngine, Engine};

    #[test]
    fn can_encrypt_and_decrypt_bytes() {
        let engine = Aes256GcmEngine::new("key".to_string()).unwrap();
        let message = "some message".as_bytes();

        let encrypted = engine.encrypt_bytes(message).unwrap();
        let decrypted = engine.decrypt_bytes(encrypted.as_slice()).unwrap();

        assert_eq!(message, decrypted.as_slice());
    }
}
