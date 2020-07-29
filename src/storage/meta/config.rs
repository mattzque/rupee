// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License

use crate::storage::meta::backend::mem::{MemoryMetaStorageConfig};
use crate::storage::meta::backend::rocksdb::{RocksDbMetaStorageConfig};
use crate::storage::meta::backend::postgres::{PostgresMetaStorageConfig};
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Deserialize)]
pub struct MetaStorageConfig {
    pub storage_meta_type: String,
    pub storage_meta_mem: MemoryMetaStorageConfig,
    pub storage_meta_rocksdb: RocksDbMetaStorageConfig,
    pub storage_meta_postgres: PostgresMetaStorageConfig,
}
