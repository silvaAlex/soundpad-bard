use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const PBKDF2_ITERATIONS: u32 = 100_000;
const ENCRYPTED_PREFIX: &str = "enc:";

pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_PREFIX)
}

fn machine_key_material() -> Result<String> {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "default-host".to_string());

    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default-user".to_string());

    Ok(format!("{}:{}", hostname, username))
}

fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(salt);

    let base = hasher.finalize();

    let mut current = base.to_vec();
    for i in 1..PBKDF2_ITERATIONS {
        let mut h = Sha256::new();
        h.update(&current);
        h.update(salt);
        h.update(i.to_le_bytes());
        current = h.finalize().to_vec();
        for (a, b) in key.iter_mut().zip(current.iter()) {
            *a ^= b;
        }
    }

    Ok(key)
}

pub fn encrypt_password(plaintext: &str) -> Result<String> {
    let key_material = machine_key_material()?;

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(key_material.as_bytes(), &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).context("falha ao criar cipher AES-256-GCM")?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("falha ao criptografar senha: {}", e))?;

    let mut payload = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    Ok(format!("{}{}", ENCRYPTED_PREFIX, STANDARD.encode(&payload)))
}

pub fn decrypt_password(encrypted: &str) -> Result<String> {
    if !is_encrypted(encrypted) {
        return Ok(encrypted.to_owned());
    }

    let encoded = &encrypted[ENCRYPTED_PREFIX.len()..];
    let payload = STANDARD
        .decode(encoded)
        .context("falha ao decodificar base64 da senha criptografada")?;

    if payload.len() < SALT_LEN + NONCE_LEN + 1 {
        anyhow::bail!("payload de senha criptografada muito curto");
    }

    let salt = &payload[..SALT_LEN];
    let nonce_bytes = &payload[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &payload[SALT_LEN + NONCE_LEN..];

    let key_material = machine_key_material()?;
    let key = derive_key(key_material.as_bytes(), salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).context("falha ao criar cipher AES-256-GCM")?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("falha ao descriptografar senha - chave invalida ou dados corrompidos: {}", e))?;

    String::from_utf8(plaintext).context("senha descriptografada nao e UTF-8 valido")
}
