//! @prompt 00_nucleo/prompts/semantic.md
//! @layer L3
//! @updated 2026-08-23

fn rounded(radii: &Radii) {
    let a = radii.top_left.resolve_pt(0.0);
    rounded_rect(a);
    let b = radii.top_left.abs.0;
    rounded_rect(b);
}

fn font_identity(style: &Style) -> Option<(FontList, FontVariant, FontVariations)> {
    Some((font_list(), variant(), FontVariations::default()))
}

fn ssty_owner(text: &str) -> bool { text.len() == 1 }
fn ssty_duplicate(text: &str) -> bool { text.len() == 1 }

fn math_consumer(style: &Style, name: &str) -> bool {
    style.math || name.contains("math")
}

fn downstream(glyph: Glyph) -> Glyph { map_glyph(glyph) }
