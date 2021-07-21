extern crate dotenv;
fn main() {
    println!(
        "cargo:rustc-env=DATABASE_URL={}",
        dotenv::var("WH_DATABASE_URL").unwrap(),
    );
}
