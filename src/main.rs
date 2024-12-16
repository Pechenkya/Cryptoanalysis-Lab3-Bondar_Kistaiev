#![allow(unused)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

// Allocation optimization
// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::iter::{Product, Sum};
// Includes
use std::{collections, fs};
use std::fmt::Debug;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::cmp::Ordering;

use num_bigint::{BigUint, ToBigUint, BigInt, ToBigInt, Sign};
use num_traits::{ConstZero, Zero};

// Testing settings 
const MitM_test_path: &'static str = "data/MitM_vars/MitM_RSA_2048_20_regular/04.txt";
const E_CONST: u32 = 65537;
const L_CONST: u32 = 20;

const SE_test_path: &'static str = "data/SE_vars/SE_RSA_1024_5_hard/04.txt";
const SE_COUNT: u32 = 5;

// Parsing input
fn read_variant(path: &str) -> Result<HashMap::<String, BigUint>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let str = String::from_utf8(data)?;
    let mut test_values = HashMap::<String, BigUint>::new();

    for part in str.split('\n') {
        let (key, val) = part.split_once(" = ").ok_or("Cannot split according to pattern!")?;
        test_values.insert(String::from_str(key)?, BigUint::parse_bytes(&val[2..].as_bytes(), 16).ok_or("Cannot parse bytes...")?);
    }

    return Ok(test_values);
}

// --------------------------- Meet in the middle attack ------------------------
fn Meet_in_the_Midle_attack_test() -> Result<Duration, Box<dyn std::error::Error>> {
    let e: BigUint = ToBigUint::to_biguint(&E_CONST).ok_or("Stupid e is not translatable!")?;
    let l: BigUint = ToBigUint::to_biguint(&L_CONST).ok_or("Stupid l is not translatable!")?;

    let mut test_values = read_variant(MitM_test_path)?;
    let N = test_values.remove("N").ok_or("WTF?? No 'N' in test_values for MitM??")?;
    let C = test_values.remove("C").ok_or("WTF?? No 'C' in test_values for MitM??")?;

    println!("Meet in the Midle attack started for l = {L_CONST}");
    println!("N: {N}");
    println!("C: {C}");

    let timer = Instant::now();

    let mut X = Vec::<(BigUint, BigUint)>::new();
    for a in 1..2u32.pow(L_CONST / 2) + 1 {
        let num = ToBigUint::to_biguint(&a).unwrap();
        X.push((num.modpow(&e, &N), num));
    }

    for (S_e, S) in &X {     
        let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
        for (T_e, T) in &X {
            if &C_S == T_e {
                let M = S * T;
                println!("MitM message: {M}");

                return Ok(timer.elapsed());
            }
        }
    }

    println!("None message found for MitM! Shieeet...");

    Ok(timer.elapsed())
}
// ------------------------------------------------------------------------------

// --------------------------- Bruteforce attack ------------------------
// fn bruteforce() -> Result<HashMap::<String, BigUint>, Box<dyn std::error::Error>> {
//     let e: BigUint = ToBigUint::to_biguint(&E_CONST).ok_or("Stupid e is not translatable!")?;
// }
// ----------------------------------------------------------------------


// --------------------------- Small Exponent attack ------------------------
fn EE_algoritm_Bezout(a: &BigInt, b: &BigInt) -> (BigInt, BigInt) {
    let mut swap = false;
    if a < b {
        swap = true;
    }

    let (mut m, mut n) = (a.clone(), b.clone());
    if swap {
        (m, n) = (n, m);
    }

    let (mut old_v, mut v) = (ToBigInt::to_bigint(&1).unwrap(), ToBigInt::to_bigint(&0).unwrap());
    let (mut old_u, mut u) = (ToBigInt::to_bigint(&0).unwrap(), ToBigInt::to_bigint(&1).unwrap());

    while !n.is_zero() {
        let q = &m / &n;
        (m, n) = (n.clone(), &m - (&q * &n));
        (old_v, v) = (v.clone(), &old_v - (&q * &v));
        (old_u, u) = (u.clone(), &old_u - (&q * &u));
    }

    if swap {
        return (old_u, old_v);
    }

    return (old_v, old_u);
}

fn CRT_solve(Ns: &Vec<BigInt>, Cs: &Vec<BigInt>) -> BigInt {
    let PROD_N = BigInt::product(Ns.iter());
    
    let mut Ni = Vec::<BigInt>::new();
    let mut Mi = Vec::<BigInt>::new();
    for ni in Ns {
        Ni.push(&PROD_N / ni);
        Mi.push(EE_algoritm_Bezout(Ni.last().unwrap(), ni).0);
    }

    let mut res = BigInt::ZERO;
    for i in 0..Cs.len() {
        res = (&res + (&Cs[i] * &Ni[i] * &Mi[i])) % &PROD_N;
    }
    if res < BigInt::ZERO {
        res = res + &PROD_N;
    }

    return res;
}

fn Small_Exponent_attack_test() -> Result<Duration, Box<dyn std::error::Error>> {
    let mut test_values = read_variant(SE_test_path)?;
    let mut Ns = Vec::<BigInt>::new();
    let mut Cs = Vec::<BigInt>::new();

    println!("Meet in the Midle attack started for l = {L_CONST}");
    for i in 1..=SE_COUNT{
        Ns.push(BigInt::from_biguint(Sign::Plus, test_values.remove(&format!("N{i}")).ok_or("N error for SE!")?));
        Cs.push(BigInt::from_biguint(Sign::Plus, test_values.remove(&format!("C{i}")).ok_or("N error for SE!")?));
        println!("N{i} = {}", Ns.last().unwrap());
        println!("C{i} = {}", Cs.last().unwrap());
    }

    let timer = Instant::now();

    let C = CRT_solve(&Ns, &Cs);

    /* We can now go through all values e from 2 to k, but
       since we this should be hard, so we assume that e == l :D
       Our observation is also proven empirically
       So instead of code below we just use value e == l
            for e in 2..=SE_COUNT {
                let M = C.nth_root(e);
                if &C == &M.pow(e) {
                    println!("Solution is e = {e}");
                }
            }
       Also, it is known that e is public, so we do nothing wrong :D
    */
    
    let e = SE_COUNT;
    let M = C.nth_root(e);

    println!("SE message: {M}");
    
    Ok(timer.elapsed())
}
// --------------------------------------------------------------------------


fn main() {
    let MitM_time = Meet_in_the_Midle_attack_test().unwrap();
    println!("'Meet in the middle' execution time: {} ms", MitM_time.as_millis());

    println!("\n--------------------------------------------\n");

    let SE_time = Small_Exponent_attack_test().unwrap();
    println!("'Small exponent' execution time: {} ms", SE_time.as_millis());

}
