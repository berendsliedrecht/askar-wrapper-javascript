use core::{
    convert::TryInto,
    fmt::{self, Debug, Formatter},
};

use crate::generic_array::typenum::U32;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret as SecretKey};
use zeroize::Zeroize;

use crate::{
    buffer::{ArrayKey, SecretBytes, WriteBuffer},
    caps::{KeyGen, KeySecretBytes},
    error::Error,
    jwk::{FromJwk, JwkEncoder, JwkParts, ToJwk},
    kdf::KeyExchange,
};

pub const PUBLIC_KEY_LENGTH: usize = 32;
pub const SECRET_KEY_LENGTH: usize = 32;
pub const KEYPAIR_LENGTH: usize = SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH;

pub static JWK_KEY_TYPE: &'static str = "OKP";
pub static JWK_CURVE: &'static str = "X25519";

#[derive(Clone)]
pub struct X25519KeyPair {
    // SECURITY: SecretKey (StaticSecret) zeroizes on drop
    secret: Option<SecretKey>,
    public: PublicKey,
}

impl X25519KeyPair {
    #[inline(always)]
    pub(crate) fn new(sk: Option<SecretKey>, pk: PublicKey) -> Self {
        Self {
            secret: sk,
            public: pk,
        }
    }

    pub fn from_keypair_bytes(kp: &[u8]) -> Result<Self, Error> {
        if kp.len() != KEYPAIR_LENGTH {
            return Err(err_msg!("Invalid keypair bytes"));
        }
        let sk = SecretKey::from(
            TryInto::<[u8; SECRET_KEY_LENGTH]>::try_into(&kp[..SECRET_KEY_LENGTH]).unwrap(),
        );
        let pk = PublicKey::from(
            TryInto::<[u8; PUBLIC_KEY_LENGTH]>::try_into(&kp[SECRET_KEY_LENGTH..]).unwrap(),
        );
        // FIXME: derive pk from sk and check value?

        Ok(Self {
            secret: Some(sk),
            public: pk,
        })
    }

    #[inline]
    pub(crate) fn from_secret_key(sk: SecretKey) -> Self {
        let public = PublicKey::from(&sk);
        Self {
            secret: Some(sk),
            public,
        }
    }

    pub fn from_public_key_bytes(key: &[u8]) -> Result<Self, Error> {
        if key.len() != PUBLIC_KEY_LENGTH {
            return Err(err_msg!("Invalid x25519 key length"));
        }
        Ok(Self::new(
            None,
            PublicKey::from(TryInto::<[u8; PUBLIC_KEY_LENGTH]>::try_into(key).unwrap()),
        ))
    }

    pub fn to_public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.public.to_bytes()
    }

    pub fn to_keypair_bytes(&self) -> Option<SecretBytes> {
        if let Some(secret) = self.secret.as_ref() {
            let output = SecretBytes::new_with(KEYPAIR_LENGTH, |buf| {
                buf[..SECRET_KEY_LENGTH].copy_from_slice(&secret.to_bytes()[..]);
                buf[SECRET_KEY_LENGTH..].copy_from_slice(self.public.as_bytes());
            });
            Some(output)
        } else {
            None
        }
    }
}

impl Debug for X25519KeyPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("X25519KeyPair")
            .field(
                "secret",
                if self.secret.is_some() {
                    &"<secret>"
                } else {
                    &"None"
                },
            )
            .field("public", &self.public)
            .finish()
    }
}

impl KeyGen for X25519KeyPair {
    fn generate() -> Result<Self, Error> {
        let sk = SecretKey::new(OsRng);
        let pk = PublicKey::from(&sk);
        Ok(Self::new(Some(sk), pk))
    }
}

impl KeySecretBytes for X25519KeyPair {
    fn from_key_secret_bytes(key: &[u8]) -> Result<Self, Error> {
        if key.len() != SECRET_KEY_LENGTH {
            return Err(err_msg!("Invalid x25519 key length"));
        }

        // pre-check key to ensure that clamping has no effect
        if key[0] & 7 != 0 || (key[31] & 127 | 64) != key[31] {
            return Err(err_msg!("invalid x25519 secret key"));
        }

        let sk = SecretKey::from(TryInto::<[u8; SECRET_KEY_LENGTH]>::try_into(key).unwrap());

        // post-check key
        // let mut check = sk.to_bytes();
        // if &check[..] != key {
        //     return Err(err_msg!("invalid x25519 secret key"));
        // }
        // check.zeroize();

        Ok(Self::from_secret_key(sk))
    }

    fn to_key_secret_buffer<B: WriteBuffer>(&self, out: &mut B) -> Result<(), Error> {
        if let Some(sk) = self.secret.as_ref() {
            out.write_slice(&sk.to_bytes()[..])
        } else {
            Err(err_msg!(MissingSecretKey))
        }
    }
}

