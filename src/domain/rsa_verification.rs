use std::fmt::Debug;

use anyhow::{bail, Result};
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    PaddingScheme, PublicKey,
};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RSAEncodedMsg(Vec<u8>);
#[derive(Debug, Clone)]
pub struct PubKey(rsa::RsaPublicKey);
pub struct PrivKey(rsa::RsaPrivateKey);

const PRIVATE_KEY_LEN: usize = 1024;
const RSA_MESSAGE_LEN: usize = 117;

impl Debug for RSAEncodedMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.0))
    }
}

struct KeyVisitor;
impl<'de> Visitor<'de> for KeyVisitor {
    type Value = String;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expecting string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}
impl<'de> Deserialize<'de> for PubKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = deserializer.deserialize_string(KeyVisitor)?;
        let pub_key = rsa::RsaPublicKey::from_pkcs1_pem(&str).map_err(|_| {
            serde::de::Error::custom(&format!(
                "Couldn't create public key from saved pem: {}",
                str
            ))
        })?;
        Ok(PubKey(pub_key))
    }
}

impl Serialize for PubKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let key = self
            .0
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::CRLF)
            .map_err(|_| serde::ser::Error::custom("Couldn't serialize pub key."))?;
        serializer.serialize_str(&key)
    }
}

pub fn generate_key() -> Result<(PrivKey, PubKey)> {
    let mut rng = rand::thread_rng();
    let key = rsa::RsaPrivateKey::new(&mut rng, PRIVATE_KEY_LEN).map(|key| PrivKey(key))?;
    let pub_key = key.0.to_public_key();
    Ok((key, PubKey(pub_key)))
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
