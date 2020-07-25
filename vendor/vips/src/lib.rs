// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License
mod image;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
extern crate vips_sys;
use image::Image;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;

/// NOTE: Unsafe! This should not crash!
fn const_char_to_string(value: *const c_char) -> String {
    let c_str: &CStr = unsafe { CStr::from_ptr(value) };
    let str_slice: &str = c_str
        .to_str()
        .expect("expected null-terminated string from Cstr!");
    let str_buf: String = str_slice.to_owned();
    str_buf
}

/// NOTE: Unsafe! This should not crash!
pub fn path_to_cstring(path: &Path) -> CString {
    CString::new(
        path.to_str()
            .expect("Path to string failed!")
            .clone()
            .as_bytes(),
    )
    .expect("CString::new failed!")
}

#[derive(Debug, Clone, PartialEq)]
pub enum VipsError {
    VipsInitError,
    VipsImageFormatError,
    VipsImageWriteError,
    VipsImageLoadError,
    VipsImageFileNotFoundError,
    VipsImageResizeError,
}

/// Image Formats, vips supports many more but we constrain this to just these formats.
///
/// Note: GIF files are written using ImageMagick
/// Note: SVG file support is limited since vips does not support vector graphics!
#[derive(Debug, Clone, PartialEq)]
pub enum VipsFormat {
    VipsJpeg,
    VipsPng,
    VipsGif,
    VipsWebP,
    VipsSvg,
}

impl VipsFormat {
    fn from_operation_name(name: &str) -> Result<VipsFormat, VipsError> {
        match name {
            "VipsForeignLoadPngBuffer" => Ok(VipsFormat::VipsPng),
            "VipsForeignLoadJpegBuffer" => Ok(VipsFormat::VipsJpeg),
            "VipsForeignLoadGifBuffer" => Ok(VipsFormat::VipsGif),
            "VipsForeignLoadWebpBuffer" => Ok(VipsFormat::VipsWebP),
            "VipsForeignLoadSvgBuffer" => Ok(VipsFormat::VipsSvg),
            "VipsForeignLoadPng" => Ok(VipsFormat::VipsPng),
            "VipsForeignLoadJpeg" => Ok(VipsFormat::VipsJpeg),
            "VipsForeignLoadGif" => Ok(VipsFormat::VipsGif),
            "VipsForeignLoadWebp" => Ok(VipsFormat::VipsWebP),
            "VipsForeignLoadSvg" => Ok(VipsFormat::VipsSvg),
            "VipsForeignLoadPngFile" => Ok(VipsFormat::VipsPng),
            "VipsForeignLoadJpegFile" => Ok(VipsFormat::VipsJpeg),
            "VipsForeignLoadGifFile" => Ok(VipsFormat::VipsGif),
            "VipsForeignLoadWebpFile" => Ok(VipsFormat::VipsWebP),
            "VipsForeignLoadSvgFile" => Ok(VipsFormat::VipsSvg),
            _ => Err(VipsError::VipsImageFormatError),
        }
    }
}

#[derive(Debug)]
pub struct Vips {}

impl Vips {
    pub fn new() -> Result<Vips, VipsError> {
        let args: Vec<String> = env::args().collect();
        let argv0 =
            CString::new(args[0].clone().into_bytes()).expect("CString::new failed on argv[0]!");
        // TODO this should never crash!

        eprintln!("Vips: Initializing...");

        let result = unsafe { vips_sys::vips_init(argv0.as_ptr()) };

        match result {
            0 => Ok(Vips {}),
            _ => Err(VipsError::VipsInitError),
        }
    }
}

impl Drop for Vips {
    fn drop(&mut self) {
        let errors = self.read_error_buffer();
        if errors.len() > 0 {
            eprintln!("Vips: Error Buffer Contents:\n{}", errors);
        }
        unsafe {
            vips_sys::vips_shutdown();
        }
        eprintln!("Vips: Shutdown.");
    }
}

impl Vips {
    pub fn version(&self) -> String {
        let version_str = unsafe { vips_sys::vips_version_string() };
        const_char_to_string(version_str)
    }

    pub fn enable_leak_checks(&self) {
        unsafe { vips_sys::vips_leak_set(1) }
    }

    pub fn read_error_buffer(&self) -> String {
        let buffer = unsafe { vips_sys::vips_error_buffer() };
        const_char_to_string(buffer)
    }

