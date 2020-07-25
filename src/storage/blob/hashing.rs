// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
extern crate blake2;
extern crate fasthash;
extern crate hex;
extern crate sha2;
extern crate sha3;
extern crate whirlpool;
use blake2::{Blake2b, Blake2s, Digest as BlakeDigest};
use fasthash::t1ha;
use sha2::{Digest as Sha2Digest, Sha256, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use std::str::FromStr;
use whirlpool::{Digest as WhirlpoolDigest, Whirlpool};

#[derive(Debug)]
enum HashError {
    HashTypeParseError,
}

#[derive(Debug)]
enum Hash {
    /// SHA-2 256
    Sha2_256,

    /// SHA-2 512
    Sha2_512,

    /// SHA-3 224
    Sha3_224,

    /// SHA-3 256
    Sha3_256,

    /// SHA-3 384
    Sha3_384,

    /// SHA-3 512
    Sha3_512,

    /// t1ha (8bytes)
    T1ha,

    /// Blake-2s (32bytes)
    Blake2s,

    /// Blake-2b (64bytes)
    Blake2b,

    /// Whirlpool (64bytes)
    Whirlpool,
}

impl Hash {
    fn hash_bytes(&self, bytes: &[u8]) -> Vec<u8> {
        match *self {
            Hash::Sha2_256 => {
                let mut h = Sha256::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Sha2_512 => {
                let mut h = Sha512::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Sha3_224 => {
                let mut h = Sha3_224::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Sha3_256 => {
                let mut h = Sha3_256::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Sha3_384 => {
                let mut h = Sha3_384::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Sha3_512 => {
                let mut h = Sha3_512::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::T1ha => t1ha::hash64(bytes).to_be_bytes().to_vec(),
            Hash::Blake2s => {
                let mut h = Blake2s::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Blake2b => {
                let mut h = Blake2b::new();
                h.input(bytes);
                h.result().to_vec()
            }
            Hash::Whirlpool => {
                let mut h = Whirlpool::new();
                h.input(bytes);
                h.result().to_vec()
            }
        }
    }
}

impl FromStr for Hash {
    type Err = HashError;

    fn from_str(s: &str) -> Result<Hash, HashError> {
        match s {
            "sha2_256" => Ok(Hash::Sha2_256),
            "sha2_512" => Ok(Hash::Sha2_512),
            "sha3_224" => Ok(Hash::Sha3_224),
            "sha3_256" => Ok(Hash::Sha3_256),
            "sha3_384" => Ok(Hash::Sha3_384),
            "sha3_512" => Ok(Hash::Sha3_512),
            "t1ha" => Ok(Hash::T1ha),
            "Blake2s" => Ok(Hash::Blake2s),
            "Blake2b" => Ok(Hash::Blake2b),
            "whirlpool" => Ok(Hash::Whirlpool),
            _ => Err(HashError::HashTypeParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Hash;
    use hex;
    use std::concat;

    #[test]
    fn test_hash_sha2_256() {
        let hasher = Hash::Sha2_256;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81";
        assert_eq!(result.len(), 32);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_sha2_512() {
        let hasher = Hash::Sha2_512;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = concat!(
            "27864cc5219a951a7a6e52b8c8dddf6981d098da1658d96258c870b2c88dfbcb",
            "51841aea172a28bafa6a79731165584677066045c959ed0f9929688d04defc29"
        );
        assert_eq!(result.len(), 64);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_sha3_224() {
        let hasher = Hash::Sha3_224;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = "a83f2a82afecf04807fa166fc3d618b795c1543424714090c7cc5a56";
        assert_eq!(result.len(), 28);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_sha3_256() {
        let hasher = Hash::Sha3_256;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = "fd1780a6fc9ee0dab26ceb4b3941ab03e66ccd970d1db91612c66df4515b0a0a";
        assert_eq!(result.len(), 32);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_sha3_384() {
        let hasher = Hash::Sha3_384;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = concat!(
            "854ec9031d7e34de34973dd41c3dd156a2cde981fff0153281b87d72b481d98c",
            "dfc0a9791d522d719e607656d7655b1a"
        );
        assert_eq!(result.len(), 48);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_sha3_512() {
        let hasher = Hash::Sha3_512;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = concat!(
            "0c60ae04fbb17fe36f4e84631a5b8f3cd6d0cd46e80056bdfec97fd305f764da",
            "adef8ae1adc89b203043d7e2af1fb341df0ce5f66dfe3204ec3a9831532a8e4c"
        );
        assert_eq!(result.len(), 64);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_t1ha() {
        let hasher = Hash::T1ha;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = "0ceafaa956060ffa";
        assert_eq!(result.len(), 8);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_blake2b() {
        let hasher = Hash::Blake2b;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = concat!(
            "cf94f6d605657e90c543b0c919070cdaaf7209c5e1ea58acb8f3568fa2114268",
            "dc9ac3bafe12af277d286fce7dc59b7c0c348973c4e9dacbe79485e56ac2a702"
        );
        assert_eq!(result.len(), 64);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_blake2s() {
        let hasher = Hash::Blake2s;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = "5f76d39b042c1b6000041e7ba3746c8f1d4707425c571cebd70cab0f87317eff";
        assert_eq!(result.len(), 32);
        assert_eq!(hex::encode(&result), expected);
    }

    #[test]
    fn test_hash_whirlpool() {
        let hasher = Hash::Whirlpool;
        let result = hasher.hash_bytes(&vec![1, 2, 3]);
        let expected = concat!(
            "99cc2c37d3cc0a00b06640462b8e68253b2e981c8ab38ee3017f0abf2436839c",
            "bac31c2c4f123bdb3e088c65cd13e4ab2ea75f8dd204d64afe0c5c4b53d07754"
        );
        assert_eq!(result.len(), 64);
        assert_eq!(hex::encode(&result), expected);
    }
}
