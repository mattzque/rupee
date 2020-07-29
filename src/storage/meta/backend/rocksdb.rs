// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
use uuid::Uuid;
use crate::storage::meta::{MetaStorageError, MetaStorage};
use crate::domain::meta::BlobMeta;
use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::usize;
use serde::{Serialize, Deserialize};
use rocksdb::DB;


#[derive(Debug, Clone, Deserialize)]
pub struct RocksDbMetaStorageConfig {
    /// The rocksdb data directory.
    pub path: PathBuf,
}

pub struct RocksDbMetaStorage {
    metas: DB,
    blob_refs: DB,
}

impl From<rocksdb::Error> for MetaStorageError {
    fn from(error: rocksdb::Error) -> Self {
        eprintln!("Meta: RocksDb Error {:?}", error);
        MetaStorageError::BackendError("rocksdb error")
    }
}

impl From<rmp_serde::encode::Error> for MetaStorageError {
    fn from(error: rmp_serde::encode::Error) -> Self {
        eprintln!("Meta: Serialization Error {:?}", error);
        MetaStorageError::BackendError("serialization error")
    }
}

impl From<rmp_serde::decode::Error> for MetaStorageError {
    fn from(error: rmp_serde::decode::Error) -> Self {
        eprintln!("Meta: Unserializing Error {:?}", error);
        MetaStorageError::BackendError("unserialize error")
    }
}


impl RocksDbMetaStorage {
    /// Global RocksDb Meta Storage Initialization function.
    /// This function is called only once per program lifetime, it is used to do various
    /// checks and preparations on the storage backend.
    pub fn init(config: &RocksDbMetaStorageConfig) -> Result<(), MetaStorageError> {
        // create the rocksdb storage directory
        if !config.path.is_dir() {
            if let Err(_) = fs::create_dir_all(&config.path) {
                return Err(MetaStorageError::CreateStorageError(
                    "Error creating the blob storage directory!",
                ));
            }
        }
        Ok(())
    }

    pub fn new(config: RocksDbMetaStorageConfig) -> Result<Self, MetaStorageError> {
        let metas = DB::open_default(config.path.join("metas"))?;
        let blob_refs = DB::open_default(config.path.join("blob_refs"))?;

        Ok(Self { metas, blob_refs })
    }
}

impl MetaStorage for RocksDbMetaStorage {
    fn put(
        &mut self,
        meta: BlobMeta,
        blob_refs: HashMap<String, Box<dyn BlobRef>>
    ) -> Result<(), MetaStorageError> {
        let key = meta.id.as_bytes();

        // encode using msgpack
        let meta_encoded = rmp_serde::to_vec_named(&meta)?;
        let blob_refs_encoded = rmp_serde::to_vec_named(&blob_refs)?;

        self.metas.put(key, meta_encoded)?;
        self.blob_refs.put(key, blob_refs_encoded)?;

        Ok(())
    }

    fn get_meta(&mut self, id: Uuid) -> Result<Option<BlobMeta>, MetaStorageError> {
        let key = id.as_bytes();

        match self.metas.get(key)? {
            Some(value) => {
                let meta_decoded: BlobMeta = rmp_serde::from_read_ref(&value)?;
                Ok(Some(meta_decoded))
            },
            None => Ok(None),
        }
    }

    fn get_blob_refs(&mut self, id: Uuid) -> Result<Option<HashMap<String, Box<dyn BlobRef>>>, MetaStorageError> {
        let key = id.as_bytes();

        match self.blob_refs.get(key)? {
            Some(value) => {
                let blob_refs_decoded: HashMap<String, Box<dyn BlobRef>> = rmp_serde::from_read_ref(&value)?;
                Ok(Some(blob_refs_decoded))
            },
            None => Ok(None),
        }
    }

    fn delete(&mut self, id: Uuid) -> Result<(), MetaStorageError> {
        let key = id.as_bytes();
        self.metas.delete(&key)?;
        self.blob_refs.delete(&key)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::meta::MetaStorage;
    use crate::domain::meta::BlobMeta;
    use crate::{load_fixture};
    use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
    use crate::storage::meta::backend::rocksdb::{RocksDbMetaStorage, RocksDbMetaStorageConfig};
    use crate::storage::blob::backend::mem::{MemoryBlobRef};
    use crate::storage::blob::backend::bucket::{BucketBlobRef};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_rocksdb_meta_storage() {
        let dir = tempdir().expect("expected to write temporary directory!");

        let config = RocksDbMetaStorageConfig {
            path: dir.path().to_path_buf(),
        };

        RocksDbMetaStorage::init(&config).expect("Error in init of bucket storage!");

        let mut storage = RocksDbMetaStorage::new(config).expect("cant create meta storage");

        let meta = BlobMeta::new(1024);
        let memory_blob_ref = MemoryBlobRef { index: 42 };
        let bucket_blob_ref = BucketBlobRef { bucket: 23, offset: 1, size: 1024 };

        let mut blob_refs: HashMap<String, Box<dyn BlobRef>> = HashMap::new();
        blob_refs.insert("mem".to_string(), Box::new(memory_blob_ref));
        blob_refs.insert("bucket".to_string(), Box::new(bucket_blob_ref));

        storage.put(meta, blob_refs).expect("cant put to meta storage");

        let got_meta = storage.get_meta(meta.id).unwrap();
        let got_blob_refs = storage.get_blob_refs(meta.id).unwrap().unwrap();

        let got_meta = got_meta.unwrap();
        assert_eq!(got_meta.size, 1024);

        let got_mem_blob_ref = got_blob_refs
            .get("mem")
            .unwrap()
            .any().downcast_ref::<MemoryBlobRef>()
            .unwrap();
        assert_eq!(got_mem_blob_ref.index, 42);

        let got_bucket_blob_ref = got_blob_refs
            .get("bucket")
            .unwrap()
            .any().downcast_ref::<BucketBlobRef>()
            .unwrap();
        assert_eq!(got_bucket_blob_ref.bucket, 23);
        assert_eq!(got_bucket_blob_ref.offset, 1);
        assert_eq!(got_bucket_blob_ref.size, 1024);

        storage.delete(meta.id).unwrap();
    }
}
