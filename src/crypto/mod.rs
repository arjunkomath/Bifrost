use anyhow::Result;
use ring::aead::Aad;
use ring::aead::BoundKey;
use ring::aead::Nonce;
use ring::aead::NonceSequence;
use ring::aead::OpeningKey;
use ring::aead::SealingKey;
use ring::aead::UnboundKey;
use ring::aead::AES_256_GCM;
use ring::aead::NONCE_LEN;
use ring::error::Unspecified;

struct CounterNonceSequence(u32);

impl NonceSequence for CounterNonceSequence {
    // called once for each seal operation
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = vec![0; NONCE_LEN];

        let bytes = self.0.to_be_bytes();
        nonce_bytes[8..].copy_from_slice(&bytes);

        self.0 += 1; // advance the counter
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

pub enum Error {
    CryptoError,
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::CryptoError
    }
}

pub fn encrypt(data: String) -> Result<(String, String), Error> {
    let encryption_key = std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY is required");
    let encryption_key = encryption_key.into_bytes();

    // Create a new AEAD key without a designated role or nonce sequence
    let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key)?;

    // Create a new NonceSequence type which generates nonces
    let nonce_sequence = CounterNonceSequence(1);

    // Create a new AEAD key for encrypting and signing ("sealing"), bound to a nonce sequence
    // The SealingKey can be used multiple times, each time a new nonce will be used
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

    // This data will be authenticated but not encrypted
    let associated_data = Aad::empty(); // is optional so can be empty

    // Create a mutable copy of the data that will be encrypted in place
    let mut in_out = data.into_bytes().clone();

    // Encrypt the data with AEAD using the AES_256_GCM algorithm
    let tag = sealing_key.seal_in_place_separate_tag(associated_data, &mut in_out)?;

    Ok((hex::encode(in_out).to_string(), hex::encode(tag)))
}

pub fn decrypt(data: String) -> Result<String, Error> {
    let encryption_key = std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY is required");
    let encryption_key = encryption_key.into_bytes();

    // Recreate the previously moved variables
    let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key)?;
    let nonce_sequence = CounterNonceSequence(1);
    let associated_data = Aad::empty();

    // Create a new AEAD key for decrypting and verifying the authentication tag
    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

    // Decrypt the data by passing in the associated data and the cypher text with the authentication tag appended
    let mut cypher_text_with_tag = hex::decode(data).unwrap();
    let decrypted_data = opening_key.open_in_place(associated_data, &mut cypher_text_with_tag)?;

    Ok(String::from_utf8(decrypted_data.to_vec()).unwrap())
}
