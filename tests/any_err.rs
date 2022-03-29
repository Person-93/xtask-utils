use xtask_utils::any_err::AnyErr;

#[test]
fn retains_message() {
  let err = AnyErr::new("hello world");
  assert!(err.to_string().contains("hello world"));
  let err = AnyErr::with_source("hi again", err);
  assert!(err.to_string().contains("hi again"));
}
