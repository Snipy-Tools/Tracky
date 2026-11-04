use resvg::{tiny_skia, usvg};

pub const SVG: &str = include_str!("assets/tracky.svg");

/// Renders the logo to RGBA pixels.
pub fn rgba(size: u32) -> Vec<u8> {
    let tree = usvg::Tree::from_str(SVG, &usvg::Options::default()).expect("invalid svg");
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("invalid icon size");
    let scale = size as f32 / tree.size().width().max(tree.size().height());
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.take_demultiplied()
}

pub fn data_uri() -> String {
    let encoded = SVG
        .trim()
        .replace('%', "%25")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('"', "%22")
        .replace(['\r', '\n'], " ");
    format!("data:image/svg+xml;utf8,{encoded}")
}