    pub fn find_image_format_from_buffer(&self, buffer: &[u8]) -> Result<VipsFormat, VipsError> {
        let load_op_name = unsafe {
            vips_sys::vips_foreign_find_load_buffer(buffer.as_ptr() as *const c_void, buffer.len())
        };

        if load_op_name.is_null() {
            Err(VipsError::VipsImageFormatError)
        } else {
            VipsFormat::from_operation_name(&const_char_to_string(load_op_name))
        }
    }

    pub fn find_image_format_from_file(&self, filename: &Path) -> Result<VipsFormat, VipsError> {
        if !filename.is_file() {
            return Err(VipsError::VipsImageFileNotFoundError);
        }

        let load_op_name =
            unsafe { vips_sys::vips_foreign_find_load(path_to_cstring(&filename).as_ptr()) };

        if load_op_name.is_null() {
            Err(VipsError::VipsImageFormatError)
        } else {
            VipsFormat::from_operation_name(&const_char_to_string(load_op_name))
        }
    }

    pub fn load_image_from_file(&self, filename: &Path) -> Result<Image, VipsError> {
        if !filename.is_file() {
            return Err(VipsError::VipsImageFileNotFoundError);
        }

        eprintln!("Vips: loading file from disk: {:?}", filename);
        let vips_image = unsafe {
            vips_sys::vips_image_new_from_file(
                path_to_cstring(&filename).as_ptr(),
                ptr::null() as *const c_void,
            )
        };

        if vips_image.is_null() {
            Err(VipsError::VipsImageLoadError)
        } else {
            // we keep a reference to Vips in the image, this way the lifetime of vips is
            // bound to the Image instance, to ensure that Vips isn't going out of
            // scope while there is still an Image is around.
            Ok(Image::new(&self, vips_image))
        }
    }

    pub fn load_image_from_buffer(&self, buffer: &[u8]) -> Result<Image, VipsError> {
        eprintln!("Vips: loading file from memory buffer: {:?}", buffer.len());
        let vips_image = unsafe {
            vips_sys::vips_image_new_from_buffer(
                buffer.as_ptr() as *const c_void,
                buffer.len(),
                ptr::null() as *const c_char,
                ptr::null() as *const c_char,
            )
        };

        if vips_image.is_null() {
            Err(VipsError::VipsImageLoadError)
        } else {
            // we keep a reference to Vips in the image, this way the lifetime of vips is
            // bound to the Image instance, to ensure that Vips isn't going out of
            // scope while there is still an Image is around.
            Ok(Image::new(&self, vips_image))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::image::{VipsCropMode, VipsSizeMode};
    use super::{Vips, VipsFormat};
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    lazy_static! {
        static ref VIPS: Vips = Vips::new().expect("unexpected vips initialization error!");
    }

    #[test]
    fn test_load_from_file() {
        let fixtures = env::current_dir()
            .expect("Can not determine current working directory!")
            .join("../../res/fixtures/images");

        let image = (*VIPS)
            .load_image_from_file(&fixtures.join("rgb.jpeg"))
            .expect("unexpected error loading image!");

        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 300);
        assert_eq!(image.channel(), 3);

        let image = (*VIPS)
            .load_image_from_file(&fixtures.join("rgba.png"))
            .expect("unexpected error loading image!");

        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 300);
        assert_eq!(image.channel(), 4); // alpha channel
    }

    #[test]
    fn test_load_from_buffer() {
        let fixtures = env::current_dir()
            .expect("Can not determine current working directory!")
            .join("../../res/fixtures/images");

        let buffer = fs::read(fixtures.join("rgba.png"))
            .expect("unexpected fixture can't be read into buffer!");

        let image = (*VIPS)
            .load_image_from_buffer(&buffer)
            .expect("unexpected image can't be read from buffer!");

        assert_eq!(image.width(), 200);
        assert_eq!(image.height(), 300);
        assert_eq!(image.channel(), 4); // alpha channel
    }