impl ToJwk for X25519KeyPair {
    fn to_jwk_buffer<B: WriteBuffer>(&self, buffer: &mut JwkEncoder<B>) -> Result<(), Error> {
        buffer.add_str("kty", JWK_KEY_TYPE)?;
        buffer.add_str("crv", JWK_CURVE)?;
        buffer.add_as_base64("x", &self.to_public_key_bytes()[..])?;
        if buffer.is_secret() {
            if let Some(sk) = self.secret.as_ref() {
                let mut sk = sk.to_bytes();
                buffer.add_as_base64("d", &sk[..])?;
                sk.zeroize();
            }
        }
        buffer.add_str("use", "enc")?;
        Ok(())
    }
}

impl FromJwk for X25519KeyPair {
    fn from_jwk_parts(jwk: JwkParts<'_>) -> Result<Self, Error> {
        // SECURITY: ArrayKey zeroizes on drop
        let mut pk = ArrayKey::<U32>::default();
        if jwk.x.decode_base64(pk.as_mut())? != pk.as_ref().len() {
            return Err(err_msg!("invalid length for x25519 attribute 'x'"));
        }
        let pk = PublicKey::from(
            TryInto::<[u8; PUBLIC_KEY_LENGTH]>::try_into(&pk.as_ref()[..]).unwrap(),
        );
        let sk = if jwk.d.is_some() {
            let mut sk = ArrayKey::<U32>::default();
            if jwk.d.decode_base64(sk.as_mut())? != sk.as_ref().len() {
                return Err(err_msg!("invalid length for x25519 attribute 'd'"));
            }
            Some(SecretKey::from(
                TryInto::<[u8; SECRET_KEY_LENGTH]>::try_into(&sk.as_ref()[..]).unwrap(),
            ))
        } else {
            None
        };
        Ok(Self {
            secret: sk,
            public: pk,
        })
    }
}

impl KeyExchange for X25519KeyPair {
    fn key_exchange_buffer<B: WriteBuffer>(&self, other: &Self, out: &mut B) -> Result<(), Error> {
        match self.secret.as_ref() {
            Some(sk) => {
                let xk = sk.diffie_hellman(&other.public);
                out.write_slice(xk.as_bytes())?;
                Ok(())
            }
            None => Err(err_msg!(MissingSecretKey)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwk_expected() {
        // {
        //   "kty": "OKP",
        //   "d": "qL25gw-HkNJC9m4EsRzCoUx1KntjwHPzxo6a2xUcyFQ",
        //   "use": "enc",
        //   "crv": "X25519",
        //   "x": "tGskN_ae61DP4DLY31_fjkbvnKqf-ze7kA6Cj2vyQxU"
        // }
        let test_pvt_b64 = "qL25gw-HkNJC9m4EsRzCoUx1KntjwHPzxo6a2xUcyFQ";
        let test_pvt = base64::decode_config(test_pvt_b64, base64::URL_SAFE).unwrap();
        let kp =
            X25519KeyPair::from_key_secret_bytes(&test_pvt).expect("Error creating x25519 keypair");
        let jwk = kp
            .to_jwk_public()
            .expect("Error converting public key to JWK");
        let jwk = jwk.to_parts().expect("Error parsing JWK output");
        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, JWK_CURVE);
        assert_eq!(jwk.x, "tGskN_ae61DP4DLY31_fjkbvnKqf-ze7kA6Cj2vyQxU");
        assert_eq!(jwk.d, None);
        let pk_load = X25519KeyPair::from_jwk_parts(jwk).unwrap();
        assert_eq!(kp.to_public_key_bytes(), pk_load.to_public_key_bytes());

        let jwk = kp
            .to_jwk_secret()
            .expect("Error converting private key to JWK");
        let jwk = jwk.to_parts().expect("Error parsing JWK output");
        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, JWK_CURVE);
        assert_eq!(jwk.x, "tGskN_ae61DP4DLY31_fjkbvnKqf-ze7kA6Cj2vyQxU");
        assert_eq!(jwk.d, test_pvt_b64);
        let sk_load = X25519KeyPair::from_jwk_parts(jwk).unwrap();
        assert_eq!(
            kp.to_keypair_bytes().unwrap(),
            sk_load.to_keypair_bytes().unwrap()
        );
    }

    #[test]
    fn key_exchange_random() {
        let kp1 = X25519KeyPair::generate().unwrap();
        let kp2 = X25519KeyPair::generate().unwrap();
        assert_ne!(
            kp1.to_keypair_bytes().unwrap(),
            kp2.to_keypair_bytes().unwrap()
        );

        let xch1 = kp1.key_exchange_bytes(&kp2).unwrap();
        let xch2 = kp2.key_exchange_bytes(&kp1).unwrap();
        assert_eq!(xch1.len(), 32);
        assert_eq!(xch1, xch2);
    }

    #[test]
    fn round_trip_bytes() {
        let kp = X25519KeyPair::generate().unwrap();
        let cmp = X25519KeyPair::from_keypair_bytes(&kp.to_keypair_bytes().unwrap()).unwrap();
        assert_eq!(
            kp.to_keypair_bytes().unwrap(),
            cmp.to_keypair_bytes().unwrap()
        );
    }
}
