use std::{
  env,
  process::{self, Command},
};
use xtask_utils::{any_err::AnyErr, prelude::*};

fn main() -> ! {
  let mut args = env::args();
  match args.nth(1).expect("test-helper run with no args").as_str() {
    "err-exit" => AnyErr::new("example error").exit(),
    "res-exit" => {
      let res: Result<(), AnyErr> = match args
        .next()
        .expect("res-exit needs an additional arg 'ok' or 'err'")
        .as_str()
      {
        "ok" => Ok(()),
        "err" => Err(AnyErr::new("error1")).context("error 2"),
        arg => {
          panic!("test-helper res-exit called with unrecognized arg '{arg}'")
        }
      };
      res.exit();
    }
    "res-exit-err" => {
      let res: Result<(), AnyErr> = match args
        .next()
        .expect("res-exit-err needs an additional arg 'ok' or 'err'")
        .as_str()
      {
        "ok" => Ok(()),
        "err" => Err(AnyErr::new("error1")).context("error 2"),
        arg => panic!(
          "test-helper res-exit-err called with unrecognized arg '{arg}'"
        ),
      };
      res.exit_on_err();
      println!("all good");
      process::exit(0);
    }
    "exit-code" => {
      let code = args.next().expect("test-helper exit-code needs an additional arg with the desired exit code");
      let code = code
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse '{code}' as an i32"));
      process::exit(code);
    }
    "exec" => {
      let code = args.next().expect(
        "test-helper exec needs an additional arg with the desired exit code",
      );
      let code: i32 = code
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse '{code}' as an i32"));
      let err = Command::new(env::args_os().next().unwrap())
        .arg("exit-code")
        .arg(code.to_string())
        .exec();
      panic!("exec failed: {err}");
    }
    "wait" => {
      let code = args.next().expect(
        "test-helper exec needs an additional arg with the desired exit code",
      );
      let code: i32 = code
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse '{code}' as an i32"));
      Command::new(env::args_os().next().unwrap())
        .arg("exit-code")
        .arg(code.to_string())
        .wait()
        .unwrap();
      process::exit(0);
    }
    arg => panic!("test helper run with unrecognized arg '{arg}'"),
  }
}
