use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
  let _: () = log::info!("something");
  Ok(()) // ERROR
  // Ok(println!("something")) // OK
  // Ok(()) // OK
}
