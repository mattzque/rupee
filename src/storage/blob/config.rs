// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License

use crate::storage::blob::backend::bucket::{BucketBlobStorageConfig};
use crate::storage::blob::backend::mem::{MemoryBlobStorageConfig};
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Deserialize)]
pub struct BlobStorageConfig {
    pub storage_blob_type: String,
    pub storage_blob_mem: MemoryBlobStorageConfig,
    pub storage_blob_bucket: BucketBlobStorageConfig,
}
