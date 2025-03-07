use clap::Parser;
use xtask_utils::{prelude::*, run_cmd, script};

#[derive(Parser)]
pub struct Cli {}

pub fn main(Cli {}: Cli) -> ! {
  // the `script` macro runs a series of commands
  script! {
    ("echo hello world")
    ("echo hello again")
    // you can pipe commands to each other as in shell scripts
    ("cat README.md" | "grep collection")
  }
  .exit_on_err();

  // or invoke a command and then exit immediately with its exit code
  run_cmd!("echo goodbye");
}
