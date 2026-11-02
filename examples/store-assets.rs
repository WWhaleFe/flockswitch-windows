//! Writes the images of the Microsoft Store package (MSIX visual assets) from
//! packaging/icon/flockswitch.png. Used by packaging/windows/store/make-msix.ps1.
//!
//!     cargo run --example store-assets -- <package-dir>/Assets
//!
//! Each image comes in the scales and target sizes Windows picks from (resources.pri,
//! made by makepri, maps the qualified names): the Start menu tile, the app list and
//! taskbar icon, and the Store logo.

#[allow(dead_code)]
#[path = "../packaging/icon/master.rs"]
mod master;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// (base name, size at scale 100, share of the image the icon fills).
const IMAGES: &[(&str, u32, f32)] = &[
    // The Start tile leaves room around the icon, as Windows' own tiles do.
    ("Square150x150Logo", 150, 0.66),
    ("Square44x44Logo", 44, 1.0),
    ("StoreLogo", 50, 1.0),
];
const SCALES: &[u32] = &[100, 125, 150, 200, 400];
/// The taskbar, Start and Explorer icon sizes, drawn without a plate behind them.
const TARGET_SIZES: &[u32] = &[16, 24, 32, 48, 256];

fn main() {
    let out = std::env::args().nth(1).expect("usage: store-assets <out-dir>");
    std::fs::create_dir_all(&out).expect("creating output folder");
    let master = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/packaging/icon/flockswitch.png"));
    for &(name, base, fill) in IMAGES {
        for &scale in SCALES {
            let size = base * scale / 100;
            write(&format!("{out}/{name}.scale-{scale}.png"), size, &padded(master, size, fill));
        }
    }
    for &size in TARGET_SIZES {
        let image = master::scaled(master, size);
        for suffix in ["", "_altform-unplated", "_altform-lightunplated"] {
            write(&format!("{out}/Square44x44Logo.targetsize-{size}{suffix}.png"), size, &image);
        }
    }
}

/// The icon at `fill` of a transparent `size`×`size` square, centred.
fn padded(master: &Path, size: u32, fill: f32) -> Vec<u8> {
    let inner = ((size as f32 * fill).round() as u32).clamp(1, size);
    if inner == size {
        return master::scaled(master, size);
    }
    let icon = master::scaled(master, inner);
    let offset = (size - inner) / 2;
    let mut canvas = vec![0u8; (size * size * 4) as usize];
    for y in 0..inner {
        let src = &icon[(y * inner * 4) as usize..][..(inner * 4) as usize];
        let at = (((y + offset) * size + offset) * 4) as usize;
        canvas[at..at + src.len()].copy_from_slice(src);
    }
    canvas
}

fn write(path: &str, size: u32, rgba: &[u8]) {
    let file = File::create(path).expect("creating png");
    let mut enc = png::Encoder::new(BufWriter::new(file), size, size);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut w| w.write_image_data(rgba)).expect("writing png");
}
