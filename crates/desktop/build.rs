// Tell the linker where to find SDL2 on macOS / Linux. The Rust `sdl2`
// crate links against system SDL2 but doesn't add Homebrew's library
// path to the linker search. Without this the build fails with
// `ld: library 'SDL2' not found` even when `brew install sdl2` has
// already been run.

fn main() {
    if cfg!(target_os = "macos") {
        // Apple silicon Homebrew lives under /opt/homebrew; Intel
        // Homebrew uses /usr/local. Add both — the linker silently
        // ignores ones that don't exist.
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-search=/usr/local/lib");
    }
}
