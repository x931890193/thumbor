use base64::{ decode_config, encode_config, URL_SAFE_NO_PAD};
use photon_rs::transform::SamplingFilter;
use prost::Message;
use std::convert::TryFrom;
mod abi;
pub use abi::*;
use crate::pb::resize::SampleFilter;


impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

impl From<&ImageSpec> for String {
    fn from(image_spec: &ImageSpec) -> Self {
        let data = image_spec.encode_to_vec();
        encode_config(data, URL_SAFE_NO_PAD)
    }
}

impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = decode_config(value, URL_SAFE_NO_PAD)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

impl filter::Filler {
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            filter::Filler::Unspecified => None,
            filter::Filler::Oceanic => Some("oceanic"),
            filter::Filler::Islands => Some("islands"),
            filter::Filler::Marine => Some("marine"),
        }
    }
}

impl From<SampleFilter> for SamplingFilter {
    fn from(v: SampleFilter) -> Self {
        match v {
            SampleFilter::Undefined => SamplingFilter::Nearest,
            SampleFilter::Nearest => SamplingFilter::Nearest,
            SampleFilter::Triangle => SamplingFilter::Triangle,
            SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            SampleFilter::Gaussian => SamplingFilter::Gaussian,
            SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

impl Spec {
    pub fn new_resize_seam_carve(width: u32, height: u32) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::SeamCarve as i32,
                filter: resize::SampleFilter::Undefined as i32,
            }))
        }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::Normal as i32,
                filter: filter as i32,
            }))
        }
    }

    pub fn new_filer(filter: filter::Filler) -> Self {
        Self {
            data: Some(spec::Data::Filter(Filter {
                filter: filter as i32,
            }))
        }
    }

    pub fn new_water_mark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::Watermark(WaterMark {
                x,
                y
            }))
        }
    }
}