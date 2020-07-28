// Rupee - Rust Image Service
// Copyright (C) 2020 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
use actix_web::{web, Responder};
use super::super::response::pong::PongResponse;


pub fn ping_handler() -> impl Responder {
    web::Json(PongResponse::default())
}
