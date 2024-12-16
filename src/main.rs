#![allow(unused)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

// Allocation optimization
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::iter::{Product, Sum};
// Includes
use std::{collections::{BTreeMap}, fs};
use std::fmt::Debug;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::cmp::Ordering;
use std::thread;

use num_bigint::{BigUint, ToBigUint, BigInt, ToBigInt, Sign};
use num_traits::{ConstZero, Zero};
use dashmap::DashMap;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator};

// Testing settings 
const MitM_test_path: &'static str = "data/MitM_vars/bonus_MitM_RSA_2048_56_hard/04.txt";
const E_CONST: u32 = 65537;
const L_CONST: u32 = 56;
// Concurrency vars
const BLOCK_POWER: u32 = 24;
const BLOCK_SIZE: usize = 1usize << BLOCK_POWER; // 1 MB * sizeof(BigUint) ~ 2GB per table block for max RSA

// MULTI-PC PARAM
const USER_BLOCK_SHIFT: usize = 0; // PetaB - 0 (16-13: 58 + 4); Matvii - 4 (12-9: 42 + 4); Goose - 8 (8-1: 28 + 8);


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

    println!("Getting values from: '{MitM_test_path}'");
    let mut test_values = read_variant(MitM_test_path)?;
    let N = test_values.remove("N").ok_or("WTF?? No 'N' in test_values for MitM??")?;
    let C = test_values.remove("C").ok_or("WTF?? No 'C' in test_values for MitM??")?;

    println!("Meet in the Midle attack started for l = {L_CONST}");
    println!("N: {N}");
    println!("C: {C}");

    let timer = Instant::now();

    let size = 1usize << (L_CONST / 2);

    let mut X = HashMap::<BigUint, BigUint>::new();
    println!("> MitM: Started pushing at {}!", timer.elapsed().as_micros());
    let X = (1..=size).into_par_iter().map(|a| {
        let num = ToBigUint::to_biguint(&a).unwrap();
        (num.modpow(&e, &N), num)
    }).collect::<HashMap<BigUint, BigUint>>();
    println!("> MitM: Pushing finished at {}!", timer.elapsed().as_micros());

    let S_opt = X.par_iter().into_par_iter().find_any(|item| {
        let C_S = (&C * item.0.modinv(&N).unwrap()) % &N;
        X.contains_key(&C_S)
    });

    if S_opt.is_some() {
        let (S_e, S) = S_opt.unwrap();
        let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
        let M = S * X.get(&C_S).unwrap();

        println!("MitM message: {M}");
        return Ok(timer.elapsed());
    }
    
    println!("None message found for MitM! Shieeet...");

    Ok(timer.elapsed())
}

