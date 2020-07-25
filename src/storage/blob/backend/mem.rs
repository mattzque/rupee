// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
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

pub struct MemoryBlobRef {
    index: usize,
}

impl BlobRef for MemoryBlobRef {
    fn any(&self) -> &dyn Any {
        self
    }

    fn display(&self) -> String {
        format!("MemoryBlobRef({})", self.index)
    }
}

#[derive(Debug, Clone)]
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
    fn get(&self, meta: &BlobMeta, blob_ref: &Box<dyn BlobRef>) -> Result<&[u8], BlobStorageError> {
        if let Some(blob_ref) = blob_ref.any().downcast_ref::<MemoryBlobRef>() {
            let index = blob_ref.index;
            if index >= self.store.len() {
                Err(BlobStorageError::ReadStorageError)
            } else {
                Ok(&self.store[index])
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::meta::BlobMeta;
    use crate::storage::blob::create_blob_storage;

    #[test]
    fn test_blob_mem() {
        let mut storage = create_blob_storage("mem").expect("mem blob backend can't be created!");

        let buffer: Vec<u8> = vec![0, 42, 0];
        let meta = BlobMeta::new(buffer.len());

        assert_eq!(buffer.len(), 3);

        let ref_ = storage
            .put(&meta, buffer)
            .expect("put blob in mem storage failed!");
        let buf = storage
            .get(&meta, &ref_)
            .expect("getting blob from the mem storage failed!");

        assert_eq!(buf.len(), 3);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 42);
        assert_eq!(buf[2], 0);
    }
}
