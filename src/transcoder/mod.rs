
//! Transcoding is the process of conversion of some media content encoded in some way
//! into some other encoding. This includes conversion of image file formats (such as
//! JPEG into WebP), conversion of video containers (such as WEBM into MKV), conversion
//! of video or audio codecs (such as H264 into VP9). All conversions can be performed
//! in both directions. We also include things like resizing and cropping in this process
//! even though this is technically not a transcoding.
//! 
//! The transcoding process is implemented using a basic scheme:
//! 
//! MediaType - audio or video
//! MediaDescription - includes a MediaType, MimeType, and more detailed description
//!                    such as container, video coded, audio codec, profiles, bitrate
//!                    or quality for instance. This depends on the concrete type.
//! MediaTransformation - such as resizing to a specific size, etc.
//! 
//! The transformation process can then be triggered using the transcode function:
//! 
//! fn transcode(MediaType, source: MediaDescription, target: MediaDescription, transforms:
//! Vec<MediaTransformation>) -> Result<Transcoder, TranscoderError>
//! 
//! The returned transcoder runs the transcoding process in a seperate thread and
//! provides a convenient interface in blocking and concurrent scenarios.
//! The transcode function does validation on the descriptions and ensures there is
//! a matching encoder implementation available.
//! 
//! 

ResizeTransform {
    width: 200,
    height: 300,
    scale: ScaleMode::ENTROPY,
    crop: CropMode::ENTROPY,

}

transcode(&input, MediaType::Jpeg, MediaType::Png, &transforms)
