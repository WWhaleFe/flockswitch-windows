//! Every build gets the app icon at 96 px for the settings window. Windows builds also
//! get version info (product name/version — required by SignPath), an application
//! manifest and an .ico, all made from packaging/icon/flockswitch.png.

#[allow(dead_code)]
#[path = "packaging/icon/master.rs"]
mod master;

use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=packaging/icon/flockswitch.png");
    println!("cargo:rerun-if-changed=packaging/icon/master.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let window_icon = master::scaled(Path::new(MASTER), WINDOW_ICON);
    std::fs::write(out.join("window-icon.rgba"), window_icon).expect("writing window icon");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let ico = out.join("flockswitch.ico");
    write_ico(&ico).expect("writing icon");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico.to_str().unwrap());
    // FileVersion / ProductVersion come from CARGO_PKG_VERSION automatically.
    res.set("ProductName", "flockswitch");
    res.set("FileDescription", "flockswitch - move mouse and keyboard to another computer together");
    res.set("OriginalFilename", "flockswitch.exe");
    res.set("InternalName", "flockswitch");
    res.set("LegalCopyright", "Copyright (c) 2026 WWhaleFe. MIT License.");
    res.set_manifest(MANIFEST);
    res.compile().expect("compiling Windows resources");
}

const MASTER: &str = "packaging/icon/flockswitch.png";
/// Side of the RGBA icon in OUT_DIR/window-icon.rgba (kept in sync with settings_ui.rs).
const WINDOW_ICON: u32 = 96;

/// Modern common controls (menus look native) and per-monitor DPI awareness (sharp icon).
const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity type="win32" name="flockswitch" version="1.0.0.0"/>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#;

/// Writes a multi-size .ico with 32-bit BMP images of the app icon.
fn write_ico(path: &Path) -> std::io::Result<()> {
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let master = Path::new(MASTER);
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&s| bmp_image(s, &master::scaled(master, s)))
        .collect();

    let mut f = std::fs::File::create(path)?;
    f.write_all(&[0, 0, 1, 0])?; // reserved, type = icon
    f.write_all(&(sizes.len() as u16).to_le_bytes())?;
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (&s, img) in sizes.iter().zip(&images) {
        let dim = if s >= 256 { 0 } else { s as u8 };
        f.write_all(&[dim, dim, 0, 0])?; // width, height, palette, reserved
        f.write_all(&1u16.to_le_bytes())?; // planes
        f.write_all(&32u16.to_le_bytes())?; // bits per pixel
        f.write_all(&(img.len() as u32).to_le_bytes())?;
        f.write_all(&offset.to_le_bytes())?;
        offset += img.len() as u32;
    }
    for img in &images {
        f.write_all(img)?;
    }
    Ok(())
}

/// One icon image: BITMAPINFOHEADER, bottom-up BGRA pixels, then an empty AND mask.
fn bmp_image(size: u32, rgba: &[u8]) -> Vec<u8> {
    let mask_row = size.div_ceil(32) * 4;
    let mut v = Vec::new();
    v.extend_from_slice(&40u32.to_le_bytes());
    v.extend_from_slice(&(size as i32).to_le_bytes());
    v.extend_from_slice(&((size * 2) as i32).to_le_bytes()); // XOR + AND heights
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&32u16.to_le_bytes());
    v.extend_from_slice(&[0u8; 24]); // no compression, sizes and palette unused
    for y in (0..size).rev() {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            v.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
        }
    }
    v.extend(std::iter::repeat(0u8).take((mask_row * size) as usize));
    v
}
