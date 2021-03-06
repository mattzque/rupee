// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
use crate::domain::meta::BlobMeta;
use crate::storage::meta::{MetaStorage, MetaStorageError};
use crate::storage::meta::config::{MetaStorageConfig};
use crate::storage::meta::backend::mem::{MemoryMetaStorage, MemoryMetaStorageConfig};
use crate::storage::meta::backend::rocksdb::{RocksDbMetaStorage, RocksDbMetaStorageConfig};
use crate::storage::meta::backend::postgres::{PostgresMetaStorage, PostgresMetaStorageConfig};
use std::any::Any;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;


pub fn create_meta_storage(config: MetaStorageConfig) -> Result<Box<dyn MetaStorage>, MetaStorageError> {
    match config.storage_meta_type.as_ref() {
        "mem" => {
            Ok(Box::new(MemoryMetaStorage::new(config.storage_meta_mem)?))
        }
        "rocksdb" => {
            Ok(Box::new(RocksDbMetaStorage::new(config.storage_meta_rocksdb)?))
        }
        "postgres" => {
            Ok(Box::new(PostgresMetaStorage::new(config.storage_meta_postgres)?))
        }
        _ => Err(MetaStorageError::UnknownBackendError),
    }
}

