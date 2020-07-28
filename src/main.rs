#![allow(unused_imports)]
#![allow(unused_variables)]
// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
// mod service;
// use actix_web::{web, App, HttpServer};
// use service::handler::ping::ping_handler;
extern crate rupee;
extern crate uuid;
extern crate vips;
use rupee::{Config};
use rupee::domain::meta::BlobMeta;
use rupee::storage::blob::factory::create_blob_storage;
use vips::Vips;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Read;

fn main() {
    // let vips = Vips::new().expect("unexpected vips init error!");

    let config: Config = serde_yaml::from_reader(File::open("res/config.yml").expect("error!")).expect("error!");

    println!("test");
    println!("{:?}", config);
    println!("{:?}", config.storage_blob_bucket.path);
    println!("{:?}", config.storage_blob_bucket.max_size);

}

/*
fn main() {
    let mut backend = create_blob_storage("mem").expect("WHAT?");

    let buffer: Vec<u8> = vec![0, 42, 0];
    let meta = BlobMeta::new(buffer.len());

    let blob_ref = backend.put(&meta, buffer).expect("WHAT?");

    println!("meta: {}", meta);
    println!("blob_ref: {}", blob_ref);

    let buf = backend.get(&meta, &blob_ref).expect("asdf");

    println!("{}", buf.len());
    println!("{:?}", buf);
}
*/

/*
fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        .route("/ping", web::get().to(ping_handler))
    )
    .bind("127.0.0.1:8080")?
    .run()
}
*/
