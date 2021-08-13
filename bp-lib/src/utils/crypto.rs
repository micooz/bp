use bytes::Bytes;
use hkdf::Hkdf;
use rand::{rngs::StdRng, thread_rng, RngCore, SeedableRng};
use sha2::Sha256;

pub fn random_bytes(len: usize) -> Bytes {
    let mut rng = StdRng::from_rng(thread_rng()).unwrap();
    let mut pad_buf = vec![0u8; len];

    rng.fill_bytes(&mut pad_buf);
    pad_buf.into()
}

pub fn random_u8() -> u8 {
    random_bytes(1)[0]
}

pub fn hkdf_sha256(ikm: Bytes, salt: Bytes, info: Bytes, len: usize) -> Bytes {
    let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &ikm);
    let mut okm = [0u8; 42];

    hk.expand(&info, &mut okm)
        .expect("42 is a valid length for Sha256 to output");

    Vec::from(&okm[0..len]).into()
}
