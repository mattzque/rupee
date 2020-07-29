// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
//! Bucket Blob Storage Backend Implementation
//!
//! This backend stores binary blobs in file buckets, large binary files
//! containing multiple binary blobs concatenated with external metadata.
//!
use crate::domain::meta::BlobMeta;
use crate::storage::blob::{BlobRef, BlobStorage, BlobStorageError};
use crate::{load_fixture};
use serde::{Serialize, Deserialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::usize;
use std::convert::TryInto;
use typetag::serde;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BucketBlobRef {
    /// This refers to the bucket file index.
    pub bucket: usize,
    /// The byte offset within the bucket filename pointing at the blob.
    pub offset: usize,
    /// The size of the binary blob stored.
    pub size: usize,
}

#[typetag::serde]
impl BlobRef for BucketBlobRef {
    fn any(&self) -> &dyn Any {
        self
    }

    fn display(&self) -> String {
        format!(
            "BucketBlobRef(#{}, offset={}, size={})",
            self.bucket, self.offset, self.size
        )
    }

    fn clone_as_dyn_blob_ref(&self) -> Box<dyn BlobRef + 'static> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BucketBlobStorageConfig {
    /// The bucket data directory.
    pub path: PathBuf,
    /// Maximum bucket size (in bytes).
    pub max_size: u64,
}

struct BucketFile {
    /// path to the bucket storage directory
    path: PathBuf,
    /// the index of the bucket file
    index: usize,
    descriptor: File,
}
impl BucketFile {
    fn format_bucket(path: &Path, bucket: usize) -> PathBuf {
        path.join(format!("{:08}", bucket))
    }

    /// Attempts to create and lock a new or existing bucket file.
    /// Returns None if the bucket file is locked or larger than the max bucket size.
    fn new(
        path: &Path,
        index: usize,
        max_size: u64,
    ) -> Result<Option<BucketFile>, BlobStorageError> {
        let filename = BucketFile::format_bucket(path, index);

        // if this bucket already exists, check the maximum size:
        // if the existing bucket is larger return Ok(None)
        // if the existing bucket file size cannot be determined return a Error
        if filename.is_file() {
            match fs::metadata(&filename) {
                // bucket file is larger than configured max bucket size
                Ok(ref metadata) if metadata.len() >= max_size => return Ok(None),
                // break with error
                Err(_) => {
                    return Err(BlobStorageError::CreateStorageError(
                        "Error reading size of existing file!",
                    ))
                }
                _ => {}
            }
        }

        // attempt to create a lockfile in the format <bucket#num>.lock
        // if the file cannot be created because it already exists return None
        // NOTE this must be atomic / thread/process-safe
        let lockfile = filename.with_extension("lock");
        if let Err(err) = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&lockfile)
        {
            eprintln!(
                "Bucket: error gaining the file lock! err={:?} filename={:?}",
                err, &lockfile
            );
            Ok(None)
        } else {
            match OpenOptions::new()
                .write(true)
                .read(true)
                .append(true)
                .create(true)
                .open(filename)
            {
                Ok(file) => Ok(Some(BucketFile {
                    path: path.to_path_buf(),
                    index,
                    descriptor: file,
                })),
                Err(x) => Err(BlobStorageError::CreateStorageError(
                    "Error opening bucket file!",
                )),
            }
        }
    }

    fn new_readonly(
        path: &Path,
        index: usize,
    ) -> Result<BucketFile, BlobStorageError> {
        let filename = BucketFile::format_bucket(path, index);
        match OpenOptions::new()
            .write(false)
            .read(true)
            .open(filename)
        {
            Ok(file) => Ok(BucketFile {
                path: path.to_path_buf(),
                index,
                descriptor: file,
            }),
            Err(x) => Err(BlobStorageError::CreateStorageError(
                "Error opening bucket file!",
            ))
        }
    }

    /// Finds and returns an available bucket file.
    fn find_available(path: &Path, max_size: u64) -> Result<Self, BlobStorageError> {
        const MAX_BUCKET_FILES: usize = 9999999;
        let mut index: usize = 1;
        loop {
            eprintln!("Bucket: attempt to get bucketfile @ index={}", index);
            let bucket = Self::new(path, index, max_size)?;

            if let Some(bucket) = bucket {
                return Ok(bucket);
            }

            index += 1;
            if index > MAX_BUCKET_FILES {
                return Err(BlobStorageError::CreateStorageError(
                    "Error finding bucket file to use!",
                ));
            }
        }
    }

    /// Check if there are any existing lockfiles in the bucket directory.
    /// This is indicating either a previous crash of the application or another instance is
    /// running on the same directory.
    fn find_locks(path: &Path) -> Result<bool, BlobStorageError> {
        match fs::read_dir(&path) {
            Ok(files) => {
                // eprintln!("{:?}", files);
                for file in files {
                    // this is silly:
                    if let Ok(file) = file {
                        if let Ok(file) = file.file_name().into_string() {
                            if file.ends_with(".lock") {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
            Err(_) => Err(BlobStorageError::CreateStorageError(
                "Error finding existing lock files!",
            )),
        }
    }

    /// Returns the bucket index.
    fn index(&self) -> usize {
        self.index
    }

    /// Returns the file descriptors current position in the bucket file.
    fn tell(&mut self) -> Result<usize, BlobStorageError> {
        self.seek(SeekFrom::Current(0))
    }

    /// Move the file descriptors current position to the given position.
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, BlobStorageError> {
        match self.descriptor.seek(pos) {
            Ok(pos) => Ok(pos as usize), // TODO fix cast!!!!
            Err(_) => Err(BlobStorageError::WriteError),
        }
    }

    /// Write some new data into the bucket file, returns the offset into the file where the buffer was
    /// written at.
    fn write(&mut self, buffer: &[u8]) -> Result<usize, BlobStorageError> {
        // seek to the end of the file and return the file location
        let offset = self.seek(SeekFrom::End(0))?;

        // this should not be needed:
        // self.descriptor.sync_all()?;

        match self.descriptor.write_all(buffer) {
            Ok(_) => Ok(offset),
            Err(_) => Err(BlobStorageError::WriteError),
        }
    }

    /// Read a single blob out of the bucket.
    fn read(&mut self, offset: usize, size: usize) -> Result<Vec<u8>, BlobStorageError> {
        self.seek(SeekFrom::Start(offset.try_into().unwrap()))?;

        let mut buffer = vec![0; size as usize];
        self.descriptor.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
impl Drop for BucketFile {
    fn drop(&mut self) {
        let lockfile = BucketFile::format_bucket(&self.path, self.index).with_extension("lock");
        //println!("Bucket: release lock file: {:?}", lockfile);
        if let Err(_) = fs::remove_file(lockfile) {
            println!("Bucket: Error removing bucket lock file!");
        }
    }
}

pub struct BucketBlobStorage {
    /// Current bucket file.
    bucket: Box<BucketFile>,
    /// Bucket configuration.
    config: Box<BucketBlobStorageConfig>,
}

impl BucketBlobStorage {
    /// Global Bucket Blob Storage Initialization function.
    /// This function is called only once per program lifetime, it is used to do various
    /// checks and preparations on the storage backend.
    pub fn init(config: &BucketBlobStorageConfig) -> Result<(), BlobStorageError> {
        // create the block storage directory
        if !config.path.is_dir() {
            if let Err(_) = fs::create_dir_all(&config.path) {
                return Err(BlobStorageError::CreateStorageError(
                    "Error creating the blob storage directory!",
                ));
            }
        }

        // check for any existing lock files
        match BucketFile::find_locks(&config.path) {
            Ok(true) => {
                eprintln!("Error creating the bucket blob storage backend!");
                eprintln!(
                    "Found existing lock files in storage directory: {:?}",
                    &config.path
                );
                eprintln!("Check existing running instances and delete lock files.");
                Err(BlobStorageError::CreateStorageError(
                    "Error found existing locks in bucket storage directory!",
                ))
            }
            Ok(false) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Create a new Bucket Blob Storage instance.
    /// It automatically choses a bucket file to use based on existing files and their sizes.
    /// This thread is thread-safe but ideally it should be wrapped in a mutex.
    pub fn new(config: BucketBlobStorageConfig) -> Result<Self, BlobStorageError> {
        if !config.path.is_dir() {
            return Err(BlobStorageError::CreateStorageError(
                "Storage directory not found!",
            ));
        }

        // create a new bucket file this instance is going to work with, the bucketfile
        // holds a lock file in the storage path that is deleted when its deconstructed,
        // this way the lock file is bound to the lifetime of the (immutable) bucketfile.
        let bucket = Box::new(BucketFile::find_available(&config.path, config.max_size)?);
        let config = Box::new(config);

        Ok(Self { bucket, config })
    }

    fn get_current_bucket(&mut self) -> Result<&mut BucketFile, BlobStorageError> {
        if self.bucket.tell()? >= self.config.max_size as usize {
            self.bucket = Box::new(BucketFile::find_available(&self.config.path, self.config.max_size)?);
        }
        Ok(&mut self.bucket)
    }
}

impl BlobStorage for BucketBlobStorage {
    /// Read blob back from storage.
    /// This reads the entire blob and returns it.
    /// Args:
    ///     meta: storage-independent blob meta reference
    ///     blob_ref: storage-specific blob reference
    /// Returns:
    ///     bytes or error (of type BlobStorageError)
    fn get(&self, meta: &BlobMeta, blob_ref: &Box<dyn BlobRef>) -> Result<Vec<u8>, BlobStorageError> {
        if let Some(blob_ref) = blob_ref.any().downcast_ref::<BucketBlobRef>() {
            if blob_ref.size != meta.size {
                Err(BlobStorageError::ReadStorageError)
            }
            else {
                println!("blob_ref.offset => {:?}", blob_ref.offset);
                println!("blob_ref.size => {:?}", blob_ref.size);
                match BucketFile::new_readonly(&self.config.path, blob_ref.bucket) {
                    Ok(mut bucket) => bucket.read(blob_ref.offset, blob_ref.size),
                    Err(x) => Err(BlobStorageError::ReadStorageError)
                }
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
        let bucket = self.get_current_bucket()?;

        // store in current bucket
        let offset = bucket.write(&buffer)?;
        println!("offset => {:?}", offset);
        let blob_ref = BucketBlobRef {
            bucket: bucket.index(),
            offset,
            size: buffer.len(),
        };
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
    extern crate tempfile;
    use crate::domain::meta::BlobMeta;
    use crate::storage::blob::backend::bucket::*;

    use std::path::Path;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;


    #[test]
    fn test_blob_race() {
        let mut threads = Vec::new();
        let dir = tempdir().expect("expected to write temporary directory!");

        let config = BucketBlobStorageConfig {
            path: dir.path().to_path_buf(),
            max_size: 1024 * 1024 * 1024 * 24,
        };

        BucketBlobStorage::init(&config).expect("Error in init of bucket storage!");

        let x = 42;

        for n in 0..15 {
            let config = config.clone();
            let handle = thread::spawn(|| {
                let mut storage =
                    BucketBlobStorage::new(config).expect("error creating the blob storage");
                thread::sleep(Duration::from_millis(100));

                // put a data blob in the bucket:
                let buffer: Vec<u8> = vec![0, 42, 0, 42, 0];
                let meta = BlobMeta::new(buffer.len());

                let ref_ = storage
                    .put(&meta, buffer)
                    .expect("put blob in bucket storage failed!");

                thread::sleep(Duration::from_millis(100));

                println!("ref -> {:?}", ref_.display());
                println!("config -> {:?}", storage.config);
            });
            threads.push(handle);
        }

        for handle in threads {
            handle.join().expect("error, expecting to join thread!");
        }
    }

    #[test]
    fn test_blob_bucket() {
        // load test fixture files:
        let image_1 = load_fixture(Path::new("images").join("rgb.jpeg"));
        let image_1_meta = BlobMeta::new(image_1.len());

        let image_2 = load_fixture(Path::new("images").join("rgb.png"));
        let image_2_meta = BlobMeta::new(image_2.len());

        let image_3 = load_fixture(Path::new("images").join("rgba.png"));
        let image_3_meta = BlobMeta::new(image_3.len());

        let dir = tempdir().expect("expected to write temporary directory!");
        let config = BucketBlobStorageConfig {
            path: dir.path().to_path_buf(),
            max_size: 1024 * 1024 * 1024 * 24,
        };
        BucketBlobStorage::init(&config).expect("Error in init of bucket storage!");
        let mut storage = BucketBlobStorage::new(config).expect("error creating the blob storage");

        // put images into bucket:
        let reference_1 = storage.put(&image_1_meta, image_1.to_vec()).expect("put blob failed!");
        let reference_2 = storage.put(&image_2_meta, image_2.to_vec()).expect("put blob failed!");
        let reference_3 = storage.put(&image_3_meta, image_3.to_vec()).expect("put blob failed!");

        // read back again
        let image_1_res = storage.get(&image_1_meta, &reference_1).expect("get blob failed!").to_vec();
        let image_2_res = storage.get(&image_2_meta, &reference_2).expect("get blob failed!").to_vec();
        let image_3_res = storage.get(&image_3_meta, &reference_3).expect("get blob failed!").to_vec();

        assert_eq!(image_1.len(), image_1_res.len());
        assert_eq!(image_2.len(), image_2_res.len());
        assert_eq!(image_3.len(), image_3_res.len());

        assert_eq!(image_1, image_1_res);
        assert_eq!(image_2, image_2_res);
        assert_eq!(image_3, image_3_res);
    }

    #[test]
    fn test_blob_bucket_chunking() {
        // tests files placed in different buckets
        // load test fixture files:
        let image_1 = load_fixture(Path::new("images").join("rgb.jpeg"));
        let image_1_meta = BlobMeta::new(image_1.len());

        let image_2 = load_fixture(Path::new("images").join("rgb.png"));
        let image_2_meta = BlobMeta::new(image_2.len());

        let image_3 = load_fixture(Path::new("images").join("rgba.png"));
        let image_3_meta = BlobMeta::new(image_3.len());

        let dir = tempdir().expect("expected to write temporary directory!");
        let config = BucketBlobStorageConfig {
            path: dir.path().to_path_buf(),
            max_size: 1024,  // small size, each image gets placed in different bucket
        };
        BucketBlobStorage::init(&config).expect("Error in init of bucket storage!");
        let mut storage = BucketBlobStorage::new(config).expect("error creating the blob storage");

        // put images into bucket:
        let reference_1 = storage.put(&image_1_meta, image_1.to_vec()).expect("put blob failed!");
        let reference_2 = storage.put(&image_2_meta, image_2.to_vec()).expect("put blob failed!");
        let reference_3 = storage.put(&image_3_meta, image_3.to_vec()).expect("put blob failed!");

        assert_eq!(reference_1.any().downcast_ref::<BucketBlobRef>().expect("").bucket, 1);
        assert_eq!(reference_2.any().downcast_ref::<BucketBlobRef>().expect("").bucket, 2);
        assert_eq!(reference_3.any().downcast_ref::<BucketBlobRef>().expect("").bucket, 3);

        // read back again
        let image_1_res = storage.get(&image_1_meta, &reference_1).expect("get blob failed!").to_vec();
        let image_2_res = storage.get(&image_2_meta, &reference_2).expect("get blob failed!").to_vec();
        let image_3_res = storage.get(&image_3_meta, &reference_3).expect("get blob failed!").to_vec();

        assert_eq!(image_1.len(), image_1_res.len());
        assert_eq!(image_2.len(), image_2_res.len());
        assert_eq!(image_3.len(), image_3_res.len());

        assert_eq!(image_1, image_1_res);
        assert_eq!(image_2, image_2_res);
        assert_eq!(image_3, image_3_res);
    }
}
