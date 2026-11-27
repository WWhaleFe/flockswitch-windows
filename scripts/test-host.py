#!/usr/bin/env python3
"""Run the Windows app's unit tests (the lib AND the bin crate) on a macOS/Linux machine.

CI runs `cargo test --locked` on windows-latest, but the bin crate links Windows APIs
and cannot be built on a Mac. Two facts make it testable anyway:
  - the real windows-sys / windows crates compile on any host (their functions are
    only declared, and the tests never call them, so nothing is linked);
  - the bin uses only two things from std::os::windows (OsStrExt::encode_wide and
    CommandExt::creation_flags), which a tiny stub crate (host-os) provides;
  - only the Microsoft Store client (update.rs `mod store`, built on the WinRT crates
    windows / windows-future, which do not compile off Windows) is replaced by a stub.
So this script builds a scratch copy of the real crate under target/host-tests:
  - src/ is copied verbatim, except that `std::os::windows::` becomes `host_os::windows::`
    and update.rs `mod store` becomes a stub (the script fails if a rewrite stops
    matching, so nothing is skipped silently);
  - Cargo.toml dependencies are taken from the real Cargo.toml (the Windows-only ones
    included, eframe on its OpenGL backend), and Cargo.lock is copied from the real one
    so versions match CI; build.rs, packaging/ and netcore are the real ones;
  - the linker is told to leave the (never called) Win32 symbols undefined;
  - everything is offline; CARGO_TARGET_DIR is target/host-tests/build.
Not covered: the Store client in update.rs, code under #[cfg(windows)]
(settings_ui::work_area_fit, which has no test) and anything that needs a real Windows API at run time (all such tests are
#[ignore]d or absent). `cargo check --target x86_64-pc-windows-gnu --all-targets`
(scripts/pre-push.sh) compiles that code for Windows.
"""
from pathlib import Path
import os
import re
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parent.parent
scratch = root / 'target/host-tests'
cargo = Path.home() / '.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo'
if not cargo.is_file():
    cargo = Path(shutil.which('cargo') or 'cargo')


