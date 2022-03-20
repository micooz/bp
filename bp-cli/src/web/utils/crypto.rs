use anyhow::Result;

pub trait Crypto<T> {
    fn encrypt(&self, plaintext: T) -> Result<T>;
    fn decrypt(&self, ciphertext: T) -> Result<T>;
}

pub struct NoneCrypto;

impl Crypto<String> for NoneCrypto {
    fn encrypt(&self, plaintext: String) -> Result<String> {
        Ok(plaintext)
    }

    fn decrypt(&self, ciphertext: String) -> Result<String> {
        Ok(ciphertext)
    }
}

pub struct Base64Crypto;

impl Crypto<String> for Base64Crypto {
    fn encrypt(&self, plaintext: String) -> Result<String> {
        Ok(base64::encode(plaintext))
    }

    fn decrypt(&self, ciphertext: String) -> Result<String> {
        Ok(String::from_utf8(base64::decode(ciphertext)?)?)
    }
}
