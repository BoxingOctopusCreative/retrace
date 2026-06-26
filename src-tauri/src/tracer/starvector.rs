use super::{ImageTracer, RasterImage, TraceOptions, VectorOutput};
use crate::tracer::sidecar::call_sidecar;

pub enum StarVectorTier {
    OneB,
    EightB,
}

pub struct StarVectorBackend {
    pub app: tauri::AppHandle,
    pub tier: StarVectorTier,
}

impl ImageTracer for StarVectorBackend {
    fn trace(&self, input: &RasterImage, _opts: &TraceOptions) -> anyhow::Result<VectorOutput> {
        let backend = match self.tier {
            StarVectorTier::OneB => "starvector-1b",
            StarVectorTier::EightB => "starvector-8b",
        };
        call_sidecar(&self.app, backend, input)
    }

    fn name(&self) -> &str {
        match self.tier {
            StarVectorTier::OneB => "starvector-1b",
            StarVectorTier::EightB => "starvector-8b",
        }
    }
}
