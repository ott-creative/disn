use crate::error::Result;
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::rngs::OsRng;
use rand::RngCore;

#[allow(dead_code)]
pub fn generate_rsa_keypair() -> (String, String) {
    let keypair = Rsa::generate(2048).unwrap();
    let private_pem = keypair.private_key_to_pem().unwrap();
    let private_key = base64::encode(private_pem);
    let public_pem = keypair.public_key_to_pem_pkcs1().unwrap();
    let public_key = base64::encode(public_pem);
    (private_key, public_key)
}

#[allow(dead_code)]
pub fn seal(issuer_pubkey: &str, holder_pubkey: &str, data: &str) -> Result<(String, String, String)> {
    let mut key_iv: [u8; 48] = [0; 48];
    OsRng.fill_bytes(&mut key_iv);
    let key: &[u8] = &key_iv[..32];
    let iv: &[u8] = &key_iv[32..];
    let cipher = Cipher::aes_256_cbc();
    let cipherdata = encrypt(cipher, key, Some(iv), data.as_bytes())?;
    let cipherdata = base64::encode(cipherdata);

    let issuer_key = base64::decode(issuer_pubkey)?;
    let issuer_key = Rsa::public_key_from_pem_pkcs1(&issuer_key)?;
    let mut eipherkey_issuer = vec![0; issuer_key.size() as usize];
    issuer_key.public_encrypt(&key_iv, &mut eipherkey_issuer, Padding::PKCS1)?;
    let eipherkey_issuer = base64::encode(eipherkey_issuer);

    let holder_key = base64::decode(holder_pubkey)?;
    let holder_key = Rsa::public_key_from_pem_pkcs1(&holder_key)?;
    let mut eipherkey_holder = vec![0; holder_key.size() as usize];
    holder_key.public_encrypt(&key_iv, &mut eipherkey_holder, Padding::PKCS1)?;
    let eipherkey_holder = base64::encode(eipherkey_holder);

    Ok((cipherdata, eipherkey_issuer, eipherkey_holder))
}

#[allow(dead_code)]
pub fn unseal(cipherdata: &str, eipherkey: &str, private_key: &str) -> Result<String> {
    let issuer_key = base64::decode(private_key)?;
    let privkey = Rsa::private_key_from_pem(&issuer_key)?;
    let mut decrypted = vec![0; privkey.size() as usize];
    let encrypted = base64::decode(eipherkey)?;
    privkey.private_decrypt(&encrypted, &mut decrypted, Padding::PKCS1)?;
    let key = &decrypted[..32];
    let iv = &decrypted[32..];

    let cipher = Cipher::aes_256_cbc();
    let cipherdata = base64::decode(cipherdata)?;
    let result = decrypt(cipher, key, Some(iv), &cipherdata)?;
    Ok(String::from_utf8(result)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_and_unseal() {
        let (holder_pri, holder_pub) = generate_rsa_keypair();
        let (issuer_pri, issuer_pub) = generate_rsa_keypair();

        let data = "123abcABC!@#中国";
        let (cipherdata, eipherkey_issuer, eipherkey_holder) =
            seal(&issuer_pub, &holder_pub, data).unwrap();
        let expected_data1 = unseal(&cipherdata, &eipherkey_issuer, &issuer_pri).unwrap();
        let expected_data2 = unseal(&cipherdata, &eipherkey_holder, &holder_pri).unwrap();
        assert_eq!(data, &expected_data1);
        assert_eq!(data, &expected_data2);
    }
}
