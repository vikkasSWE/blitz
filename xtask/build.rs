fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let current_dir = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join(&profile)
        .to_string_lossy()
        .replace('\\', "/");
    let assets_dir = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets")
        .to_string_lossy()
        .replace('\\', "/");

    println!("cargo:rustc-env=PROFILE={profile}");
    println!("cargo:rustc-env=OUT_DIR={current_dir}");
    println!("cargo:rustc-env=ASSETS_DIR={assets_dir}");
}