fn Meet_in_the_Midle_attack_space_compromise_test() -> Result<Duration, Box<dyn std::error::Error>> {
    let e: BigUint = ToBigUint::to_biguint(&E_CONST).ok_or("Stupid e is not translatable!")?;
    let l: BigUint = ToBigUint::to_biguint(&L_CONST).ok_or("Stupid l is not translatable!")?;

    println!("Getting values from: '{MitM_test_path}'");
    let mut test_values = read_variant(MitM_test_path)?;
    let N = test_values.remove("N").ok_or("WTF?? No 'N' in test_values for MitM??")?;
    let C = test_values.remove("C").ok_or("WTF?? No 'C' in test_values for MitM??")?;

    println!("Meet in the Midle attack started for l = {L_CONST}");
    println!("N: {N}");
    println!("C: {C}");

    let timer = Instant::now();
    let mut check: Option<(BigUint, BigUint)>;

    let blocks = 1usize << ((L_CONST / 2) - BLOCK_POWER);

    for bn_t in USER_BLOCK_SHIFT..blocks {
        println!("Start: bn_t = {bn_t}, bn_s = {bn_t} : {}", timer.elapsed().as_micros());
        // Symmetrical variant
        let shift_t_start = 1 + bn_t*BLOCK_SIZE;
        let shift_t_end = (bn_t + 1)*BLOCK_SIZE;
        let T_block = DashMap::<BigUint, BigUint>::with_capacity(BLOCK_SIZE);
        (shift_t_start..=shift_t_end).into_par_iter().for_each(|a| {
            let num = ToBigUint::to_biguint(&a).unwrap();
            T_block.insert(num.modpow(&e, &N), num);
        });
        
        println!("Generated: bn_t = {bn_t}, bn_s = {bn_t} : {}", timer.elapsed().as_micros());
        // for (S_e, S) in T_block.clone() {     
        //     let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
        //     if T_block.contains_key(&C_S) {
        //         let M = S * &*T_block.get(&C_S).unwrap();
        //         println!("MitM message: {M}");
    
        //         return Ok(timer.elapsed());
        //     }
        // }

        let T_e_values: Vec<_> = T_block.iter().map(|entry| entry.key().clone()).collect();
        let S_e_opt = T_e_values.par_iter().find_any(|S_e| {
            let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
            T_block.contains_key(&C_S)
        });

        if S_e_opt.is_some() {
            let S_e = S_e_opt.unwrap();
            let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
            let M = &*T_block.get(&S_e).unwrap() * &*T_block.get(&C_S).unwrap();

            println!("MitM message: {M}");
            return Ok(timer.elapsed());
        }

        println!("End: bn_t = {bn_t}, bn_s = {bn_t} : {}", timer.elapsed().as_micros());

        // Asymmetrical variant
        for bn_s in bn_t + 1..blocks
        {
            println!("Start: bn_t = {bn_t}, bn_s = {bn_s} : {}", timer.elapsed().as_micros());

            let shift_s_start = 1 + bn_s*BLOCK_SIZE;
            let shift_s_end = (bn_s + 1)*BLOCK_SIZE;
            let S_block = DashMap::<BigUint, BigUint>::with_capacity(BLOCK_SIZE);
            (shift_s_start..=shift_s_end).into_par_iter().for_each(|a| {
                let num = ToBigUint::to_biguint(&a).unwrap();
                S_block.insert(num.modpow(&e, &N), num);
            });
            
            let S_e_values: Vec<_> = S_block.iter().map(|entry| entry.key().clone()).collect();

            // Compare to other
            // for (S_e, S) in S_block.clone().into_iter() {     
            //     let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
            //     if T_block.contains_key(&C_S) {
            //         let M = S * &*T_block.get(&C_S).unwrap();
            //         println!("MitM message: {M}");
        
            //         return Ok(timer.elapsed());
            //     }
            // }
            let S_e_opt = S_e_values.par_iter().find_any(|S_e| {
                let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
                T_block.contains_key(&C_S)
            });
    
            if S_e_opt.is_some() {
                let S_e = S_e_opt.unwrap();
                let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
                let M = &*S_block.get(&S_e).unwrap() * &*T_block.get(&C_S).unwrap();
    
                println!("MitM message: {M}");
                return Ok(timer.elapsed());
            }



            // for (T_e, T) in T_block.clone().into_iter() {     
            //     let C_T = (&C * T_e.modinv(&N).unwrap()) % &N;
            //     if S_block.contains_key(&C_T) {
            //         let M = T * &*S_block.get(&C_T).unwrap();
            //         println!("MitM message: {M}");
        
            //         return Ok(timer.elapsed());
            //     }
            // }
            let S_e_opt = T_e_values.par_iter().find_any(|S_e| {
                let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
                S_block.contains_key(&C_S)
            });
    
            if S_e_opt.is_some() {
                let S_e = S_e_opt.unwrap();
                let C_S = (&C * S_e.modinv(&N).unwrap()) % &N;
                let M = &*T_block.get(&S_e).unwrap() * &*S_block.get(&C_S).unwrap();
    
                println!("MitM message: {M}");
                return Ok(timer.elapsed());
            }

            println!("End: bn_t = {bn_t}, bn_s = {bn_s} : {}", timer.elapsed().as_micros());
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
    println!("Getting values from: '{SE_test_path}'");
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
    // let MitM_time = Meet_in_the_Midle_attack_test().unwrap();
    // println!("'Meet in the middle' execution time: {} µs", MitM_time.as_micros());

    let MitM_time = Meet_in_the_Midle_attack_space_compromise_test().unwrap();
    println!("'Meet in the middle with space compromise' execution time: {} µs", MitM_time.as_micros());

    // println!("\n--------------------------------------------\n");

    // let SE_time = Small_Exponent_attack_test().unwrap();
    // println!("'Small exponent' execution time: {} µs", SE_time.as_micros());

    // println!("{}", 1u128 << (L_CONST / 2));
}
