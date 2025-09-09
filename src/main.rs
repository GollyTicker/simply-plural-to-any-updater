use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
  Ok(log::info!("something")) // ERROR
  // Ok(println!("something")) // OK
  // Ok(()) // OK
}
