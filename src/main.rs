#![allow(unused)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::cmp::Ordering;
// Includes
use std::{collections, fs};
use std::fmt::Debug;
use std::collections::HashMap;
use std::str::FromStr;

use num_bigint::{BigUint, ToBigUint};

// Testing settings 
const MitM_test_path: &'static str = "data/MitM_vars/MitM_RSA_2048_20_regular/04.txt";
const SE_test_path: &'static str = "data/SE_vars/test_SE_RSA_256_3_for_dummy_dummies/04.txt";
const E_CONST: u32 = 65537;
const L_CONST: u32 = 20;
const SE_COUNT: u32 = 3;

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

// Meet in the middle attack
fn Meet_in_the_Midle_attack_test() -> Result<(), Box<dyn std::error::Error>> {
    let e: BigUint = ToBigUint::to_biguint(&E_CONST).ok_or("Stupid e is not translatable!")?;
    let l: BigUint = ToBigUint::to_biguint(&L_CONST).ok_or("Stupid l is not translatable!")?;

    let mut test_values = read_variant(MitM_test_path)?;
    let N = test_values.remove("N").ok_or("WTF?? No 'N' in test_values for MitM??")?;
    let C = test_values.remove("C").ok_or("WTF?? No 'C' in test_values for MitM??")?;

    println!("N: {N}");
    println!("C: {C}");

    let mut X = Vec::<(BigUint, BigUint)>::new();
    for a in 1..2u32.pow(L_CONST / 2) + 1 {
        let num = ToBigUint::to_biguint(&a).unwrap();
        X.push((num.modpow(&e, &N), num));
    }

    for (S_e, S) in &X {     
        let C_S = (C.clone() * S_e.modinv(&N).unwrap()) % &N;
        for (T_e, T) in &X {
            if C_S.cmp(T_e) == Ordering::Equal {
                let M = S * T;
                print!("Message: {M:#?}");

                return Ok(());
            }
        }
    }

    println!("None found! Shieeet...");

    Ok(())
}


fn main() {
    Meet_in_the_Midle_attack_test();
}