    #[test]
    fn test_find_image_format() {
        let fixtures = env::current_dir()
            .expect("Can not determine current working directory!")
            .join("../../res/fixtures/images");

        let formats = [
            ("rgb.png", VipsFormat::VipsPng),
            ("rgb.jpeg", VipsFormat::VipsJpeg),
            ("jfif_apple.jpeg", VipsFormat::VipsJpeg),
            ("animated.gif", VipsFormat::VipsGif),
            ("rgb.webp", VipsFormat::VipsWebP),
            ("example.svg", VipsFormat::VipsSvg),
            ("example2.svg", VipsFormat::VipsSvg),
        ];

        for (filename, format) in formats.iter() {
            // Test detection from buffer: find_image_format_from_buffer
            let buffer = fs::read(fixtures.join(filename))
                .expect(&format!("can't load fixture file: {}", filename));
            let op = (*VIPS)
                .find_image_format_from_buffer(&buffer)
                .expect(&format!("can't recognize format from buffer: {}", filename));
            println!("detected {} as {:?}", filename, op);
            assert_eq!(op, *format);

            // Test detection from file: find_image_format_from_file
            let op = (*VIPS)
                .find_image_format_from_file(&fixtures.join(filename))
                .expect(&format!("can't recognize format from file: {}", filename));
            println!("detected {} as {:?}", filename, op);
            assert_eq!(op, *format);
        }
    }

    #[test]
    fn test_image_resize() {
        let fixtures = env::current_dir()
            .expect("Can not determine current working directory!")
            .join("../../res/fixtures/images");

        let image = (*VIPS)
            .load_image_from_file(&fixtures.join("rgba.png"))
            .expect("unexpected error loading image!");

        let smaller = image
            .resize(100, 120, None, None)
            .expect("image should be able to be rescaled!");

        assert_eq!(smaller.width(), 100);
        assert_eq!(smaller.height(), 120);
        assert_eq!(smaller.channel(), 4); // maintain alpha channel after resize!

        // Size UP -> 200x300 Take largest axis (H) and then calculate the other axis according to
        // the aspect ratio 300 > 800, 200 -> 533
        let smaller = image
            .resize(800, 800, Some(VipsSizeMode::VipsSizeUp), None)
            .expect("image should be able to be rescaled!");

        assert_eq!(smaller.width(), 533);
        assert_eq!(smaller.height(), 800);
        assert_eq!(smaller.channel(), 4); // maintain alpha channel after resize!

        // Size DOWN -> This will not upsize if the requested size is larger than the current size.
        let smaller = image
            .resize(800, 800, Some(VipsSizeMode::VipsSizeDown), None)
            .expect("image should be able to be rescaled!");

        assert_eq!(smaller.width(), 200);
        assert_eq!(smaller.height(), 300);
        assert_eq!(smaller.channel(), 4); // maintain alpha channel after resize!
    }

    #[test]
    fn test_image_save_to_buffer() {
        let fixtures = env::current_dir()
            .expect("Can not determine current working directory!")
            .join("../../res/fixtures/images");

        let image = (*VIPS)
            .load_image_from_file(&fixtures.join("example.svg"))
            .expect("unexpected error loading image!");

        let formats = [
            ("rgb.png", VipsFormat::VipsPng),
            ("rgb.jpeg", VipsFormat::VipsJpeg),
            ("jfif_apple.jpeg", VipsFormat::VipsJpeg),
            ("animated.gif", VipsFormat::VipsGif),
            ("rgb.webp", VipsFormat::VipsWebP),
            ("example.svg", VipsFormat::VipsSvg),
            ("example2.svg", VipsFormat::VipsSvg),
        ];

        let dir = tempdir().expect("expected to write temporary directory!");

        for (filename, format) in formats.iter() {
            let image = (*VIPS)
                .load_image_from_file(&fixtures.join(filename))
                .expect("unexpected error loading image!");

            let buffer = image
                .save_to_buffer(format)
                .expect(&format!("Error loading buffer: {} {:?}", filename, format));

            assert!(buffer.len() > 0);

            // check the buffer format:
            let format_ = (*VIPS)
                .find_image_format_from_buffer(&buffer)
                .expect(&format!("can't recognize format from buffer: {}", filename));
            assert_eq!(*format, format_);

            // save to temporary file
            let new_file = dir.path().join(format!("{:?}", format));
            image
                .save_to_file(&new_file, format)
                .expect(&format!("Error saving file: {:?}", format));
            // Test detection from file: find_image_format_from_file
            let format_ = (*VIPS)
                .find_image_format_from_file(&new_file)
                .expect(&format!("can't recognize format from file: {}", filename));
            assert_eq!(format_, *format);
        }
    }
}