def write_if_changed(path, text):
    """Keeps the modification time of unchanged files, so cargo rebuilds only what changed."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.is_file() or path.read_text() != text:
        path.write_text(text)


def sections(text):
    """The sections of a Cargo.toml as {header: [lines]}."""
    out, current = {'': []}, ''
    for line in text.splitlines():
        m = re.match(r'^\[(.+)\]\s*$', line)
        if m:
            current = m.group(1)
            out.setdefault(current, [])
        else:
            out[current].append(line)
    return out


def dep_lines(lines, skip=()):
    return [l for l in lines if l.strip() and not l.lstrip().startswith('#')
            and not any(l.startswith(s + ' ') for s in skip)]


STORE_STUB = '''mod store {
    use anyhow::{bail, Result};
    pub fn newer() -> Result<Option<String>> {
        bail!("the Store client is not available in host tests")
    }
    pub fn install_now(_owner: Option<isize>) -> Result<()> {
        bail!("the Store client is not available in host tests")
    }
    pub fn install() -> Result<()> {
        bail!("the Store client is not available in host tests")
    }
}
'''

# The shell's recycle bin does not exist here: the stub deletes the file instead.
RECYCLE_STUB = '''fn recycle(path: &Path) -> Recycle {
    if std::fs::remove_file(path).is_ok() { Recycle::Done } else { Recycle::Failed }
}
'''

# 1. src/, with the two std::os::windows paths pointed at the stub crate.
rewritten = 0
for src in sorted((root / 'src').rglob('*.rs')):
    text = src.read_text()
    if 'std::os::windows::' in text:
        rewritten += 1
        text = text.replace('std::os::windows::', 'host_os::windows::')
    if src.name == 'update.rs':
        text, n = re.subn(r'^/// The Store version\'s updates, through the Store\.\nmod store \{.*?^\}\n', STORE_STUB, text,
                          count=1, flags=re.M | re.S)
        if n != 1:
            sys.exit('test-host: update.rs `mod store` not found; update scripts/test-host.py')
        rewritten += 1
    if src.name == 'backup.rs':
        text, n = re.subn(r'^fn recycle\(path: &Path\) -> Recycle \{.*?^\}\n', RECYCLE_STUB, text, count=1, flags=re.M | re.S)
        if n != 1:
            sys.exit('test-host: backup.rs `fn recycle` not found; update scripts/test-host.py')
        rewritten += 1
    write_if_changed(scratch / src.relative_to(root), text)
if rewritten == 0:
    sys.exit('test-host: no std::os::windows rewrite matched; update scripts/test-host.py')
for stale in (scratch / 'src').rglob('*.rs'):
    if not (root / stale.relative_to(scratch)).is_file():
        stale.unlink()

# 2. The stub for std::os::windows.
write_if_changed(scratch / 'host-os/Cargo.toml', '[package]\nname = "host-os"\nversion = "0.0.0"\nedition = "2021"\n')
write_if_changed(scratch / 'host-os/src/lib.rs', '''//! Stand-ins for the two std::os::windows items the app uses.
pub mod windows {
    pub mod ffi {
        pub trait OsStrExt {
            fn encode_wide(&self) -> std::vec::IntoIter<u16>;
        }
        impl OsStrExt for std::ffi::OsStr {
            fn encode_wide(&self) -> std::vec::IntoIter<u16> {
                self.to_string_lossy().encode_utf16().collect::<Vec<_>>().into_iter()
            }
        }
    }
    pub mod process {
        pub trait CommandExt {
            fn creation_flags(&mut self, flags: u32) -> &mut Self;
            fn raw_arg<S: AsRef<std::ffi::OsStr>>(&mut self, text: S) -> &mut Self;
        }
        impl CommandExt for std::process::Command {
            fn creation_flags(&mut self, _flags: u32) -> &mut Self {
                self
            }
            fn raw_arg<S: AsRef<std::ffi::OsStr>>(&mut self, text: S) -> &mut Self {
                self.arg(text)
            }
        }
    }
}
''')

# 3. Cargo.toml and Cargo.lock: the real dependencies, all of them, on this host.
real = sections((root / 'Cargo.toml').read_text())
win = real['target."cfg(windows)".dependencies']
deps = dep_lines(real['dependencies'], skip=('eframe',)) + dep_lines(win, skip=('eframe', 'windows', 'windows-future'))
deps = [l.replace('path = "netcore"', f'path = "{root / "netcore"}"') for l in deps]
# On Windows another crate turns on the windows-sys feature GetAdaptersAddresses needs.
deps = [l.replace('features = [', 'features = ["Win32_NetworkManagement_Ndis", ', 1) if l.startswith('windows-sys ') else l for l in deps]
# The features Windows unifies (OpenGL and Direct3D/wgpu); wgpu has a Metal backend on a Mac.
eframe = next(l for l in real['dependencies'] if l.startswith('eframe ')).replace('features = [', 'features = ["wgpu", ', 1)
manifest = ['[package]'] + dep_lines(real['package']) + ['', '[dependencies]'] + deps + [eframe, 'host-os = { path = "host-os" }',
    '', '[build-dependencies]'] + dep_lines(real['build-dependencies'])
write_if_changed(scratch / 'Cargo.toml', '\n'.join(manifest) + '\n')
shutil.copyfile(root / 'Cargo.lock', scratch / 'Cargo.lock')
for name in ('build.rs', 'packaging'):
    link = scratch / name
    if not link.is_symlink():
        link.symlink_to(root / name)

env = {**os.environ, 'PATH': str(cargo.parent) + os.pathsep + os.environ.get('PATH', ''),
       'CARGO_TARGET_DIR': str(scratch / 'build')}
if sys.platform == 'darwin':
    # The Win32 functions are declared but not defined here: let the linker leave them
    # open, then (below) define them in a library that aborts if a test ever calls one.
    env['RUSTFLAGS'] = '-C link-arg=-Wl,-undefined,dynamic_lookup ' + os.environ.get('RUSTFLAGS', '')
else:
    env['RUSTFLAGS'] = '-C link-arg=-Wl,--unresolved-symbols=ignore-all ' + os.environ.get('RUSTFLAGS', '')
manifest_path = str(scratch / 'Cargo.toml')


def cargo_run(*args, **kw):
    return subprocess.run([str(cargo), *args, '--offline', '--manifest-path', manifest_path], cwd=root, env=env, **kw)


if sys.platform == 'darwin':
    # dyld binds every symbol at load, so a binary with open Win32 symbols will not start.
    built = cargo_run('test', '--no-run', '--message-format=json', stdout=subprocess.PIPE, text=True)
    if built.returncode != 0:
        sys.exit(built.returncode)
    import json
    exes = [m['executable'] for m in map(json.loads, filter(lambda l: l.startswith('{'), built.stdout.splitlines()))
            if m.get('reason') == 'compiler-artifact' and m.get('executable') and m.get('profile', {}).get('test')]
    names = set()
    for exe in exes:
        out = subprocess.run(['nm', '-um', exe], capture_output=True, text=True).stdout
        names.update(l.split()[2].lstrip('_') for l in out.splitlines() if 'dynamically looked up' in l)
    stubs = scratch / 'win32-stubs'
    stubs.mkdir(exist_ok=True)
    # Time zone conversion is the one Win32 call a test relies on (Stamp::local): the
    # stand-ins keep the time as it is (UTC), which is enough for the date checks.
    real = {
        'GetLocalTime': 'void GetLocalTime(void *t) { memset(t, 0, 16); }',
        'SystemTimeToTzSpecificLocalTime': 'int SystemTimeToTzSpecificLocalTime(void *z, const void *from, void *to) '
                                           '{ memcpy(to, from, 16); return 1; }',
    }
    write_if_changed(stubs / 'stubs.c', '#include <stdlib.h>\n#include <string.h>\n' + ''.join(
        real.get(n, f'void {n}(void) {{ abort(); }}') + '\n' for n in sorted(names)))
    subprocess.run(['cc', '-dynamiclib', '-o', str(stubs / 'stubs.dylib'), str(stubs / 'stubs.c')], check=True)
    runner = stubs / 'run.sh'
    write_if_changed(runner, f'#!/bin/sh\nexec env DYLD_INSERT_LIBRARIES="{stubs / "stubs.dylib"}" "$@"\n')
    runner.chmod(0o755)
    host = re.search(r'^host: (.+)$', subprocess.run([str(cargo.parent / 'rustc'), '-vV'], capture_output=True, text=True).stdout, re.M).group(1)
    env['CARGO_TARGET_' + host.upper().replace('-', '_') + '_RUNNER'] = str(runner)
sys.exit(cargo_run('test').returncode)
