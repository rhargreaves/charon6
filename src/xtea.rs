const NUM_ROUNDS: u32 = 32;
const DELTA: u32 = 0x9E3779B9;

pub fn encrypt(block: &[u8; 8], key: &[u32; 4]) -> [u8; 8] {
    let mut v0 = u32::from_be_bytes(block[0..4].try_into().unwrap());
    let mut v1 = u32::from_be_bytes(block[4..8].try_into().unwrap());
    let mut sum: u32 = 0;

    for _ in 0..NUM_ROUNDS {
        v0 = v0.wrapping_add(
            ((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1) ^ sum.wrapping_add(key[(sum & 3) as usize]),
        );
        sum = sum.wrapping_add(DELTA);
        v1 = v1.wrapping_add(
            ((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)
                ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
        );
    }

    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&v0.to_be_bytes());
    out[4..8].copy_from_slice(&v1.to_be_bytes());
    out
}

pub fn decrypt(block: &[u8; 8], key: &[u32; 4]) -> [u8; 8] {
    let mut v0 = u32::from_be_bytes(block[0..4].try_into().unwrap());
    let mut v1 = u32::from_be_bytes(block[4..8].try_into().unwrap());
    let mut sum: u32 = DELTA.wrapping_mul(NUM_ROUNDS);

    for _ in 0..NUM_ROUNDS {
        v1 = v1.wrapping_sub(
            ((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)
                ^ sum.wrapping_add(key[((sum >> 11) & 3) as usize]),
        );
        sum = sum.wrapping_sub(DELTA);
        v0 = v0.wrapping_sub(
            ((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1) ^ sum.wrapping_add(key[(sum & 3) as usize]),
        );
    }

    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&v0.to_be_bytes());
    out[4..8].copy_from_slice(&v1.to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u32; 4] = [0x01234567, 0x89ABCDEF, 0xFEDCBA98, 0x76543210];

    #[test]
    fn encrypt_decrypt_round_trip() {
        let plaintext: [u8; 8] = [0x01, 0x06, b'h', b'e', b'l', b'l', b'o', b' '];
        let ciphertext = encrypt(&plaintext, &TEST_KEY);
        assert_ne!(ciphertext, plaintext);
        let decrypted = decrypt(&ciphertext, &TEST_KEY);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_output_for_different_inputs() {
        let a = encrypt(&[0, 6, b'a', b'b', b'c', b'd', b'e', b'f'], &TEST_KEY);
        let b = encrypt(&[1, 6, b'a', b'b', b'c', b'd', b'e', b'f'], &TEST_KEY);
        assert_ne!(a, b);
    }

    #[test]
    fn decrypt_with_wrong_key_produces_wrong_output() {
        let plaintext: [u8; 8] = [0x00, 0x03, b'h', b'i', b'!', 0, 0, 0];
        let ciphertext = encrypt(&plaintext, &TEST_KEY);
        let wrong_key: [u32; 4] = [0, 0, 0, 0];
        let decrypted = decrypt(&ciphertext, &wrong_key);
        assert_ne!(decrypted, plaintext);
    }
}
