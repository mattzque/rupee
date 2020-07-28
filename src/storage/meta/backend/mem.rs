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
use std::sync::Arc;
use std::usize;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryMetaStorageConfig {}

pub struct MemoryMetaStorage {
    metas: HashMap<Uuid, BlobMeta>,
    blob_refs: HashMap<Uuid, HashMap<String, Box<dyn BlobRef>>>,
}

impl MemoryMetaStorage {
    pub fn new(config: MemoryMetaStorageConfig) -> Result<Self, MetaStorageError> {
        Ok(Self { metas: HashMap::new(), blob_refs: HashMap::new() })
    }
}

impl MetaStorage for MemoryMetaStorage {
    fn put(
        &mut self,
        meta: BlobMeta,
        blob_refs: HashMap<String, Box<dyn BlobRef>>
    ) -> Result<(), MetaStorageError> {
        let key = meta.id;
        self.metas.insert(key, meta);
        self.blob_refs.insert(key, blob_refs);
        Ok(())
    }

    fn get_meta(&self, id: Uuid) -> Result<Option<BlobMeta>, MetaStorageError> {
        match self.metas.get(&id) {
            Some(meta) => Ok(Some(*meta)),
            None => Ok(None)
        }
    }

    fn get_blob_refs(&self, id: Uuid) -> Result<Option<HashMap<String, Box<dyn BlobRef>>>, MetaStorageError> {
        match self.blob_refs.get(&id) {
            Some(blob_refs) => Ok(Some((*blob_refs).clone())),
            None => Ok(None)
        }
    }

    fn delete(&mut self, id: Uuid) -> Result<(), MetaStorageError> {
        self.metas.remove(&id);
        self.blob_refs.remove(&id);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::meta::MetaStorage;
    use crate::domain::meta::BlobMeta;
    use crate::{load_fixture};
    use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
    use crate::storage::meta::backend::mem::{MemoryMetaStorage, MemoryMetaStorageConfig};
    use crate::storage::blob::backend::mem::{MemoryBlobRef};
    use crate::storage::blob::backend::bucket::{BucketBlobRef};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;

    #[test]
    fn test_memory_meta_storage() {
        let mut storage = MemoryMetaStorage::new(MemoryMetaStorageConfig {}).expect("cant create meta storage");

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





        /*

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

        */


