use cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

const KEY_LEN: usize = 16;
pub(crate) const HMAC_LEN: usize = 16;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cipher([u8; KEY_LEN]);

impl Cipher {
    pub fn from_passphrase(passphrase: &str) -> Self {
        let hash = Sha256::digest(passphrase.as_bytes());
        Self(hash[..KEY_LEN].try_into().unwrap())
    }

    pub fn encrypt(&self, block: &[u8; 8]) -> [u8; 8] {
        let cipher = xtea::Xtea::new((&self.0).into());
        let mut data = (*block).into();
        cipher.encrypt_block(&mut data);
        data.into()
    }

    pub fn decrypt(&self, block: &[u8; 8]) -> [u8; 8] {
        let cipher = xtea::Xtea::new((&self.0).into());
        let mut data = (*block).into();
        cipher.decrypt_block(&mut data);
        data.into()
    }

    pub fn compute_hmac(&self, message: &[u8]) -> [u8; HMAC_LEN] {
        let mut mac = HmacSha256::new_from_slice(&self.0).expect("HMAC accepts any key length");
        mac.update(message);
        let result = mac.finalize().into_bytes();
        result[..HMAC_LEN].try_into().unwrap()
    }

    pub fn verify_hmac(&self, message: &[u8], tag: &[u8; HMAC_LEN]) -> bool {
        self.compute_hmac(message) == *tag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cipher() -> Cipher {
        Cipher::from_passphrase("test-password")
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = test_cipher();
        let plaintext: [u8; 8] = [0x01, 0x06, b'h', b'e', b'l', b'l', b'o', b' '];
        let ciphertext = key.encrypt(&plaintext);
        assert_ne!(ciphertext, plaintext);
        let decrypted = key.decrypt(&ciphertext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_output_for_different_inputs() {
        let key = test_cipher();
        let a = key.encrypt(&[0, 6, b'a', b'b', b'c', b'd', b'e', b'f']);
        let b = key.encrypt(&[1, 6, b'a', b'b', b'c', b'd', b'e', b'f']);
        assert_ne!(a, b);
    }

    #[test]
    fn decrypt_with_wrong_key_produces_wrong_output() {
        let key = test_cipher();
        let plaintext: [u8; 8] = [0x00, 0x03, b'h', b'i', b'!', 0, 0, 0];
        let ciphertext = key.encrypt(&plaintext);
        let wrong_key = Cipher::from_passphrase("wrong-password");
        let decrypted = wrong_key.decrypt(&ciphertext);
        assert_ne!(decrypted, plaintext);
    }

    #[test]
    fn empty_passphrase_produces_valid_key() {
        let key = Cipher::from_passphrase("");
        let plaintext: [u8; 8] = [0, 3, b'a', b'b', b'c', 0, 0, 0];
        let decrypted = key.decrypt(&key.encrypt(&plaintext));
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn hmac_verifies_correct_message() {
        let c = test_cipher();
        let tag = c.compute_hmac(b"hello world");
        assert!(c.verify_hmac(b"hello world", &tag));
    }

    #[test]
    fn hmac_rejects_tampered_message() {
        let c = test_cipher();
        let tag = c.compute_hmac(b"hello world");
        assert!(!c.verify_hmac(b"hello worlD", &tag));
    }

    #[test]
    fn hmac_rejects_wrong_key() {
        let c1 = test_cipher();
        let c2 = Cipher::from_passphrase("other-password");
        let tag = c1.compute_hmac(b"hello");
        assert!(!c2.verify_hmac(b"hello", &tag));
    }
}
