// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
/*

enum MetaStorageError {
    WriteError,
    DeleteError
}

trait MetaStorage {
    fn put(meta: &BlobMeta) -> Result<(), MetaStorageError>;
    fn get(id: Uuid) -> Option<BlobMeta>;
    fn delete(id: Uuid) -> Result<(), MetaStorageError>;
}

*/

use uuid::Uuid;
use crate::domain::meta::BlobMeta;
use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::usize;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub enum MetaStorageError {
    InitError,
    PutError,
}


pub trait MetaStorage {
    /// Persist meta objects into the storage.
    fn put(
        &mut self,
        meta: BlobMeta,
        blob_refs: HashMap<String, Box<dyn BlobRef>>
    ) -> Result<(), MetaStorageError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryMetaStorageConfig {}

pub struct MemoryMetaStorage {
}

impl MemoryMetaStorage {
    pub fn new(config: MemoryMetaStorageConfig) -> Result<Self, MetaStorageError> {
        Ok(Self {})
    }
}

impl MetaStorage for MemoryMetaStorage {
    fn put(
        &mut self,
        meta: BlobMeta,
        blob_refs: HashMap<String, Box<dyn BlobRef>>
    ) -> Result<(), MetaStorageError> {

        let test_yaml = serde_yaml::to_string(&blob_refs).unwrap();
        println!("yaml <- {:?}", test_yaml);
        println!("yaml <- {:?}", test_yaml.len());
        let test_yaml_decoded: HashMap<String, Box<dyn BlobRef>> = serde_yaml::from_str(&test_yaml).unwrap();
        println!("yaml -> {:?}", serde_json::to_string(&test_yaml_decoded).unwrap());

        let test_json = serde_json::to_string(&blob_refs).unwrap();
        println!("json <- {:?}", test_json);
        println!("json <- {:?}", test_json.len());
        let test_json_decoded: HashMap<String, Box<dyn BlobRef>> = serde_json::from_str(&test_json).unwrap();
        println!("json -> {:?}", serde_json::to_string(&test_json_decoded).unwrap());

        let test_msgpack = rmp_serde::to_vec_named(&blob_refs).unwrap();
        println!("msgpack <- {:?}", test_msgpack);
        println!("msgpack <- {:?}", test_msgpack.len());
        let test_msgpack_decoded: HashMap<String, Box<dyn BlobRef>> = rmp_serde::from_read_ref(&test_msgpack).unwrap();
        println!("msgpack -> {:?}", serde_json::to_string(&test_msgpack_decoded).unwrap());

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::meta::MetaStorage;
    use crate::domain::meta::BlobMeta;
    use crate::{load_fixture};
    use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
    use crate::storage::meta::{MemoryMetaStorage, MemoryMetaStorageConfig};
    use crate::storage::blob::backend::mem::{MemoryBlobRef};
    use crate::storage::blob::backend::bucket::{BucketBlobRef};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;

    #[test]
    fn test_memory_meta_storage() {
        let mut storage = MemoryMetaStorage::new(MemoryMetaStorageConfig {}).expect("cant create meta storage");

        let meta = BlobMeta::new(1024);
        let memory_blob_ref = MemoryBlobRef { index: 42 };
        let bucket_blob_ref = BucketBlobRef { bucket: 42, offset: 1, size: 1024 };

        let mut blob_refs: HashMap<String, Box<dyn BlobRef>> = HashMap::new();
        blob_refs.insert("mem".to_string(), Box::new(memory_blob_ref));
        blob_refs.insert("bucket".to_string(), Box::new(bucket_blob_ref));

        storage.put(meta, blob_refs).expect("cant put to meta storage");
    }
}




