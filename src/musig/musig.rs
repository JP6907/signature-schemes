extern crate amcl;
extern crate rand;

use rand::rngs::EntropyRng;
use super::amcl_utils::{random_big_number, hash_as_BigNum};
use super::types::{BigNum, GroupG};
use super::constants::{CURVE_ORDER, GeneratorG, GroupG_Size, MODBYTES, CurveOrder};

pub struct SigKey {
    pub x: BigNum
}

impl SigKey {
    pub fn new(rng: Option<EntropyRng>) -> Self {
        SigKey {
            x: random_big_number(&CURVE_ORDER, rng),
        }
    }
}

pub struct VerKey {
    pub point: GroupG
}

impl Clone for VerKey {
    fn clone(&self) -> VerKey {
        let mut temp_v = GroupG::new();
        temp_v.copy(&self.point);
        VerKey {
            point: temp_v
        }
    }
}

impl VerKey {
    pub fn from_sigkey(sk: &SigKey) -> Self {
        VerKey {
            point: GeneratorG.mul(&sk.x),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vk_clone = self.clone();
        let mut vk_bytes: [u8; GroupG_Size] = [0; GroupG_Size];
        vk_clone.point.tobytes(&mut vk_bytes, false);
        vk_bytes.to_vec()
    }
}

pub struct Keypair {
    pub sig_key: SigKey,
    pub ver_key: VerKey
}

impl Keypair {
    pub fn new(rng: Option<EntropyRng>) -> Self {
        let sk = SigKey::new(rng);
        let vk = VerKey::from_sigkey(&sk);
        Keypair { sig_key: sk, ver_key: vk }
    }
}

pub struct Nonce {
    pub x: Option<BigNum>,
    pub point: GroupG
}

impl Clone for Nonce {
    fn clone(&self) -> Nonce {
        let mut temp_v = GroupG::new();
        temp_v.copy(&self.point);
        Nonce {
            x: self.x.clone(),
            point: temp_v
        }
    }
}

impl Nonce {
    pub fn new(rng: Option<EntropyRng>) -> Self {
        let x = random_big_number(&CURVE_ORDER, rng);
        Nonce {
            x: Some(x),
            point: GeneratorG.mul(&x),
        }
    }

    pub fn aggregate(nonces: &Vec<Nonce>) -> Nonce {
        let mut an: GroupG = GroupG::new();
        an.inf();
        for n in nonces {
            an.add(&n.point);
        }
        Nonce {
            x: None,
            point: an,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut clone = self.clone();
        let mut nonce_bytes: [u8; GroupG_Size] = [0; GroupG_Size];
        clone.point.tobytes(&mut nonce_bytes, false);
        nonce_bytes.to_vec()
    }
}


pub struct HashedVerKeys {
    pub b: [u8; MODBYTES]
}

impl HashedVerKeys {
    pub fn new(verkeys: &Vec<VerKey>) -> HashedVerKeys {
        let mut bytes: Vec<u8> = vec![];
        for vk in verkeys {
            bytes.extend(vk.to_bytes());
        }
        let mut n = hash_as_BigNum(&bytes);
        let mut b: [u8; MODBYTES] = [0; MODBYTES];
        n.tobytes(&mut b);
        HashedVerKeys {
            b
        }
    }

    pub fn hash_with_verkey(&self, verkey: &VerKey) -> BigNum {
        let mut bytes: Vec<u8> = vec![];
        bytes.extend(self.b.iter());
        bytes.extend(verkey.to_bytes());
        BigNum::frombytes(&bytes)
    }
}

pub struct AggregatedVerKey {
    pub point: GroupG
}

impl Clone for AggregatedVerKey {
    fn clone(&self) -> AggregatedVerKey {
        let mut temp_v = GroupG::new();
        temp_v.copy(&self.point);
        AggregatedVerKey {
            point: temp_v
        }
    }
}

impl AggregatedVerKey {
    pub fn new(verkeys: &Vec<VerKey>) -> AggregatedVerKey {
        let L = HashedVerKeys::new(verkeys);
        let mut avk: GroupG = GroupG::new();
        avk.inf();
        for vk in verkeys {
            let point = vk.point.mul(&L.hash_with_verkey(vk));
            avk.add(&point);
        }
        AggregatedVerKey {
            point: avk,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vk_clone = self.clone();
        let mut vk_bytes: [u8; GroupG_Size] = [0; GroupG_Size];
        vk_clone.point.tobytes(&mut vk_bytes, false);
        vk_bytes.to_vec()
    }
}

pub struct Signature {
    pub x: BigNum
}

impl Signature {
    // Signature =
    pub fn new(msg: &[u8], sig_key: &SigKey, nonce: &BigNum, verkey: &VerKey,
               all_nonces: &Vec<Nonce>, all_verkeys: &Vec<VerKey>) -> Self {
        let R = Nonce::aggregate(all_nonces);
        let L = HashedVerKeys::new(all_verkeys);
        let avk = AggregatedVerKey::new(all_verkeys);

        Signature::new_using_aggregated_objs(msg, sig_key, nonce, verkey, &R, &L, &avk)
    }

