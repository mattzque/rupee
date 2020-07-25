extern crate pkg_config;

fn main() {
    pkg_config::Config::new().probe("vips").unwrap();
    // println!("rustc-link-lib=magic");
}
