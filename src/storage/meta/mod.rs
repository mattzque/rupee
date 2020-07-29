// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
pub mod backend;
pub mod config;
pub mod factory;
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
    UnknownBackendError,
    CreateStorageError(&'static str),
    BackendError(&'static str),
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

    /// Load meta object from storage.
    fn get_meta(&mut self, id: Uuid) -> Result<Option<BlobMeta>, MetaStorageError>;

    /// Load blob refs from storage.
    fn get_blob_refs(&mut self, id: Uuid) -> Result<Option<HashMap<String, Box<dyn BlobRef>>>, MetaStorageError>;

    /// Delete meta and blob refs from storage.
    fn delete(&mut self, id: Uuid) -> Result<(), MetaStorageError>;
}
