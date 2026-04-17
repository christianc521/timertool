fn main() {
    if std::env::var("CARGO_FEATURE_SIMULATOR").is_err() {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
}
