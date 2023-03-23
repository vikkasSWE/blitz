fn main() {
    let assets_dir = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets")
        .to_string_lossy()
        .replace('\\', "/");

    println!("cargo:rustc-env=ASSETS_DIR={assets_dir}");
}
