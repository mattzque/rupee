// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
//! Memory Blob Storage Backend Implementation
//!
//! This is storing binary objects in-memory, useful in tests.
//!
use crate::domain::meta::BlobMeta;
use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::usize;
use serde::{Serialize, Deserialize};
use typetag::serde;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryBlobRef {
    pub index: usize,
}

#[typetag::serde]
impl BlobRef for MemoryBlobRef {
    fn any(&self) -> &dyn Any {
        self
    }

    fn display(&self) -> String {
        format!("MemoryBlobRef({})", self.index)
    }

    fn clone_as_dyn_blob_ref(&self) -> Box<dyn BlobRef + 'static> {
        Box::new(self.clone())
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct MemoryBlobStorageConfig {}

pub struct MemoryBlobStorage {
    store: Vec<Vec<u8>>,
}

impl MemoryBlobStorage {
    pub fn init(config: &MemoryBlobStorageConfig) -> Result<(), BlobStorageError> {
        Ok(())
    }

    pub fn new(config: MemoryBlobStorageConfig) -> Result<Self, BlobStorageError> {
        Ok(Self { store: Vec::new() })
    }
}

impl BlobStorage for MemoryBlobStorage {
    fn get(&self, meta: &BlobMeta, blob_ref: &Box<dyn BlobRef>) -> Result<Vec<u8>, BlobStorageError> {
        if let Some(blob_ref) = blob_ref.any().downcast_ref::<MemoryBlobRef>() {
            let index = blob_ref.index;
            if index >= self.store.len() {
                Err(BlobStorageError::ReadStorageError)
            } else {
                Ok(self.store[index].clone())
            }
        } else {
            Err(BlobStorageError::ReadBlobRefMismatch)
        }
    }

    fn put(
        &mut self,
        meta: &BlobMeta,
        buffer: Vec<u8>,
    ) -> Result<Box<dyn BlobRef>, BlobStorageError> {
        let index: usize = self.store.len();
        let blob_ref = MemoryBlobRef { index };
        self.store.push(buffer);
        Ok(Box::new(blob_ref))
    }

    fn delete(
        &mut self,
        meta: &BlobMeta,
        blob_ref: &Box<dyn BlobRef>,
    ) -> Result<(), BlobStorageError> {
        Ok(())  // no-op
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::blob::BlobStorage;
    use crate::domain::meta::BlobMeta;
    use crate::{load_fixture};
    use crate::storage::blob::backend::mem::{MemoryBlobStorage, MemoryBlobStorageConfig};
    use std::path::{Path, PathBuf};

    #[test]
    fn test_blob_mem() {
        let mut storage = MemoryBlobStorage::new(MemoryBlobStorageConfig {})
            .expect("mem blob backend can't be created!");

        let buffer = load_fixture(Path::new("images").join("rgb.jpeg"));
        let meta = BlobMeta::new(buffer.len());

        let reference = storage
            .put(&meta, buffer.to_vec())
            .expect("put blob in mem storage failed!");

        // get burrow and copy buffer
        let buf = storage
            .get(&meta, &reference)
            .expect("getting blob from the mem storage failed!").to_vec();

        // no-op
        storage
            .delete(&meta, &reference)
            .expect("deleting blob from the mem storage failed!");

        assert_eq!(buf.len(), buffer.len());
        assert_eq!(buf, buffer);
    }
}
