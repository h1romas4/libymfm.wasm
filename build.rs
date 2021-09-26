fn main() {
    // export LD_LIBRARY_PATH=$(pwd)/dist
    println!("cargo:rustc-link-search=native=./dist");
}
