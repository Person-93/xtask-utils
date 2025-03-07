#![forbid(unsafe_code)]

pub mod prelude {
  #[cfg(feature = "command_ext")]
  pub use super::command_ext::prelude::*;
  #[cfg(feature = "result_ext")]
  pub use super::result_ext::prelude::*;
}

#[cfg(feature = "any_err")]
pub mod any_err;
#[cfg(feature = "command_ext")]
pub mod command_ext;
#[cfg(feature = "script")]
pub mod pipe;
#[cfg(feature = "result_ext")]
pub mod result_ext;

#[macro_export]
macro_rules! tasks {
  ($($name:ident)*) => {
    $(mod $name;)*

    #[derive(::clap::Parser)]
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
