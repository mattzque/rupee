// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
extern crate serde;
use serde::{Serialize};


#[derive(Serialize)]
pub struct PongResponse {
    message: String
}


impl Default for PongResponse {
    fn default() -> Self {
        Self { message: String::from("pong") }
    }
}
