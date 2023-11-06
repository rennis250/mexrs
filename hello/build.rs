fn main() {
    let path = "C:\\Program Files\\MATLAB\\R2020a\\extern\\lib\\win64\\microsoft";
    println!("cargo:rustc-link-search=native={}", path);
}