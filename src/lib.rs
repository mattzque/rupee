#![allow(unused_imports)]
#![allow(unused_variables)]
// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
pub mod domain;
pub mod storage;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Read;
use serde::{Serialize, Deserialize};

fn load_fixture(filename: PathBuf) -> Vec<u8> {
    let fixture = Path::new("res").join("fixtures").join(filename);
    println!("loading fixture from {:?}", fixture);
    let mut file = File::open(fixture).expect("error opening fixture file!");
    let mut contents = vec![];
    file.read_to_end(&mut contents).expect("error reading fixture file!");
    contents
}

use crate::storage::blob::backend::bucket::{BucketBlobStorage, BucketBlobStorageConfig};
use crate::storage::blob::backend::mem::{MemoryBlobStorage, MemoryBlobStorageConfig};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub storage_blob_bucket: BucketBlobStorageConfig,
}
