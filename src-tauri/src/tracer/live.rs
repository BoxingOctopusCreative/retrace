use super::{ImageTracer, RasterImage, TraceOptions, VectorOutput};
use crate::tracer::sidecar::call_sidecar;

pub struct LiveBackend {
    pub app: tauri::AppHandle,
}

impl ImageTracer for LiveBackend {
    fn trace(&self, input: &RasterImage, _opts: &TraceOptions) -> anyhow::Result<VectorOutput> {
        call_sidecar(&self.app, "live", input)
    }

    fn name(&self) -> &str {
        "live"
    }

    fn is_slow(&self) -> bool {
        true
    }
}
