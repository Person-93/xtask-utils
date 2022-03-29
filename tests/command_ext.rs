use self::common::*;

mod common;

#[test]
fn wait_ok() {
  run_helper(true, ["wait", "0"]);
}

#[test]
fn wait_err() {
  run_helper(false, ["wait", "1"]);
}

#[test]
fn exec_ok() {
  run_helper(true, ["exec", "0"]);
}

#[test]
fn exec_err() {
  run_helper(false, ["exec", "1"]);
}
