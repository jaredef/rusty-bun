// Ω.5.P46.E1.napi-v1: export napi_* symbols dynamically so dlopen'd
// .node native modules can resolve them via dlsym from this process.
// Linux uses --export-dynamic; macOS uses -export_dynamic. The
// keepalive-array in pilots/rusty-js-runtime/derived/src/napi.rs (plus
// the reference from main.rs) keeps the symbols from being dead-
// stripped before the linker even gets to export them.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    let is_macos = target.contains("apple-darwin");
    if is_macos {
        println!("cargo:rustc-link-arg-bin=rusty-bun-host-v2=-Wl,-export_dynamic");
    } else {
        // Linux + most other unix-likes.
        println!("cargo:rustc-link-arg-bin=rusty-bun-host-v2=-Wl,--export-dynamic");
    }
}
