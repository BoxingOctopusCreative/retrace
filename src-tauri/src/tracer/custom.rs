use super::{ImageTracer, RasterImage, TraceOptions, VectorOutput};

pub struct CustomTracer;

impl ImageTracer for CustomTracer {
    fn trace(&self, _input: &RasterImage, _opts: &TraceOptions) -> anyhow::Result<VectorOutput> {
        anyhow::bail!("custom tracer not yet implemented")
    }

    fn name(&self) -> &str {
        "custom"
    }
}
