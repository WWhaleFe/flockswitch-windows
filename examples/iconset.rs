//! Writes the macOS app icon set (PNG files for `iconutil`) from
//! packaging/icon/flockswitch.png. Used by packaging/macos/make-app.sh.
//!
//!     cargo run --example iconset -- <out-dir>/AppIcon.iconset

#[allow(dead_code)]
#[path = "../packaging/icon/master.rs"]
mod master;

use std::fs::File;
use std::io::BufWriter;

const SIZES: &[(&str, u32)] = &[
    ("icon_16x16", 16),
    ("icon_16x16@2x", 32),
    ("icon_32x32", 32),
    ("icon_32x32@2x", 64),
    ("icon_128x128", 128),
    ("icon_128x128@2x", 256),
    ("icon_256x256", 256),
    ("icon_256x256@2x", 512),
    ("icon_512x512", 512),
    ("icon_512x512@2x", 1024),
];

fn main() {
    let out = std::env::args().nth(1).expect("usage: iconset <out-dir>");
    std::fs::create_dir_all(&out).expect("creating output folder");
    let master = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/packaging/icon/flockswitch.png"));
    for &(name, size) in SIZES {
        let canvas = master::scaled(master, size);
        let file = File::create(format!("{out}/{name}.png")).expect("creating png");
        let mut enc = png::Encoder::new(BufWriter::new(file), size, size);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().and_then(|mut w| w.write_image_data(&canvas)).expect("writing png");
    }
}
