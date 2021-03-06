extern crate amcl;

use super::amcl_utils::{hash_on_GroupG1, ate_pairing};
use super::types::GroupG1;
use super::constants::{CURVE_ORDER, GeneratorG2, GroupG2_SIZE};
use super::common::{SigKey, VerKey, Keypair};
use bls::errors::SerzDeserzError;
use bls::amcl_utils::get_G1_point_from_bytes;
use bls::amcl_utils::get_bytes_for_G1_point;

pub struct Signature {
    pub point: GroupG1,
}

impl Clone for Signature {
    fn clone(&self) -> Signature {
        let mut temp_s = GroupG1::new();
        temp_s.copy(&self.point);
        Signature {
            point: temp_s
        }
    }
}

impl Signature {
    // Signature = H_0(msg) * sk
    pub fn new(msg: &[u8], sig_key: &SigKey) -> Self {
        let hash_point = hash_on_GroupG1(msg);
        let sig = hash_point.mul(&sig_key.x);
        // This is different from the paper, the other exponentiation happens in aggregation.
        // This avoids the signer to know beforehand of all other participants
        Signature { point: sig }
    }

    pub fn verify(&self, msg: &[u8], ver_key: &VerKey) -> bool {
        // TODO: Check if point exists on curve, maybe use `ECP::new_big` and x cord of verkey
        if self.point.is_infinity() {
            println!("Signature point at infinity");
            return false;
        }
        let msg_hash_point = hash_on_GroupG1(msg);
        let mut lhs = ate_pairing(&GeneratorG2, &self.point);
        let mut rhs = ate_pairing(&ver_key.point, &msg_hash_point);
        /*let mut lhs_bytes: [u8; FP12_SIZE] = [0; FP12_SIZE];
        let mut rhs_bytes: [u8; FP12_SIZE] = [0; FP12_SIZE];
        lhs.tobytes(&mut lhs_bytes);
        rhs.tobytes(&mut rhs_bytes);
        lhs_bytes.to_vec() == rhs_bytes.to_vec()*/
        lhs.equals(&mut rhs)
    }

    pub fn from_bytes(sig_bytes: &[u8]) -> Result<Signature, SerzDeserzError> {
        Ok(Signature {
            point: get_G1_point_from_bytes(sig_bytes)?
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        get_bytes_for_G1_point(&self.point)
    }
}

/*impl CurvePoint for Signature {
    fn is_valid_point(&self) -> bool {
        if self.point.is_infinity() {
            return false;
        }
        true
    }
}*/

#[cfg(test)]
mod tests {
    // TODO: Add tests for failure
    // TODO: Add more test vectors
    use super::*;

    #[test]
    fn sign_verify() {
        let keypair = Keypair::new(None);
        let sk = keypair.sig_key;
        let vk = keypair.ver_key;

        let msg = "Small msg";
        let msg1 = "121220888888822111212";
        let msg2 = "Some message to sign";
        let msg3 = "Some message to sign, making it bigger, ......, still bigger........................, not some entropy, hu2jnnddsssiu8921n ckhddss2222";
        for m in vec![msg, msg1, msg2, msg3] {
            let b = m.as_bytes();
            let mut sig = Signature::new(&b, &sk);
            assert!(sig.verify(&b, &vk));

            let bs = sig.to_bytes();
            let mut sig1 = Signature::from_bytes(&bs).unwrap();
            assert_eq!(&sig.point.tostring(), &sig1.point.tostring());
        }
    }

    #[test]
    fn verification_failure() {
        let keypair = Keypair::new(None);
        let sk = keypair.sig_key;
        let vk = keypair.ver_key;

        let mut msg = "Some msg";
        let sig = Signature::new(&msg.as_bytes(), &sk);
        msg = "Other msg";
        assert_eq!(sig.verify(&msg.as_bytes(), &vk), false);
        msg = "";
        assert_eq!(sig.verify(&msg.as_bytes(), &vk), false);
    }

    #[test]
    fn signature_at_infinity() {
        let keypair = Keypair::new(None);
        let vk = keypair.ver_key;

        let msg = "Small msg".as_bytes();
        let mut sig = Signature { point: GroupG1::new() };
        sig.point.inf();
        assert_eq!(sig.verify(&msg, &vk), false);
    }
}
