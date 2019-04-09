
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=templates/list.html");
    println!("cargo:rerun-if-changed=templates/index.html");
}

