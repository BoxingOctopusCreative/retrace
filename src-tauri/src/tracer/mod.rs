use serde::{Deserialize, Serialize};

pub mod custom;
pub mod live;
pub mod sidecar;
pub mod starvector;
pub mod vtracer;

#[derive(Debug)]
pub struct RasterImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceOptions {
    pub color_precision: u8,
    pub filter_speckle: u32,
    pub corner_threshold: f64,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            color_precision: 6,
            filter_speckle: 4,
            corner_threshold: 60.0,
        }
    }
}

pub struct VectorOutput {
    pub svg: String,
}

pub trait ImageTracer: Send + Sync {
    fn trace(&self, input: &RasterImage, opts: &TraceOptions) -> anyhow::Result<VectorOutput>;
    fn name(&self) -> &str;
}
