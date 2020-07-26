// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
pub mod backend;
pub mod hashing;
pub mod config;
pub mod factory;
use crate::domain::meta::BlobMeta;
use backend::bucket::{BucketBlobStorage, BucketBlobStorageConfig};
use backend::mem::{MemoryBlobStorage, MemoryBlobStorageConfig};
use std::any::Any;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub enum BlobStorageError {
    UnknownBackendError,
    /// Error with the given storage backend configuration (wrong config passed to backend?).
    StorageConfigError,
    ReadBlobRefMismatch,
    CreateStorageError(&'static str),
    ReadStorageError,
    WriteError,
    PutError,
    DeleteError,
    IOError,
}

impl From<std::io::Error> for BlobStorageError {
    fn from(error: std::io::Error) -> Self {
        BlobStorageError::IOError
    }
}

/// Blob References are used to reference previously stored blobs.
pub trait BlobRef {
    /// Returns the Any trait of the reference for downcasting to concrete types in backends.
    fn any(&self) -> &dyn Any;

    /// Helper method for debug/display.
    fn display(&self) -> String;
}

impl fmt::Display for dyn BlobRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Trait all storage backends need to implement.
pub trait BlobStorage {
    /// Reads some binary data from the storage.
    fn get(&self, meta: &BlobMeta, blob_ref: &Box<dyn BlobRef>) -> Result<Vec<u8>, BlobStorageError>;

    /// Persists some binary data into the storage.
    fn put(
        &mut self,
        meta: &BlobMeta,
        buffer: Vec<u8>,
    ) -> Result<Box<dyn BlobRef>, BlobStorageError>;

    /// Delete the associated binary data in the storage.
    fn delete(
        &mut self,
        meta: &BlobMeta,
        blob_ref: &Box<dyn BlobRef>,
    ) -> Result<(), BlobStorageError>;
}
