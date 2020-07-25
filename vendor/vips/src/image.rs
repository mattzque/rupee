// Rupee - Rust Image Service
// Copyright (C) 2019 Matthias Hecker
// Licensed under the Apache License, Version 2.0, or the MIT License

use super::{path_to_cstring, Vips, VipsError, VipsFormat};
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;

/// API: https://jcupitt.github.io/libvips/API/current/libvips-resample.html#VipsSize
#[derive(Debug, Clone, PartialEq)]
pub enum VipsSizeMode {
    /// size both up and down
    VipsSizeBoth,
    /// only upsize
    VipsSizeUp,
    /// only downsize
    VipsSizeDown,
    /// force size, that is, break aspect ratio
    VipsSizeForce,
}

impl VipsSizeMode {
    fn to_lib_int(&self) -> vips_sys::VipsSize {
        match *self {
            VipsSizeMode::VipsSizeBoth => vips_sys::VipsSize_VIPS_SIZE_BOTH,
            VipsSizeMode::VipsSizeUp => vips_sys::VipsSize_VIPS_SIZE_UP,
            VipsSizeMode::VipsSizeDown => vips_sys::VipsSize_VIPS_SIZE_DOWN,
            VipsSizeMode::VipsSizeForce => vips_sys::VipsSize_VIPS_SIZE_FORCE,
        }
    }
}

impl Default for VipsSizeMode {
    fn default() -> Self {
        VipsSizeMode::VipsSizeForce
    }
}

/// API: https://jcupitt.github.io/libvips/API/current/libvips-conversion.html#VipsInteresting
#[derive(Debug, Clone, PartialEq)]
pub enum VipsCropMode {
    /// do nothing
    VipsCropNone,
    /// just take the centre
    VipsCropCenter,
    /// use an entropy measure
    VipsCropEntropy,
    /// look for features likely to draw human attention
    VipsCropAttention,
}

impl VipsCropMode {
    fn to_lib_int(&self) -> vips_sys::VipsSize {
        match *self {
            VipsCropMode::VipsCropNone => vips_sys::VipsInteresting_VIPS_INTERESTING_NONE,
            VipsCropMode::VipsCropCenter => vips_sys::VipsInteresting_VIPS_INTERESTING_CENTRE,
            VipsCropMode::VipsCropEntropy => vips_sys::VipsInteresting_VIPS_INTERESTING_ENTROPY,
            VipsCropMode::VipsCropAttention => vips_sys::VipsInteresting_VIPS_INTERESTING_ATTENTION,
        }
    }
}

impl Default for VipsCropMode {
    fn default() -> Self {
        VipsCropMode::VipsCropNone
    }
}

pub struct Image<'a> {
    vips: &'a Vips,
    vips_image: *const vips_sys::VipsImage,
}

impl<'a> Image<'a> {
    /// Create a new Image instance.
    ///
    /// You are not supposed to call this method yourself, instead use one of the methods
    /// on an instance of [Vips](vips::Vips) to create an Image instance.
    pub(super) fn new(vips: &'a Vips, vips_image: *const vips_sys::VipsImage) -> Image {
        // borrow vips instance, move vips image pointer into the newly created struct
        Image { vips, vips_image }
    }

    /// Returns the width of the image.
    pub fn width(&self) -> i32 {
        unsafe { vips_sys::vips_image_get_width(self.vips_image) }
    }

    /// Returns the height of the image.
    pub fn height(&self) -> i32 {
        unsafe { vips_sys::vips_image_get_height(self.vips_image) }
    }

    /// Returns the number of channels (bands) in the image.
    pub fn channel(&self) -> i32 {
        unsafe { vips_sys::vips_image_get_bands(self.vips_image) }
    }

