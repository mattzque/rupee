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
use postgres::{Config, Client, NoTls};
use postgres::types::Type;


#[derive(Debug, Clone, Deserialize)]
pub struct PostgresMetaStorageConfig {
    pub hostname: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

pub struct PostgresMetaStorage {
    client: Client
}



impl From<postgres::Error> for MetaStorageError {
    fn from(error: postgres::Error) -> Self {
        eprintln!("Meta: PostgreSQL Error {:?}", error);
        MetaStorageError::BackendError("postgres error")
    }
}

impl From<serde_json::error::Error> for MetaStorageError {
    fn from(error: serde_json::error::Error) -> Self {
        eprintln!("Meta: Json Serialization Error {:?}", error);
        MetaStorageError::BackendError("json serialization error")
    }
}



impl PostgresMetaStorage {
    fn get_config(config: &PostgresMetaStorageConfig) -> Config {
        let mut dbconfig = Config::new();

        dbconfig.user(&config.username);
        dbconfig.password(&config.password);
        dbconfig.dbname(&config.database);
        dbconfig.host(&config.hostname);
        dbconfig.port(config.port);

        dbconfig
    }

    /// Global Postgres Meta Storage Initialization function.
    /// This function is called only once per program lifetime, it is used to do various
    /// checks and preparations on the storage backend.
    pub fn init(config: &PostgresMetaStorageConfig) -> Result<(), MetaStorageError> {
        let mut client = Self::get_config(&config).connect(NoTls)?;
        client.batch_execute("
          CREATE TABLE IF NOT EXISTS meta (
              id        UUID PRIMARY KEY,
              meta      JSONB NOT NULL,
              blob_refs JSONB NOT NULL
          );
        ")?;
        Ok(())
    }

    pub fn new(config: PostgresMetaStorageConfig) -> Result<Self, MetaStorageError> {
        let mut client = Self::get_config(&config).connect(NoTls)?;
        Ok(Self { client })
    }
}

impl MetaStorage for PostgresMetaStorage {
    fn put(
        &mut self,
        meta: BlobMeta,
        blob_refs: HashMap<String, Box<dyn BlobRef>>
    ) -> Result<(), MetaStorageError> {
        let key: String = meta.id.to_hyphenated().to_string();

        // encode to json
        let meta_encoded = serde_json::to_string(&meta)?;
        let blob_refs_encoded = serde_json::to_string(&blob_refs)?;

        let statement = self.client.prepare_typed(
            "INSERT INTO meta (id, meta, blob_refs) VALUES ($1::uuid, $2::jsonb, $3::jsonb)",
            &[Type::TEXT, Type::TEXT, Type::TEXT],
        )?;
        self.client.execute(&statement, &[&key, &meta_encoded, &blob_refs_encoded])?;

        Ok(())
    }

    fn get_meta(&mut self, id: Uuid) -> Result<Option<BlobMeta>, MetaStorageError> {
        let key: String = id.to_hyphenated().to_string();

        let statement = self.client.prepare_typed(
            "SELECT meta::text FROM meta WHERE id = $1::uuid",
            &[Type::TEXT],
        )?;

        match self.client.query_opt(&statement, &[&key])? {
            Some(row) => {
                let meta_string = row.try_get(0)?;
                let meta: BlobMeta = serde_json::from_str(meta_string)?;
                Ok(Some(meta))
            },
            None => Ok(None)
        }
    }

    fn get_blob_refs(&mut self, id: Uuid) -> Result<Option<HashMap<String, Box<dyn BlobRef>>>, MetaStorageError> {
        let key: String = id.to_hyphenated().to_string();

        let statement = self.client.prepare_typed(
            "SELECT blob_refs::text FROM meta WHERE id = $1::uuid",
            &[Type::TEXT],
        )?;

        match self.client.query_opt(&statement, &[&key])? {
            Some(row) => {
                let blob_refs_string = row.try_get(0)?;
                let blob_refs: HashMap<String, Box<dyn BlobRef>> = serde_json::from_str(blob_refs_string)?;
                Ok(Some(blob_refs))
            },
            None => Ok(None)
        }
    }

    fn delete(&mut self, id: Uuid) -> Result<(), MetaStorageError> {
        let key = id.to_string();

        let statement = self.client.prepare_typed(
            "DELETE FROM meta WHERE id = $1::uuid;",
            &[Type::TEXT],
        )?;
        self.client.execute(&statement, &[&key])?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::storage::meta::MetaStorage;
    use crate::domain::meta::BlobMeta;
    use crate::{load_fixture};
    use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
    use crate::storage::meta::backend::postgres::{PostgresMetaStorage, PostgresMetaStorageConfig};
    use crate::storage::blob::backend::mem::{MemoryBlobRef};
    use crate::storage::blob::backend::bucket::{BucketBlobRef};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_postgres_meta_storage() {
        let dir = tempdir().expect("expected to write temporary directory!");

        let config = PostgresMetaStorageConfig {
            hostname: "localhost".to_string(),
            port: 5432,
            database: "rupee".to_string(),
            username: "rupee".to_string(),
            password: "hu4euShohn7e".to_string(),
        };

        PostgresMetaStorage::init(&config).expect("Error in init of bucket storage!");

        let mut storage = PostgresMetaStorage::new(config).expect("cant create meta storage");

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
