#[derive(clap::Parser)]
/// Print a friendly greeting
pub struct Cli {
  /// Name of person to be greeted
  #[clap(default_value = "world")]
  name: String,
}

pub fn main(Cli { name }: Cli) -> ! {
  println!("Hello, {name}!");
  std::process::exit(0);
}
