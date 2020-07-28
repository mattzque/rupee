// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
use std::fmt;
use std::usize;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Binary Object Meta Data
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    // Stores a unique identifier of this blob.
    pub id: Uuid,

    // A Binary Hash like MD5 or similar (configurable).
    //checksum: Checksum,

    // Metric space embeddings of the content, pHash or similar (configurable).
    //embeddings: HashMap<String, Embedding>,

    /// The size of this binary object:
    pub size: usize,

    // Mimetype of the blob:
    //mime: Mimetype,

    // Holds information used by the storage backend to read the associated binary contents.
    //storage_args: Box<BinaryStorageMetaParams>,
}

impl BlobMeta {
    pub fn new(size: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            size,
        }
    }

    //pub fn new_from_buffer(buffer: &[u8], hash: &Hash) -> Self {
    //}
}

impl fmt::Display for BlobMeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BlobMeta<{}>", self.id.to_hyphenated())
    }
}
