use std::{
  env,
  io::{self, Write},
  process::Command,
};

pub fn run_helper(
  exit_success: bool,
  args: impl IntoIterator<Item = &'static str>,
) -> String {
  let mut cargo =
    env::var_os("CARGO").map_or_else(|| Command::new("cargo"), Command::new);
  let output = cargo
    .args(["run", "--manifest-path=test-helper/Cargo.toml", "--"])
    .args(args)
    .spawn()
    .expect("failed spawning test-helper")
    .wait_with_output()
    .expect("failed waiting for test-helper to exit");

  if exit_success != output.status.success() {
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    if exit_success {
      panic!(
        "expected test helper to exit zero: actual_exit: {}",
        output.status
      );
    } else {
      panic!("expected test helper to exit non-zero")
    }
  }

  String::from_utf8(output.stdout).unwrap()
}
