use anyhow::anyhow;

use super::{ImageTracer, RasterImage, TraceOptions, VectorOutput};

pub struct VtracerBackend;

impl ImageTracer for VtracerBackend {
    fn trace(&self, input: &RasterImage, opts: &TraceOptions) -> anyhow::Result<VectorOutput> {
        let img = vtracer::ColorImage {
            pixels: input.data.clone(),
            width: input.width as usize,
            height: input.height as usize,
        };

        let config = vtracer::Config {
            filter_speckle: opts.filter_speckle as usize,
            color_precision: opts.color_precision as i32,
            corner_threshold: opts.corner_threshold as i32,
            ..vtracer::Config::default()
        };

        let svg_file = vtracer::convert(img, config).map_err(|e| anyhow!("{}", e))?;
        Ok(VectorOutput {
            svg: format!("{}", svg_file),
        })
    }

    fn name(&self) -> &str {
        "vtracer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracer::{RasterImage, TraceOptions};

    fn solid_rgba(r: u8, g: u8, b: u8, w: u32, h: u32) -> RasterImage {
        let mut data = vec![255u8; (w * h * 4) as usize];
        for i in (0..data.len()).step_by(4) {
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
        }
        RasterImage { data, width: w, height: h }
    }

    fn checkerboard(w: u32, h: u32) -> RasterImage {
        let mut data = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let base = ((y * w + x) * 4) as usize;
                let white = (x + y) % 2 == 0;
                data[base] = if white { 255 } else { 0 };
                data[base + 1] = if white { 255 } else { 0 };
                data[base + 2] = if white { 255 } else { 0 };
                data[base + 3] = 255;
            }
        }
        RasterImage { data, width: w, height: h }
    }

    #[test]
    fn trace_solid_color_produces_svg() {
        let raster = solid_rgba(255, 255, 255, 8, 8);
        let result = VtracerBackend.trace(&raster, &TraceOptions::default());
        assert!(result.is_ok(), "trace failed: {:?}", result.err());
        let svg = result.unwrap().svg;
        assert!(svg.contains("<svg"), "output missing <svg tag:\n{}", svg);
        assert!(svg.contains("</svg>"), "output missing </svg> tag");
    }

    #[test]
    fn trace_checkerboard_produces_paths() {
        let raster = checkerboard(16, 16);
        let result = VtracerBackend.trace(&raster, &TraceOptions::default());
        assert!(result.is_ok());
        let svg = result.unwrap().svg;
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn trace_with_custom_options() {
        let raster = checkerboard(8, 8);
        let opts = TraceOptions {
            color_precision: 4,
            filter_speckle: 2,
            corner_threshold: 45.0,
        };
        let result = VtracerBackend.trace(&raster, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn name_returns_vtracer() {
        assert_eq!(VtracerBackend.name(), "vtracer");
    }
}
