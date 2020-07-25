// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License

struct MediaFormat(MediaType, &'str)

enum MediaFormat {
    Image(mime: &'static str, ext: &'static str)
}

// MediaFormat::Image("png/image")