    /// Resize the image to the given dimension.
    /// Note: This automatically applies EXIF rotation!
    pub fn resize(
        &self,
        width: i32,
        height: i32,
        size_mode: Option<VipsSizeMode>,
        crop_mode: Option<VipsCropMode>,
    ) -> Result<Image, VipsError> {
        let vips_image: *mut _ = self.vips_image as *mut vips_sys::VipsImage;
        let mut new_vips_image: *mut vips_sys::VipsImage = ptr::null_mut();
        let size_mode = size_mode.unwrap_or(VipsSizeMode::default());
        let crop_mode = crop_mode.unwrap_or(VipsCropMode::default());

        let result = unsafe {
            vips_sys::vips_thumbnail_image(
                vips_image,
                &mut new_vips_image,
                width,
                // param pairs null delimited
                "height\0".as_ptr(),
                height,
                "size\0".as_ptr(),
                size_mode.to_lib_int(),
                "crop\0".as_ptr(),
                crop_mode.to_lib_int(),
                ptr::null() as *const c_void,
            )
        };

        if result != 0 {
            return Err(VipsError::VipsImageResizeError);
        }

        Ok(Self {
            vips: &self.vips,
            vips_image: new_vips_image,
        })
    }

    pub fn save_to_buffer(&self, format: &VipsFormat) -> Result<Vec<u8>, VipsError> {
        let vips_image: *mut _ = self.vips_image as *mut vips_sys::VipsImage;
        let mut new_buffer: *mut c_void = ptr::null_mut();
        let mut new_len: usize = 0;
        let null = ptr::null() as *const c_void;

        let result = match format {
            VipsFormat::VipsJpeg => unsafe {
                vips_sys::vips_jpegsave_buffer(vips_image, &mut new_buffer, &mut new_len, null)
            },
            VipsFormat::VipsPng => unsafe {
                vips_sys::vips_pngsave_buffer(vips_image, &mut new_buffer, &mut new_len, null)
            },
            VipsFormat::VipsGif => unsafe {
                vips_sys::vips_magicksave_buffer(
                    vips_image,
                    &mut new_buffer,
                    &mut new_len,
                    "format\0".as_ptr(),
                    "GIF\0".as_ptr(),
                    null,
                )
            },
            VipsFormat::VipsWebP => unsafe {
                vips_sys::vips_webpsave_buffer(vips_image, &mut new_buffer, &mut new_len, null)
            },
            VipsFormat::VipsSvg => unsafe {
                vips_sys::vips_magicksave_buffer(
                    vips_image,
                    &mut new_buffer,
                    &mut new_len,
                    "format\0".as_ptr(),
                    "SVG\0".as_ptr(),
                    null,
                )
            },
        };

        if result != 0 {
            eprintln!("{}", self.vips.read_error_buffer());
            return Err(VipsError::VipsImageWriteError);
        }

        Ok(unsafe { Vec::from_raw_parts(new_buffer as *mut u8, new_len, new_len) })
    }

    pub fn save_to_file(&self, filename: &Path, format: &VipsFormat) -> Result<(), VipsError> {
        let filename = path_to_cstring(&filename);

        let vips_image: *mut _ = self.vips_image as *mut vips_sys::VipsImage;
        let null = ptr::null() as *const c_void;

        let result = match format {
            VipsFormat::VipsJpeg => unsafe {
                vips_sys::vips_jpegsave(vips_image, filename.as_ptr(), null)
            },
            VipsFormat::VipsPng => unsafe {
                vips_sys::vips_pngsave(vips_image, filename.as_ptr(), null)
            },
            VipsFormat::VipsGif => unsafe {
                vips_sys::vips_magicksave(
                    vips_image,
                    filename.as_ptr(),
                    "format\0".as_ptr(),
                    "GIF\0".as_ptr(),
                    null,
                )
            },
            VipsFormat::VipsWebP => unsafe {
                vips_sys::vips_webpsave(vips_image, filename.as_ptr(), null)
            },
            VipsFormat::VipsSvg => unsafe {
                vips_sys::vips_magicksave(
                    vips_image,
                    filename.as_ptr(),
                    "format\0".as_ptr(),
                    "SVG\0".as_ptr(),
                    null,
                )
            },
        };

        if result != 0 {
            eprintln!("{}", self.vips.read_error_buffer());
            return Err(VipsError::VipsImageWriteError);
        }

        Ok(())
    }
}

impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe {
            // decrement the reference counted vips_image instance
            vips_sys::g_object_unref(self.vips_image as *mut c_void);
        }
        eprintln!("Vips: Image instance reference count decremented!");
    }
}
