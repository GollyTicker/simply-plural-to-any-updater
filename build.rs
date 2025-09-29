fn main() {
    // sqlx: trigger recompilation when a new migration is added or the license text has changed
    println!("cargo:rerun-if-changed=docker/migrations");
    println!("cargo:rerun-if-changed=docker/license*");
}
