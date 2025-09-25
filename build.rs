fn main() {
    // sqlx: trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=docker/migrations");
}
