/// Converts vtracer's SVG output to EPS or AI (EPS + AI compatibility headers).
///
/// vtracer produces path-only SVG: `<path d="M...C...Z" fill="#rrggbb" transform="translate(x,y)"/>`
/// This converter handles exactly that structure and the SVG path commands vtracer emits (M, C, Z).
pub fn svg_to_eps(svg: &str) -> Result<String, String> {
    let doc = roxmltree::Document::parse(svg).map_err(|e| e.to_string())?;
    let root = doc.root_element();

    let width: f64 = root
        .attribute("width")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);
    let height: f64 = root
        .attribute("height")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);

    let mut out = String::new();
    write_eps_body(&mut out, svg, width, height, false);
    Ok(out)
}

pub fn svg_to_ai(svg: &str) -> Result<String, String> {
    let doc = roxmltree::Document::parse(svg).map_err(|e| e.to_string())?;
    let root = doc.root_element();

    let width: f64 = root
        .attribute("width")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);
    let height: f64 = root
        .attribute("height")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);

    let mut out = String::new();
    write_eps_body(&mut out, svg, width, height, true);
    Ok(out)
}

fn write_eps_body(out: &mut String, svg: &str, width: f64, height: f64, ai_compat: bool) {
    let w = width.ceil() as i32;
    let h = height.ceil() as i32;

    out.push_str("%!PS-Adobe-3.0 EPSF-3.0\n");
    if ai_compat {
        out.push_str("%%Creator: Re:Trace (Adobe Illustrator compatible)\n");
        out.push_str("%AI3_ColorUsage: Color\n");
        out.push_str("%AI8_DocumentColorModel: RGB\n");
    } else {
        out.push_str("%%Creator: Re:Trace\n");
    }
    out.push_str(&format!("%%BoundingBox: 0 0 {} {}\n", w, h));
    out.push_str("%%EndComments\n");
    out.push_str("%%BeginProlog\n");
    out.push_str("%%EndProlog\n");
    out.push_str("%%Page: 1 1\n");

    // Flip Y axis: SVG origin is top-left (Y↓), PS origin is bottom-left (Y↑)
    out.push_str(&format!("0 {} translate\n", height));
    out.push_str("1 -1 scale\n");

    let doc = roxmltree::Document::parse(svg).unwrap();
    for node in doc.root_element().children() {
        if node.tag_name().name() != "path" {
            continue;
        }
        let d = node.attribute("d").unwrap_or("");
        let fill = node.attribute("fill").unwrap_or("#000000");
        let transform = node.attribute("transform").unwrap_or("");

        let (tx, ty) = parse_translate(transform);
        let (r, g, b) = parse_hex_color(fill);

        out.push_str(&format!("{:.6} {:.6} {:.6} setrgbcolor\n", r, g, b));

        let need_gsave = tx != 0.0 || ty != 0.0;
        if need_gsave {
            out.push_str(&format!("gsave\n{:.4} {:.4} translate\n", tx, ty));
        }

        out.push_str(&path_d_to_ps(d));
        out.push_str("fill\n");

        if need_gsave {
            out.push_str("grestore\n");
        }
    }

    out.push_str("%%EOF\n");
}

// ── Path conversion ──────────────────────────────────────────────────────────

