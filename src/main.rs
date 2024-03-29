// vigenere cipher breaking fun for Coursera Cryptography class
extern crate rand;
use rand::distributions::{Distribution, Uniform};

use std::env;
use std::fs;
use std::collections::HashMap;

mod crypto;
use crypto::decrypt;

mod errors;
use errors::ParseError;

// Load and parse a text file containing printed hex of a payload.
// That is: "FF0A" -> [255, 10]
fn parse(file: &str) -> Result<Vec<u8>, ParseError> {
    let s = fs::read_to_string(file)?;
    let bytes = s.as_bytes();

    let mut result: Vec<u8> = Vec::with_capacity((bytes.len() / 2) + 1);

    for tuple in bytes.chunks(2) {
        if tuple.len() > 1 {
            let chunk = std::str::from_utf8(tuple)?;
            let int = u8::from_str_radix(chunk, 16)?;
            result.push(int);
        }
    }
    Ok(result)
}


/// Sample a Vector, creating a new subset of every n-th element
fn stripe<T: Copy>(v: &Vec<T>, n: usize, offset: usize) -> Vec<T> {
    // Trivial case 1: keylen is really 1 or 0
    if n < 2 {
        return v.clone();
    }

    // Trivial case 2: keylen is greater than ciphertext
    if v.len() > 0 && n >= v.len() {
        return vec![v[0]];
    }

    let adj_offset = (n + offset) % n;

    let mut result: Vec<T> = Vec::new();
    let idx: Vec<usize> = (0..v.len())
        .skip(adj_offset)
        .filter(|i| i % n == adj_offset)
        .collect();

    for i in idx {
        result.push(v[i]);
    }

    result
}

/// Generate a distribution of the frequency of item occurrence
/// in a given Vector.
fn freq_of_vec<T>(v: &Vec<T>) -> HashMap<&T, f32>
where T: std::hash::Hash + std::cmp::Eq {
    let n = v.len();
    let mut result = HashMap::new();

    for x in v {
        if result.contains_key(x) {
            result.insert(x, result.get(x).unwrap() + 1.0);
        } else {
            result.insert(x, 1.0);
        }
    }

    for (_, val) in result.iter_mut() {
        *val = *val / n as f32;
    }

    result
}

/// Analyze ciphertext as a Vector of bytes and use an error
/// measurement of the distribution as a way to identify possible
/// key lengths for the Vigenere cipher
fn guess_keylen(ciphertext: Vec<u8>) {
    for i in 1..32 {
        let s = stripe(&ciphertext, i, 0);
        let f = freq_of_vec(&s);

        let num_keys = f.keys().len();
        let sum: f32 = f.values().fold(0_f32, |acc, v| acc + (v * v));

        if sum > 0.035 { // i.e. looks to be non-uniform distribution
            println!("{:?}: {:?} w/ {} unique vals", i, sum, num_keys);
        }
    }
}


/// Try to guess viable key components for decoding ciphertext
fn guess_key_part(ciphertext: &Vec<u8>, keylen: usize, offset: usize) -> HashMap<u8, f32> {
    let mut map = HashMap::new();
    let guess_range = 0x0..0xFF; // turns out the key can be any arbitrary bytes

    let c = stripe(ciphertext, keylen, offset);

    for g in guess_range {
        let p: Vec<u8> = c.clone().into_iter().map(|x| x ^ g).collect();

        let e_cnt = p.iter().filter(|&&x| x == 0x65).count() as f32;
        let e = e_cnt / c.len() as f32;

        // assumption right now is based on my manual analysis of Moby Dick,
        // we should expect lowercase 'e' to appear about 9% of the time.
        if e > 0.05 {
            map.insert(g, e);
        }
    }

    map
}

