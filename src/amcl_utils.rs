extern crate amcl;
extern crate rand;

use rand::RngCore;
use rand::rngs::EntropyRng;

use self::amcl::rand::{RAND};
use self::amcl::arch::Chunk;
use self::amcl::bls381::mpin::{SHA256, hash_id};
use self::amcl::bls381::pair::{ate, fexp};
use types::{BigNum, GroupG1, GroupG2, FP12};
use constants::MODBYTES;


pub fn random_big_number(order: &[Chunk], rng: Option<EntropyRng>) -> BigNum {
    // initialise from at least 128 byte string of raw random entropy
    let entropy_size = 256;
    let mut entropy = vec![0; entropy_size];
    match rng {
        Some(mut rng) =>  rng.fill_bytes(&mut entropy.as_mut_slice()),
        None => {
            let mut rng = EntropyRng::new();
            rng.fill_bytes(&mut entropy.as_mut_slice());
        }
    }
    let mut r = RAND::new();
    r.clean();
    r.seed(entropy_size, &entropy);
    BigNum::randomnum(&BigNum::new_ints(&order), &mut r)
}


pub fn hash_on_GroupG1(msg: &[u8]) -> GroupG1 {
    let mut h: [u8; MODBYTES] = [0; MODBYTES];
    hash_id(SHA256, msg, &mut h);
    GroupG1::mapit(&h)
}

pub fn hash_on_GroupG2(msg: &[u8]) -> GroupG2 {
    let mut h: [u8; MODBYTES] = [0; MODBYTES];
    hash_id(SHA256, msg, &mut h);
    GroupG2::mapit(&h)
}

pub fn hash_as_BigNum(msg: &[u8]) -> BigNum {
    let mut h: [u8; MODBYTES] = [0; MODBYTES];
    hash_id(SHA256, msg, &mut h);
    BigNum::frombytes(&h)
}

pub fn ate_pairing(point_G2: &GroupG2, point_G1: &GroupG1) -> FP12 {
    let e = ate(&point_G2, &point_G1);
    fexp(&e)
}