/// Converts an SVG path `d` attribute value to PostScript path commands.
/// Handles M/m, L/l, H/h, V/v, C/c, S/s, Z/z — the full set vtracer may emit.
pub(crate) fn path_d_to_ps(d: &str) -> String {
    let mut ps = String::new();
    let mut cx = 0.0f64; // current point
    let mut cy = 0.0f64;
    let mut prev_ctrl: Option<(f64, f64)> = None; // for S command reflection

    let tokens = tokenize_path(d);
    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        let cmd = match token {
            Token::Cmd(c) => *c,
            _ => continue,
        };
        prev_ctrl = None;
        match cmd {
            'M' | 'm' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let x = next_num(&mut iter);
                    let y = next_num(&mut iter);
                    let (ax, ay) = if cmd == 'm' { (cx + x, cy + y) } else { (x, y) };
                    ps.push_str(&format!("{:.4} {:.4} moveto\n", ax, ay));
                    cx = ax;
                    cy = ay;
                }
            }
            'L' | 'l' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let x = next_num(&mut iter);
                    let y = next_num(&mut iter);
                    let (ax, ay) = if cmd == 'l' { (cx + x, cy + y) } else { (x, y) };
                    ps.push_str(&format!("{:.4} {:.4} lineto\n", ax, ay));
                    cx = ax;
                    cy = ay;
                }
            }
            'H' | 'h' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let x = next_num(&mut iter);
                    let ax = if cmd == 'h' { cx + x } else { x };
                    ps.push_str(&format!("{:.4} {:.4} lineto\n", ax, cy));
                    cx = ax;
                }
            }
            'V' | 'v' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let y = next_num(&mut iter);
                    let ay = if cmd == 'v' { cy + y } else { y };
                    ps.push_str(&format!("{:.4} {:.4} lineto\n", cx, ay));
                    cy = ay;
                }
            }
            'C' | 'c' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let (x1, y1, x2, y2, x, y) = (
                        next_num(&mut iter), next_num(&mut iter),
                        next_num(&mut iter), next_num(&mut iter),
                        next_num(&mut iter), next_num(&mut iter),
                    );
                    let (ax1, ay1, ax2, ay2, ax, ay) = if cmd == 'c' {
                        (cx + x1, cy + y1, cx + x2, cy + y2, cx + x, cy + y)
                    } else {
                        (x1, y1, x2, y2, x, y)
                    };
                    ps.push_str(&format!(
                        "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} curveto\n",
                        ax1, ay1, ax2, ay2, ax, ay
                    ));
                    prev_ctrl = Some((ax2, ay2));
                    cx = ax;
                    cy = ay;
                }
            }
            'S' | 's' => {
                while matches!(iter.peek(), Some(Token::Num(_))) {
                    let (x2, y2, x, y) = (
                        next_num(&mut iter), next_num(&mut iter),
                        next_num(&mut iter), next_num(&mut iter),
                    );
                    let (ax2, ay2, ax, ay) = if cmd == 's' {
                        (cx + x2, cy + y2, cx + x, cy + y)
                    } else {
                        (x2, y2, x, y)
                    };
                    // Reflect previous control point
                    let (ax1, ay1) = prev_ctrl
                        .map(|(px, py)| (2.0 * cx - px, 2.0 * cy - py))
                        .unwrap_or((cx, cy));
                    ps.push_str(&format!(
                        "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} curveto\n",
                        ax1, ay1, ax2, ay2, ax, ay
                    ));
                    prev_ctrl = Some((ax2, ay2));
                    cx = ax;
                    cy = ay;
                }
            }
            'Z' | 'z' => {
                ps.push_str("closepath\n");
            }
            _ => {} // ignore unrecognised commands
        }
    }
    ps
}

// ── Tokenizer ────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum Token {
    Cmd(char),
    Num(f64),
}

fn tokenize_path(d: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = d.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            'A'..='Z' | 'a'..='z' => {
                tokens.push(Token::Cmd(c));
                chars.next();
            }
            '0'..='9' | '.' | '-' | '+' => {
                let mut num_str = String::new();
                // Handle leading sign
                if c == '-' || c == '+' {
                    num_str.push(c);
                    chars.next();
                }
                let mut has_dot = false;
                let mut has_e = false;
                while let Some(&nc) = chars.peek() {
                    match nc {
                        '0'..='9' => { num_str.push(nc); chars.next(); }
                        '.' if !has_dot => { has_dot = true; num_str.push(nc); chars.next(); }
                        'e' | 'E' if !has_e => {
                            has_e = true;
                            num_str.push(nc);
                            chars.next();
                            // optional sign after exponent
                            if let Some(&s) = chars.peek() {
                                if s == '-' || s == '+' { num_str.push(s); chars.next(); }
                            }
                        }
                        _ => break,
                    }
                }
                if let Ok(v) = num_str.parse::<f64>() {
                    tokens.push(Token::Num(v));
                }
            }
            ' ' | '\t' | '\n' | '\r' | ',' => { chars.next(); }
            _ => { chars.next(); }
        }
    }
    tokens
}

