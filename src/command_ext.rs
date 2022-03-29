#[cfg(unix)]
use std::os::unix::process::CommandExt as UnixCommand;
use std::{
  error::Error as StdError,
  fmt::{self, Display, Formatter},
  io,
  process::{self, Command, ExitStatus},
};

pub mod prelude {
  pub use super::{CommandExt, ExitStatusExt};
}

pub trait CommandExt {
  #[must_use]
  fn exec(&mut self) -> Error;

  fn wait(&mut self) -> Result<()>;

  fn from_str(s: impl AsRef<str>) -> Self;
}

impl CommandExt for Command {
  /// Run the command and exit with its exit code.
  ///
  /// On unix it uses `exec`
  #[must_use]
  fn exec(&mut self) -> Error {
    #[cfg(unix)]
    return Error {
      kind: ErrorKind::Spawn(UnixCommand::exec(self)),
      name: self.get_program().to_string_lossy().into_owned(),
    };

    #[cfg(not(unix))]
    match self.wait() {
      Ok(()) => process::exit(0),
      Err(err) => err,
    }
  }

  /// Spawn the command and wait for it to complete. Exit with its exit code
  /// if non-zero.
  fn wait(&mut self) -> Result<()> {
    match self
      .spawn()
      .map_err(|source| Error {
        name: self.get_program().to_string_lossy().into_owned(),
        kind: ErrorKind::Spawn(source),
      })?
      .wait()
    {
      Ok(status) => {
        status.exit_on_err();
        Ok(())
      }
      Err(source) => Err(Error {
        name: self.get_program().to_string_lossy().into_owned(),
        kind: ErrorKind::Wait(source),
      }),
    }
  }

  fn from_str(s: impl AsRef<str>) -> Self {
    let mut args = s.as_ref().split_whitespace();
    let arg0 = args.next().expect("arg0 not supplied");
    let mut cmd = Command::new(arg0);
    cmd.args(args);
    cmd
  }
}

pub trait ExitStatusExt {
  fn exit_on_err(&self);

  fn exit_on_err_with<F: FnOnce()>(&self, f: F);
}

impl ExitStatusExt for ExitStatus {
  fn exit_on_err(&self) {
    self.exit_on_err_with(|| {})
  }

  fn exit_on_err_with<F: FnOnce()>(&self, f: F) {
    if !self.success() {
      f();
      print_status_and_exit(self);
    }
  }
}

fn print_status_and_exit(status: &ExitStatus) -> ! {
  eprintln!("\n--------------------------------------\n{}", status);
  process::exit(status.code().unwrap_or(1));
}

#[derive(Debug)]
pub struct Error {
  name: String,
  kind: ErrorKind,
}

impl Error {
  pub fn kind(&self) -> &ErrorKind {
    &self.kind
  }
}

impl StdError for Error {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match &self.kind {
      ErrorKind::File(err) | ErrorKind::Wait(err) | ErrorKind::Spawn(err) => {
        Some(err)
      }
      ErrorKind::Exit(_) => None,
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let Self { name, kind } = self;
    match kind {
      ErrorKind::File(_) => write!(f, "failed opening file at {name}"),
      ErrorKind::Spawn(_) => write!(f, "failed spawning {name}"),
      ErrorKind::Wait(_) => write!(f, "failed waiting for {name}"),
      ErrorKind::Exit(status) => {
        write!(f, "child process \"{name}\" failed: {status}")
      }
    }
  }
}

#[derive(Debug)]
pub enum ErrorKind {
  File(io::Error),
  Spawn(io::Error),
  Wait(io::Error),
  Exit(ExitStatus),
}

pub type Result<T> = std::result::Result<T, Error>;
