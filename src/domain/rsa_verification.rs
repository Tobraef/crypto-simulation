use anyhow::{bail, Result};
use rsa::{PaddingScheme, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RSAEncodedMsg(Vec<u8>);
pub struct PubKey(rsa::RsaPublicKey);
pub struct PrivKey(rsa::RsaPrivateKey);

const PRIVATE_KEY_LEN: usize = 1024;
const RSA_MESSAGE_LEN: usize = 117;

pub fn generate_key() -> Result<PrivKey> {
    let mut rng = rand::thread_rng();
    rsa::RsaPrivateKey::new(&mut rng, PRIVATE_KEY_LEN)
        .map(|key| PrivKey(key))
        .map_err(anyhow::Error::msg)
}

pub fn encode_message(serialized_data: &[u8], private_key: &PrivKey) -> Result<RSAEncodedMsg> {
    if serialized_data.len() > RSA_MESSAGE_LEN {
        bail!(format!(
            "Proof is too long, can be up to {RSA_MESSAGE_LEN} but is {}",
            serialized_data.len()
        ))
    } else {
        private_key
            .0
            .sign(PaddingScheme::new_pkcs1v15_sign(None), serialized_data)
            .map(|bytes| RSAEncodedMsg(bytes))
            .map_err(|e| e.into())
    }
}

pub fn verify_message(
    serialized_data: &[u8],
    signed_data: Vec<u8>,
    public_key: &PubKey,
) -> Result<RSAEncodedMsg> {
    public_key
        .0
        .verify(
            PaddingScheme::PKCS1v15Encrypt,
            serialized_data,
            &signed_data,
        )
        .map(|_| RSAEncodedMsg(signed_data))
        .map_err(anyhow::Error::msg)
}
