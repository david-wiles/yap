use std::num::NonZeroU32;

use ring::aead::{BoundKey, Nonce, NonceSequence, NONCE_LEN, SealingKey, UnboundKey, AES_256_GCM, Aad};
use ring::pbkdf2::{derive, PBKDF2_HMAC_SHA256};
use ring::rand::{SecureRandom, SystemRandom};

use crate::{Error, Result};

pub trait Engine {
    fn encrypt_bytes(&self, bytes: &mut [u8]) -> Result<&[u8]>;
    fn decrypt_bytes(&self, bytes: &[u8]) -> Result<&[u8]>;
}

struct RandomNonceSequence {}

impl NonceSequence for RandomNonceSequence {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        let mut nonce_buf = [0u8; NONCE_LEN];
        SystemRandom::new().fill(&mut nonce_buf)?;
        Ok(Nonce::try_assume_unique_for_key(&nonce_buf)?)
    }
}

pub struct Aes256Engine<'a> {
    key: &'a [u8]
}

impl<'a> Aes256Engine<'a> {
    pub fn new(key: &'a [u8]) -> Aes256Engine {
        Aes256Engine { key }
    }
}

impl<'a> Engine for Aes256Engine<'a> {
    fn encrypt_bytes(&self, payload: &mut [u8]) -> Result<&[u8]> {
        let mut sealing_key = SealingKey::new(UnboundKey::new(&AES_256_GCM, self.key)?, RandomNonceSequence{});

        sealing_key.seal_in_place_append_tag(Aad::empty(), payload)?;

        Ok(payload)
    }

    fn decrypt_bytes(&self, bytes: &[u8]) -> Result<&[u8]> {
        todo!()
    }
}

pub fn derive_key_from_pass<'a>(pass: String) -> Result<&'a [u8]> {
    let mut key = [0u8, 32];

    // TODO salt?
    let salt = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    // Derive the key and store in `key`
    derive(PBKDF2_HMAC_SHA256, NonZeroU32::new(100000u32).unwrap(), &salt, &pass.as_bytes(), &mut key);

    Ok(&key)
}

/* From ChatGPT
use ring::{aead, error};
use ring::pbkdf2;

const KEY_LEN: usize = 32; // 256-bit key
const SALT: &[u8] = b"my_salt"; // salt for the PBKDF2 key derivation function
const NONCE_LEN: usize = 12; // nonce length for AES-GCM
const TAG_LEN: usize = 16; // tag length for AES-GCM

// Derive a key from a password using PBKDF2
fn derive_key(password: &[u8]) -> Result<[u8; KEY_LEN], error::Unspecified> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2::derive(
        &pbkdf2::PBKDF2_HMAC_SHA256,
        pbkdf2::Iterations(100_000),
        SALT,
        password,
        &mut key,
    );
    Ok(key)
}

// Encrypt data with a password
fn encrypt(password: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, error::Unspecified> {
    let key = derive_key(password)?;
    let aead = aead::Aes256Gcm::new(aead::Aes256GcmKey::new(&key));

    // Generate a random nonce
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill(&mut nonce);

    // Encrypt the data
    let ciphertext = aead.seal(nonce, plaintext, b"", TAG_LEN);
    Ok(ciphertext)
}

// Decrypt data with a password
fn decrypt(password: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, error::Unspecified> {
    let key = derive_key(password)?;
    let aead = aead::Aes256Gcm::new(aead::Aes256GcmKey::new(&key));

    // Extract the nonce and tag from the ciphertext
    let nonce = &ciphertext[..NONCE_LEN];
    let tag = &ciphertext[ciphertext.len() - TAG_LEN..];
    let ciphertext = &ciphertext[NONCE_LEN..ciphertext.len() - TAG_LEN];

    // Decrypt the data
    let plaintext = aead.open(nonce, ciphertext, b"", tag)?;
    Ok(plaintext)
}
 */