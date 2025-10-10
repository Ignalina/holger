use holger_rust_repository::RustRepo;
fn main() {
    //let repo = RustRepo::new("kalle".to_string());
    println!("path: {:?}", RustRepo::sparse_path("a"));
    println!("path: {:?}", RustRepo::sparse_path("ab"));
    println!("path: {:?}", RustRepo::sparse_path("abc"));
    println!("path: {:?}", RustRepo::sparse_path("tokio"));
}
