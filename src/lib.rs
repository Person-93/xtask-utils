#![forbid(unsafe_code)]

pub mod prelude {}

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
