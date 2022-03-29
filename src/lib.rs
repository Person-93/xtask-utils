#![forbid(unsafe_code)]

pub mod prelude {
  #[cfg(feature = "result_ext")]
  pub use super::result_ext::prelude::*;
}

#[cfg(feature = "any_err")]
pub mod any_err;
#[cfg(feature = "result_ext")]
pub mod result_ext;

#[macro_export]
macro_rules! tasks {
  ($($name:ident)*) => {
    $(mod $name;)*

    #[derive(Subcommand)]
    #[allow(non_camel_case_types)]
    enum Task {
      $($name($name::Cli)),*
    }

    impl Task {
      fn run(self) -> ! {
        match self {
          $(Task::$name(cli) => self::$name::main(cli)),*
        }
      }
    }
  };
}
