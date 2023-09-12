extern crate libc;
use libc::c_uint;
use hex;
use sha2::{Sha256, Digest};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


pub struct NexaSecp256K1POW {
    ctx: *mut secp256k1_context,
}

impl NexaSecp256K1POW {
    pub fn new(flags: c_uint) -> Result<NexaSecp256K1POW, &'static str> {
        let ctx = unsafe { secp256k1_context_create(flags) };
        if ctx.is_null() {
            Err("Failed to create secp256k1 context")
        } else {
            Ok(NexaSecp256K1POW { ctx })
        }
    }

    pub fn schnorr_sign(
        &self,
        msg: &[u8; 32],
        seckey: &[u8; 32],
    ) -> Result<[u8; 64], &'static str> {
        let mut signature = [0u8; 64];

        let res = unsafe {
            secp256k1_schnorr_sign(
                self.ctx,
                signature.as_mut_ptr(),
                msg.as_ptr(),
                seckey.as_ptr(),
                secp256k1_nonce_function_rfc6979,
                std::ptr::null()
            )
        };

        if res == 1 {
            Ok(signature)
        } else {
            Err("Failed to create Schnorr signature")
        }
    }
}

impl Drop for NexaSecp256K1POW {
    fn drop(&mut self) {
        // Destroy the secp256k1 context
        // Consult the secp256k1 documentation for the correct function to call
        unsafe { secp256k1_context_destroy(self.ctx) };
    }
}

fn pow(ctx: NexaSecp256K1POW, hash_and_nonce: String) -> Result<[u8; 32], &'static str> {
    let mut sha = Sha256::default();
    
    sha.update(hex::decode(hash_and_nonce).unwrap());
    let hash1=sha.finalize_reset();
    
    sha.update(hash1);
    let hash2=sha.finalize_reset();

    sha.update(hash2);
    let h1=sha.finalize_reset();

    let h1_array: [u8; 32] = h1.into();
    let hash2_array: [u8; 32] = hash2.into();
    
    let mut sig = [0u8; 64];
    let sig = ctx.schnorr_sign(&h1_array,&hash2_array)?;


    sha.update(sig.as_ref());
    let pow = sha.finalize_reset();
    
    let mut ss = [0u8;32];
    ss.copy_from_slice(&pow[..32]);
    Ok(ss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let flags: c_uint = SECP256K1_CONTEXT_SIGN;
        match NexaSecp256K1POW::new(flags) {
            Ok(_ctx) => assert!(true, "Context created successfully"),
            Err(e) => panic!("POW failed: {}", e),
        }
    }

    #[test]
    fn test_pow() {
        let flags: c_uint = SECP256K1_CONTEXT_SIGN;
        let ctx = NexaSecp256K1POW::new(flags).unwrap();
        let hash_and_nonce = "ae7c5ac35c375d08306524c3933bb77c37cf00d406aa1749f65648ce79cd09611dc4caa796cbc4b8e79ec200".to_string();

        match pow(ctx, hash_and_nonce) {
            Ok(result) => {
                let result_hex = hex::encode(result);
                let expected_hex = "B53281E3C15F0ACCA74CF16EEAAF8E6907976F365428C457220D17328791DB1B";
                assert_eq!(result_hex.to_uppercase(), expected_hex, "POW did not produce the expected result");
            },
            Err(e) => panic!("POW failed: {}", e),
        }
    }
}