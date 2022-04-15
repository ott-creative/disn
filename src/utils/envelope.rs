use openssl::rsa::{Rsa, Padding};
use openssl::symm::{decrypt, Cipher, encrypt};
use rand::RngCore;
use rand::rngs::OsRng;

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
fn seal(issuer_pubkey: &str, holder_pubkey: &str, data: &str) -> (String, String, String) {
  let mut key_iv:[u8;44] = [0;44];
  OsRng.fill_bytes(&mut key_iv);
  let key: &[u8] = &key_iv[..32];
  let iv: &[u8] = &key_iv[32..];
  let cipher = Cipher::aes_256_gcm();
  let cipherdata = encrypt(cipher, key, Some(iv), data.as_bytes()).unwrap();
  let cipherdata = base64::encode(cipherdata);

  let issuer_key = base64::decode(issuer_pubkey).unwrap();
  let issuer_key = Rsa::public_key_from_pem_pkcs1(&issuer_key).unwrap();
  let mut eipherkey_issuer = vec![0; issuer_key.size() as usize];
  issuer_key.public_encrypt(&key_iv, &mut eipherkey_issuer, Padding::PKCS1).unwrap();
  let eipherkey_issuer = base64::encode(eipherkey_issuer);

  let holder_key = base64::decode(holder_pubkey).unwrap();
  let holder_key = Rsa::public_key_from_pem_pkcs1(&holder_key).unwrap();
  let mut eipherkey_holder = vec![0; holder_key.size() as usize];
  holder_key.public_encrypt(&key_iv, &mut eipherkey_holder, Padding::PKCS1).unwrap();
  let eipherkey_holder = base64::encode(eipherkey_holder);

  (cipherdata, eipherkey_issuer, eipherkey_holder)
}

#[allow(dead_code)]
fn unseal(cipherdata: &str, eipherkey: &str, private_key: &str) -> String{
  let issuer_key = base64::decode(private_key).unwrap();
  let privkey = Rsa::private_key_from_pem(&issuer_key).unwrap();
  let mut decrypted = vec![0; privkey.size() as usize];
  let encrypted = base64::decode(eipherkey).unwrap();
  privkey.private_decrypt(&encrypted, &mut decrypted, Padding::PKCS1).unwrap();
  let key = &decrypted[..32];
  let iv = &decrypted[32..];

  let cipher = Cipher::aes_256_gcm();
  let cipherdata = base64::decode(cipherdata).unwrap();
  let result = decrypt(cipher, key, Some(iv), &cipherdata).unwrap();
  String::from_utf8(result).unwrap()
}