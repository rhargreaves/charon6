use cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};

pub fn encrypt(block: &[u8; 8], key: &[u8; 16]) -> [u8; 8] {
    let cipher = xtea::Xtea::new_from_slice(key).unwrap();
    let mut data = (*block).into();
    cipher.encrypt_block(&mut data);
    data.into()
}

pub fn decrypt(block: &[u8; 8], key: &[u8; 16]) -> [u8; 8] {
    let cipher = xtea::Xtea::new_from_slice(key).unwrap();
    let mut data = (*block).into();
    cipher.decrypt_block(&mut data);
    data.into()
}

pub fn key_from_passphrase(passphrase: &str) -> [u8; 16] {
    let bytes = passphrase.as_bytes();
    let mut key = [0u8; 16];
    for (i, &b) in bytes.iter().enumerate() {
        key[i % 16] = key[i % 16]
            .wrapping_add(b)
            .wrapping_mul(0x9B)
            .rotate_left(3);
    }
    // Extra mixing
    for i in 0..16 {
        key[i] = key[i]
            .wrapping_add(key[(i + 7) % 16])
            .wrapping_mul(0x6D)
            .rotate_left(5);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 16] {
        key_from_passphrase("test-password")
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = test_key();
        let plaintext: [u8; 8] = [0x01, 0x06, b'h', b'e', b'l', b'l', b'o', b' '];
        let ciphertext = encrypt(&plaintext, &key);
        assert_ne!(ciphertext, plaintext);
        let decrypted = decrypt(&ciphertext, &key);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_output_for_different_inputs() {
        let key = test_key();
        let a = encrypt(&[0, 6, b'a', b'b', b'c', b'd', b'e', b'f'], &key);
        let b = encrypt(&[1, 6, b'a', b'b', b'c', b'd', b'e', b'f'], &key);
        assert_ne!(a, b);
    }

    #[test]
    fn decrypt_with_wrong_key_produces_wrong_output() {
        let key = test_key();
        let plaintext: [u8; 8] = [0x00, 0x03, b'h', b'i', b'!', 0, 0, 0];
        let ciphertext = encrypt(&plaintext, &key);
        let wrong_key = key_from_passphrase("wrong-password");
        let decrypted = decrypt(&ciphertext, &wrong_key);
        assert_ne!(decrypted, plaintext);
    }
}