    pub fn new_using_aggregated_objs(msg: &[u8], sig_key: &SigKey, nonce: &BigNum, verkey: &VerKey,
                                     R: &Nonce, L: &HashedVerKeys, avk: &AggregatedVerKey) -> Self {
        let mut h = L.hash_with_verkey(&verkey);

        let mut challenge = Signature::compute_challenge(msg, &avk.to_bytes(), &R.to_bytes());

        let mut product = BigNum::modmul(&mut challenge, &mut h, &CurveOrder);
        let mut product = BigNum::modmul(&mut product, &mut sig_key.x.clone(), &CurveOrder);

        product.add(nonce);

        Signature { x: product }
    }

    pub fn compute_challenge(msg: &[u8], aggr_verkey: &[u8], aggr_nonce: &[u8]) -> BigNum {
        let mut challenge_bytes: Vec<u8> = vec![];
        challenge_bytes.extend(aggr_verkey);
        challenge_bytes.extend(aggr_nonce);
        challenge_bytes.extend(msg);
        hash_as_BigNum(&challenge_bytes)
    }
}


pub struct AggregatedSignature {
    pub x: BigNum
}

impl AggregatedSignature {
    pub fn new(signatures: &[Signature]) -> Self {
        let mut aggr_sig: BigNum = BigNum::new();
        for sig in signatures {
            aggr_sig.add(&sig.x);
        }
        AggregatedSignature {
            x: aggr_sig
        }
    }

    pub fn verify(&self, msg: &[u8], nonces: &Vec<Nonce>, ver_keys: &Vec<VerKey>) -> bool {
        let R = Nonce::aggregate(nonces);
        let avk = AggregatedVerKey::new(ver_keys);
        self.verify_using_aggregated_objs(msg, &R, &avk)
    }

    pub fn verify_using_aggregated_objs(&self, msg: &[u8], R: &Nonce, avk: &AggregatedVerKey) -> bool {
        let challenge = Signature::compute_challenge(msg, &avk.to_bytes(),
                                                     &R.to_bytes());
        let mut lhs = GeneratorG.mul(&self.x);
        let mut rhs = GroupG::new();
        rhs.inf();
        rhs.add(&R.point);
        rhs.add(&avk.point.mul(&challenge));
        rhs.equals(&mut lhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggr_sign_verify() {
        let keypairs: Vec<Keypair> = (0..5).map(|_| Keypair::new(None)).collect();
        let verkeys: Vec<VerKey> = keypairs.iter().map(|k| k.ver_key.clone()).collect();

        let msgs = vec![
            "Small msg",
            "121220888888822111212",
            "Some message to sign",
            "Some message to sign, making it bigger, ......, still bigger........................, not some entropy, hu2jnnddsssiu8921n ckhddss2222",
            " is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
        ];

        for msg in msgs {
            let nonces: Vec<Nonce> = (0..5).map(|_| Nonce::new(None)).collect();
            let mut signatures: Vec<Signature> = vec![];
            let msg_b = msg.as_bytes();
            for i in 0..5 {
                let keypair = &keypairs[i];
                signatures.push(Signature::new(msg_b, &keypair.sig_key, &nonces[i].x.unwrap(), &keypair.ver_key, &nonces, &verkeys));
            }
            let aggr_sig: AggregatedSignature = AggregatedSignature::new(&signatures);
            assert!(aggr_sig.verify(msg_b, &nonces, &verkeys));
            let aggr_nonce = Nonce::aggregate(&nonces);
        }
    }
}