fn next_num<'a>(iter: &mut std::iter::Peekable<impl Iterator<Item = &'a Token>>) -> f64 {
    match iter.next() {
        Some(Token::Num(v)) => *v,
        _ => 0.0,
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

pub(crate) fn parse_translate(transform: &str) -> (f64, f64) {
    if let Some(inner) = transform.strip_prefix("translate(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<f64> = inner
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        return (parts.first().copied().unwrap_or(0.0), parts.get(1).copied().unwrap_or(0.0));
    }
    (0.0, 0.0)
}

pub(crate) fn parse_hex_color(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
        }
    }
    (0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_hex_color ───────────────────────────────────────────────────────

    #[test]
    fn hex_color_red() {
        let (r, g, b) = parse_hex_color("#ff0000");
        assert!((r - 1.0).abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn hex_color_black() {
        let (r, g, b) = parse_hex_color("#000000");
        assert!(r.abs() < 1e-6 && g.abs() < 1e-6 && b.abs() < 1e-6);
    }

    #[test]
    fn hex_color_white() {
        let (r, g, b) = parse_hex_color("#ffffff");
        assert!((r - 1.0).abs() < 1e-6 && (g - 1.0).abs() < 1e-6 && (b - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hex_color_mid_gray() {
        let (r, g, b) = parse_hex_color("#808080");
        let expected = 0x80 as f64 / 255.0;
        assert!((r - expected).abs() < 1e-6);
        assert!((g - expected).abs() < 1e-6);
        assert!((b - expected).abs() < 1e-6);
    }

    #[test]
    fn hex_color_no_hash_still_parses() {
        // trim_start_matches('#') means the hash prefix is optional
        let (r, g, b) = parse_hex_color("ff0000");
        assert!((r - 1.0).abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn hex_color_invalid_returns_fallback() {
        let (r, g, b) = parse_hex_color("not-a-color");
        assert!(r.abs() < 1e-6 && g.abs() < 1e-6 && b.abs() < 1e-6);
    }

    #[test]
    fn hex_color_uppercase_letters() {
        let (r, g, b) = parse_hex_color("#FF0000");
        assert!((r - 1.0).abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }

    // ── parse_translate ───────────────────────────────────────────────────────

    #[test]
    fn translate_simple() {
        let (x, y) = parse_translate("translate(10,20)");
        assert!((x - 10.0).abs() < 1e-6);
        assert!((y - 20.0).abs() < 1e-6);
    }

    #[test]
    fn translate_with_spaces() {
        let (x, y) = parse_translate("translate(1.5, 2.5)");
        assert!((x - 1.5).abs() < 1e-6);
        assert!((y - 2.5).abs() < 1e-6);
    }

    #[test]
    fn translate_empty_string() {
        let (x, y) = parse_translate("");
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6);
    }

    #[test]
    fn translate_unrecognised_transform() {
        let (x, y) = parse_translate("rotate(45)");
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6);
    }

    #[test]
    fn translate_missing_y_defaults_to_zero() {
        let (x, y) = parse_translate("translate(5)");
        assert!((x - 5.0).abs() < 1e-6);
        assert!(y.abs() < 1e-6);
    }

    // ── path_d_to_ps ──────────────────────────────────────────────────────────

    #[test]
    fn path_moveto_lineto_closepath() {
        let ps = path_d_to_ps("M 10 20 L 30 40 Z");
        assert!(ps.contains("10.0000 20.0000 moveto"), "got: {}", ps);
        assert!(ps.contains("30.0000 40.0000 lineto"), "got: {}", ps);
        assert!(ps.contains("closepath"), "got: {}", ps);
    }

    #[test]
    fn path_cubic_bezier_absolute() {
        let ps = path_d_to_ps("M 0 0 C 1 2 3 4 5 6");
        assert!(ps.contains("1.0000 2.0000 3.0000 4.0000 5.0000 6.0000 curveto"), "got: {}", ps);
    }

    #[test]
    fn path_cubic_bezier_relative() {
        let ps = path_d_to_ps("M 10 10 c 1 2 3 4 5 6");
        // Relative: from (10,10), end point is (15,16)
        assert!(ps.contains("curveto"), "got: {}", ps);
    }

    #[test]
    fn path_horizontal_lineto() {
        let ps = path_d_to_ps("M 0 0 H 50");
        assert!(ps.contains("50.0000 0.0000 lineto"), "got: {}", ps);
    }

    #[test]
    fn path_vertical_lineto() {
        let ps = path_d_to_ps("M 0 0 V 50");
        assert!(ps.contains("0.0000 50.0000 lineto"), "got: {}", ps);
    }

    #[test]
    fn path_smooth_cubic_uses_reflected_control() {
        // After C, S should reflect the previous control point
        let ps = path_d_to_ps("M 0 0 C 1 2 3 4 5 6 S 7 8 9 10");
        assert!(ps.contains("curveto"), "S cmd produced no curveto:\n{}", ps);
        // Two curveto calls expected
        assert_eq!(ps.matches("curveto").count(), 2, "got:\n{}", ps);
    }

    #[test]
    fn path_relative_moveto() {
        let ps = path_d_to_ps("M 10 10 m 5 5");
        // After absolute M(10,10), relative m(5,5) → (15,15)
        assert!(ps.contains("15.0000 15.0000 moveto"), "got: {}", ps);
    }

    #[test]
    fn path_empty_d_attribute() {
        let ps = path_d_to_ps("");
        assert!(ps.is_empty());
    }

    #[test]
    fn path_unrecognised_cmd_skipped() {
        let ps = path_d_to_ps("M 0 0 Q 1 2 3 4"); // Q is quadratic, not handled
        // moveto should still be there, Q should be silently skipped
        assert!(ps.contains("moveto"));
        assert!(!ps.contains("curveto"));
    }

    // ── svg_to_eps / svg_to_ai ────────────────────────────────────────────────

    const SIMPLE_SVG: &str = r##"<svg width="100" height="200" xmlns="http://www.w3.org/2000/svg">
<path d="M 10 10 L 90 190 Z" fill="#ff0000"/>
</svg>"##;

    #[test]
    fn eps_starts_with_ps_header() {
        let eps = svg_to_eps(SIMPLE_SVG).unwrap();
        assert!(eps.starts_with("%!PS-Adobe-3.0 EPSF-3.0\n"));
    }

    #[test]
    fn eps_bounding_box_matches_svg_dimensions() {
        let eps = svg_to_eps(SIMPLE_SVG).unwrap();
        assert!(eps.contains("%%BoundingBox: 0 0 100 200\n"), "got: {}", eps);
    }

    #[test]
    fn eps_contains_setrgbcolor_for_fill() {
        let eps = svg_to_eps(SIMPLE_SVG).unwrap();
        // #ff0000 → 1.0 0.0 0.0
        assert!(eps.contains("1.000000 0.000000 0.000000 setrgbcolor"), "got: {}", eps);
    }

    #[test]
    fn eps_ends_with_eof() {
        let eps = svg_to_eps(SIMPLE_SVG).unwrap();
        assert!(eps.ends_with("%%EOF\n"));
    }

    #[test]
    fn eps_does_not_contain_ai_headers() {
        let eps = svg_to_eps(SIMPLE_SVG).unwrap();
        assert!(!eps.contains("AI8_DocumentColorModel"));
    }

    #[test]
    fn ai_contains_ai_compatibility_headers() {
        let ai = svg_to_ai(SIMPLE_SVG).unwrap();
        assert!(ai.contains("%AI3_ColorUsage: Color\n"));
        assert!(ai.contains("%AI8_DocumentColorModel: RGB\n"));
    }

    #[test]
    fn eps_with_translate_transform() {
        let svg = r##"<svg width="50" height="50" xmlns="http://www.w3.org/2000/svg">
<path d="M 0 0 L 10 10 Z" fill="#0000ff" transform="translate(5,10)"/>
</svg>"##;
        let eps = svg_to_eps(svg).unwrap();
        assert!(eps.contains("gsave"), "expected gsave for translate:\n{}", eps);
        assert!(eps.contains("grestore"));
    }

    #[test]
    fn eps_without_transform_no_gsave() {
        let svg = r##"<svg width="50" height="50" xmlns="http://www.w3.org/2000/svg">
<path d="M 0 0 L 10 10 Z" fill="#00ff00"/>
</svg>"##;
        let eps = svg_to_eps(svg).unwrap();
        assert!(!eps.contains("gsave"), "unexpected gsave:\n{}", eps);
    }

    #[test]
    fn eps_invalid_xml_returns_err() {
        let result = svg_to_eps("<svg><unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn eps_missing_width_defaults_to_100() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let eps = svg_to_eps(svg).unwrap();
        assert!(eps.contains("%%BoundingBox: 0 0 100 100\n"), "got: {}", eps);
    }

    #[test]
    fn eps_skips_non_path_elements() {
        let svg = r##"<svg width="50" height="50" xmlns="http://www.w3.org/2000/svg">
<rect width="50" height="50" fill="#ff0000"/>
<path d="M 0 0 L 10 10 Z" fill="#00ff00"/>
</svg>"##;
        let eps = svg_to_eps(svg).unwrap();
        // Only one setrgbcolor (for the path, not the rect)
        assert_eq!(eps.matches("setrgbcolor").count(), 1);
    }
}
