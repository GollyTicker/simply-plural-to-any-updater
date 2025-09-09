use std::convert::Infallible;

fn main() -> Result<(), Infallible> {
  let n = 0;
  Ok(log::info!("{n}")) // ERROR
  // Ok(println!("{n}")) // OK
  // Ok(()) // OK
}
