use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use flate2::Compression;
use flate2::write::GzEncoder;

fn main() {
    println!("cargo:rerun-if-changed=src/bin/pix/index.html");

    let html = fs::read("src/bin/pix/index.html").unwrap();

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut enc = GzEncoder::new(
        fs::File::create(out.join("index.html.gz")).unwrap(),
        Compression::best(),
    );
    enc.write_all(&html).unwrap();
    enc.finish().unwrap();
}
