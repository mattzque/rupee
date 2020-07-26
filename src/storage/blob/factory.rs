// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
use crate::domain::meta::BlobMeta;
use crate::storage::blob::{BlobStorage, BlobStorageError};
use crate::storage::blob::config::{BlobStorageConfig};
use crate::storage::blob::backend::bucket::{BucketBlobStorage, BucketBlobStorageConfig};
use crate::storage::blob::backend::mem::{MemoryBlobStorage, MemoryBlobStorageConfig};
use std::any::Any;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;


pub fn create_blob_storage(config: BlobStorageConfig) -> Result<Box<dyn BlobStorage>, BlobStorageError> {
    match config.storage_blob_type.as_ref() {
        "mem" => {
            Ok(Box::new(MemoryBlobStorage::new(config.storage_blob_mem)?))
        }
        "bucket" => {
            Ok(Box::new(BucketBlobStorage::new(config.storage_blob_bucket)?))
        }
        _ => Err(BlobStorageError::UnknownBackendError),
    }
}
