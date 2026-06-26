pub mod eps;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Svg,
    Eps,
    Ai,
}

pub fn convert(svg: &str, format: &ExportFormat) -> Result<String, String> {
    match format {
        ExportFormat::Svg => Ok(svg.to_string()),
        ExportFormat::Eps => eps::svg_to_eps(svg),
        ExportFormat::Ai => eps::svg_to_ai(svg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SVG: &str = r##"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg"><path d="M10 10 L90 90 Z" fill="#ff0000"/></svg>"##;

    #[test]
    fn svg_format_returns_unchanged_content() {
        let result = convert(SIMPLE_SVG, &ExportFormat::Svg).unwrap();
        assert_eq!(result, SIMPLE_SVG);
    }

    #[test]
    fn eps_format_produces_postscript_header() {
        let result = convert(SIMPLE_SVG, &ExportFormat::Eps).unwrap();
        assert!(result.starts_with("%!PS-Adobe-3.0 EPSF-3.0\n"));
        assert!(result.contains("%%BoundingBox: 0 0 100 100\n"));
        assert!(result.ends_with("%%EOF\n"));
    }

    #[test]
    fn eps_format_does_not_contain_ai_headers() {
        let result = convert(SIMPLE_SVG, &ExportFormat::Eps).unwrap();
        assert!(!result.contains("AI8_DocumentColorModel"));
    }

    #[test]
    fn ai_format_contains_ai_compatibility_headers() {
        let result = convert(SIMPLE_SVG, &ExportFormat::Ai).unwrap();
        assert!(result.contains("AI8_DocumentColorModel: RGB\n"));
        assert!(result.contains("%AI3_ColorUsage: Color\n"));
    }

    #[test]
    fn format_enum_deserializes_from_lowercase() {
        assert!(matches!(
            serde_json::from_str::<ExportFormat>(r#""svg""#).unwrap(),
            ExportFormat::Svg
        ));
        assert!(matches!(
            serde_json::from_str::<ExportFormat>(r#""eps""#).unwrap(),
            ExportFormat::Eps
        ));
        assert!(matches!(
            serde_json::from_str::<ExportFormat>(r#""ai""#).unwrap(),
            ExportFormat::Ai
        ));
    }
}