fn bruteforce(ciphertext: &Vec<u8>, keyspace: &Vec<Vec<u8>>) {
    let dimensions: Vec<usize> = keyspace.iter()
        .map(|v| v.len())
        .collect();
    let max: usize = dimensions.iter().fold(1, |acc, x| acc * x);

    // ok, for now let's try some random search of the space
    // we'll cap iterations at max just to be safe and lazy
    let mut rng = rand::thread_rng();
    let mut dice = vec![];
    dimensions.iter().for_each(|&d| dice.push(Uniform::from(0..d)));

    println!("Hold onto your butts...going to try {}k iterations...", max / 1_000);
    for _ in 0..max {
        let mut key: Vec<u8> = Vec::with_capacity(keyspace.len());

        for j in 0..keyspace.len() {
            let roll = dice[j].sample(&mut rng);
            key.push(keyspace[j][roll]);
        }

//        println!("{:x?}", key);
        let candidate = decrypt(ciphertext, &key);
        if candidate.is_ok() {
            let plaintext = candidate.unwrap();
            //println!("key[{:x?}] ==> {}", key, String::from_utf8(plaintext).unwrap());

            let s = String::from_utf8(plaintext).unwrap();
            if s.contains(" the ") && s.contains(" and ") {
                println!("key[{:x?}] ==> {}", key, s);
            }
        }
    }
}

fn usage(cmd: Option<&str>) {
    match cmd {
        Some("key") => println!("Usage: vigenere key [ciphertext file]"),
        Some("crack") => eprintln!("Usage: vigenere crack [keylen] [ciphertext file]"),
        _ => {
            println!("Usage: vigenere [command] [options...]");
            println!("");
            println!("Supported commands:");
            println!("\tkey\t\tbrute-force guess Vigenere cipher key length");
            println!("\tcrack\t\tgiven a key length, derive most likely keys");
        },
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return usage(None);
    }

    match args[1].as_str() {
        "key" => {
            if args.len() < 3 {
                return usage(Some("key"));
            }

            let file = &args[2];
            match parse(file) {
                Ok(data) => guess_keylen(data),
                Err(e) => println!("ParseError: {}", e),
            }
        },
        "crack" => {
            if args.len() < 4 {
                return usage(Some("crack"));
            }

            let keylen = usize::from_str_radix(&args[2], 10).unwrap();
            let file = &args[3];

            match parse(file) {
                Ok(data) => {
                    let mut probables = vec![];

                    // For now for testing we only try cracking 1st char
                    for i in 0..keylen {
                        let r = guess_key_part(&data, keylen, i);
                        println!("key[{}] ==> {:x?}", i, r);

                        if r.len() == 0 {
                            eprintln!("failed to find a viable key byte, aborting :-(");
                            return;
                        }

                        // so is there anyway to clone key values from a hashmap??? ugh
                        let mut guesses = vec![];
                        for (k, _) in r {
                            guesses.push(k);
                        }
                        probables.push(guesses);
                    }

                    bruteforce(&data, &probables);
                },
                Err(e) => println!("ParseError: {}", e),
            }
        },
        _ => usage(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripe_identity_cases() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(stripe(&v, 0, 0), v);
        assert_eq!(stripe(&v, 1, 0), v);
    }

    #[test]
    fn stripe_nth_members() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(stripe(&v, 2, 0), [1, 3, 5]);
        assert_eq!(stripe(&v, 2, 1), [2, 4]);
        assert_eq!(stripe(&v, 2, 2), [1, 3, 5]);

        assert_eq!(stripe(&v, 3, 0), [1, 4]);
        assert_eq!(stripe(&v, 4, 0), [1, 5]);
        assert_eq!(stripe(&v, 5, 0), [1]);
        assert_eq!(stripe(&v, 6, 0), [1]);
    }

    #[test]
    fn freq_of_vec_test() {
        let v = vec![1, 2, 2, 3, 3, 3, 4, 4, 4, 4];
        let m = freq_of_vec(&v);

        assert!(m.contains_key(&1));
        assert!(m.contains_key(&2));
        assert!(m.contains_key(&3));
        assert!(m.contains_key(&4));

        assert_eq!(m.get(&1).unwrap(), &0.1_f32);
        assert_eq!(m.get(&2).unwrap(), &0.2_f32);
        assert_eq!(m.get(&3).unwrap(), &0.3_f32);
        assert_eq!(m.get(&4).unwrap(), &0.4_f32);
    }
}
