use std::{
  env, fs,
  time::{SystemTime, UNIX_EPOCH},
};
use xtask_utils::cmd;

#[test]
fn happy_path() {
  let mut temp = env::temp_dir().join(env!("CARGO_PKG_NAME")).join(
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros()
      .to_string(),
  );
  temp.set_extension("txt");

  cmd!("echo hello world" | "cat" | "cat" > temp.clone())
    .wait()
    .unwrap();

  assert_eq!(
    fs::read_to_string(&temp)
      .unwrap_or_else(|_| panic!(
        "failed to read temp file: {}",
        temp.display()
      ))
      .trim(),
    "hello world"
  )
}

#[test]
fn single_command_fail() {
  let result = cmd!(FAIL_CMD).spawn().unwrap().join().unwrap();
  assert!(!result.success());
}

#[test]
fn multi_command_first_fail() {
  let result = cmd!(FAIL_CMD | "cat" | "cat")
    .spawn()
    .unwrap()
    .join()
    .unwrap();
  assert!(!result.success());
}

#[test]
fn multi_command_last_fail() {
  let result = cmd!("echo hello" | "cat" | FAIL_CMD)
    .spawn()
    .unwrap()
    .join()
    .unwrap();
  assert!(!result.success());
}

#[test]
fn multi_command_mid_fail() {
  let result = cmd!("echo hello" | FAIL_CMD | "echo")
    .spawn()
    .unwrap()
    .join()
    .unwrap();
  assert!(!result.success());
}

const FAIL_CMD: &str = "cargo run --manifest-path test-helper -- exit-code 1";
