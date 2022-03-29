use self::common::*;
use std::error::Error;
use xtask_utils::any_err::AnyErr;

mod common;

mod error_ext {
  use super::*;
  use xtask_utils::result_ext::ErrorExt;

  #[test]
  fn context() {
    let err = AnyErr::new("first message");
    let err = err.context("second message");
    assert!(err.to_string().contains("second message"));
    assert!(err
      .source()
      .expect("missing error source")
      .downcast_ref::<AnyErr>()
      .expect("failed downcasting error source")
      .to_string()
      .contains("first message"));
  }

  #[test]
  fn exit_err_ok() {
    run_helper(true, ["res-exit-err", "ok"]).contains("all good\n");
  }

  #[test]
  fn exit_err_err() {
    run_helper(false, ["res-exit-err", "err"]);
  }

  #[test]
  fn exit_ok() {
    run_helper(true, ["res-exit", "ok"]);
  }

  #[test]
  fn exit_err() {
    run_helper(false, ["res-exit", "err"]);
  }
}

mod result_ext {
  use super::*;

  #[test]
  fn exit_ok() {
    run_helper(true, ["res-exit-err", "ok"]);
  }

  #[test]
  fn exit_err() {
    run_helper(false, ["res-exit-err", "err"]);
  }
}
