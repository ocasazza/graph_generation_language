use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("ggl.pest");

    // Copy the pest file to the output directory
    if Path::new("ggl.pest").exists() {
        fs::copy("ggl.pest", &dest_path).unwrap();
    } else if Path::new("src/ggl.pest").exists() {
        fs::copy("src/ggl.pest", &dest_path).unwrap();
    }

    println!("cargo:rerun-if-changed=ggl.pest");
    println!("cargo:rerun-if-changed=src/ggl.pest");
}
