fn main() -> std::io::Result<()> {
    println!("cargo:rustc-cfg=slint_debug_property"); // DF HACK
    Ok(())
}
