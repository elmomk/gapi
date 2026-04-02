/// Encryption vault using ChaCha20Poly1305
use base64::prelude::*;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};
use rand::RngCore;

pub struct Vault {
    cipher: ChaCha20Poly1305,
}

impl Vault {
    pub fn new(master_key: &str) -> Self {
        let key_bytes = master_key.as_bytes();
        assert!(
            key_bytes.len() >= 32,
            "MASTER_KEY must be at least 32 bytes, got {}. Use a strong random key.",
            key_bytes.len()
        );
        let key: [u8; 32] = key_bytes[..32].try_into().expect("key slice");
        Self {
            cipher: ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&key)),
        }
    }

    pub fn encrypt(&self, data: &str) -> anyhow::Result<(String, String)> {
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        Ok((
            BASE64_STANDARD.encode(ciphertext),
            BASE64_STANDARD.encode(nonce_bytes),
        ))
    }

    pub fn decrypt(&self, ct_b64: &str, nonce_b64: &str) -> anyhow::Result<String> {
        let ct = BASE64_STANDARD.decode(ct_b64)?;
        let nonce_bytes = BASE64_STANDARD.decode(nonce_b64)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let pt = self
            .cipher
            .decrypt(nonce, ct.as_slice())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
        Ok(String::from_utf8(pt)?)
    }
}
