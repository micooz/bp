use bytes::Bytes;
use hkdf::Hkdf;
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, RngCore, SeedableRng};
use sha2::Sha256;

pub struct Crypto;

impl Crypto {
    fn std_rng() -> StdRng {
        StdRng::from_rng(thread_rng()).unwrap()
    }

    pub fn random_bytes(len: usize) -> Bytes {
        let mut rng = Self::std_rng();
        let mut pad_buf = vec![0u8; len];

        rng.fill_bytes(&mut pad_buf);
        pad_buf.into()
    }

    pub fn random_u8() -> u8 {
        Self::random_bytes(1)[0]
    }

    pub fn random_choose<T>(arr: &[T]) -> Option<&T> {
        let mut rng = Self::std_rng();
        SliceRandom::choose(arr, &mut rng)
    }

    pub fn hkdf_sha256(ikm: Bytes, salt: Bytes, info: Bytes, len: usize) -> Bytes {
        let hk = Hkdf::<Sha256>::new(Some(&salt[..]), &ikm);
        let mut okm = [0u8; 42];

        hk.expand(&info, &mut okm)
            .expect("42 is a valid length for Sha256 to output");

        Vec::from(&okm[0..len]).into()
    }
}
