use crate::{
  command_ext::{Error, ErrorKind, Result},
  prelude::*,
};
use std::{
  borrow::Cow,
  convert, fs,
  fs::File,
  path::{Path, PathBuf},
  process::{Child, ChildStdout, Command, ExitStatus, Stdio},
};

#[macro_export]
macro_rules! script {
  ($(($($command:tt)+))*) => {
    || -> Result<(), $crate::command_ext::Error> {
      $( $crate::pipe!($($command)*).wait()?; )*
      Ok(())
    }()
  };
}

#[macro_export]
macro_rules! run_cmd {
  ($($command:tt)*) => {
    $crate::pipe!($($command)*).exec()
  };
}

#[macro_export]
macro_rules! pipe {
  ($($command:tt)|*) => {
    pipe!($($command)|* > $crate::pipe::PipeIo::Inherit)
  };

  ($($command:tt)|* > $($output:tt)*) => {
    $crate::pipe::Pipe::new(vec![$(&$command as &dyn $crate::pipe::PipeSection),*], {
      $($output)*
    })
  };
}

/// Run a bunch of commands pipes the input of each to the output of the next.
/// The output of the final one is piped to the given output
#[must_use]
pub struct Pipe<'p> {
  commands: Box<dyn DoubleEndedIterator<Item = &'p dyn PipeSection> + 'p>,
  output: PipeIo,
}

impl<'p> Pipe<'p> {
  pub fn new<I>(commands: I, output: impl Into<PipeIo>) -> Self
  where
    I: IntoIterator<Item = &'p dyn PipeSection>,
    I::IntoIter: DoubleEndedIterator + 'p,
  {
    Self {
      commands: Box::new(commands.into_iter()),
      output: output.into(),
    }
  }

  pub fn spawn(self) -> Result<JoinHandle> {
    self.impl_(convert::identity, convert::identity)
  }

  pub fn exec(self) -> ! {
    self
      .impl_(|err| err.exit(), |handle| handle.join().exit())
      .exit()
  }

  pub fn wait(self) -> Result<()> {
    self
      .spawn()
      .map(|handle| handle.join().exit_on_err().exit_on_err())
  }

  fn impl_<T>(
    mut self,
    mut on_err: impl FnMut(Error) -> Error,
    join: impl FnOnce(JoinHandle) -> T,
  ) -> Result<T> {
    let first = match self.commands.next() {
      Some(first) => first,
      None => panic!("pipe was called with no commands"),
    };

    let last = match self.commands.next_back() {
      Some(last) => last,
      None => {
        let handle = first
          .end_pipe(PipeIo::Inherit, self.output)
          .map_err(&mut on_err)?;
        return Ok(join(handle));
      }
    };

    let mut handles = Vec::new();
    let (mut previous, handle) =
      first.do_pipe(PipeIo::Inherit).map_err(&mut on_err)?;
    handles.push(handle);

    for command in self.commands {
      let (io, handle) = match command.do_pipe(previous) {
        Ok(handle) => handle,
        Err(err) => {
          handles.into_iter().for_each(JoinHandle::cancel);
          return Err(on_err(err));
        }
      };
      previous = io;
      handles.push(handle);
    }

    match last.end_pipe(previous, self.output) {
      Ok(handle) => handles.push(handle),
      Err(err) => {
        handles.into_iter().for_each(JoinHandle::cancel);
        err.exit();
      }
    }

    Ok(join(JoinHandle::Multiple(handles)))
  }
}

pub trait PipeSection {
  fn do_pipe(&self, input: PipeIo) -> Result<(PipeIo, JoinHandle)>;

  fn end_pipe(&self, input: PipeIo, output: PipeIo) -> Result<JoinHandle>;
}

impl PipeSection for &str {
  fn do_pipe(&self, input: PipeIo) -> Result<(PipeIo, JoinHandle)> {
    let mut command = Command::from_str(*self);
    let mut child = command
      .stdin(TryInto::<Stdio>::try_into(input)?)
      .stdout(Stdio::piped())
      .spawn()
      .map_err(|err| Error {
        name: command.get_program().to_string_lossy().into_owned(),
        kind: ErrorKind::Spawn(err),
      })?;
    let io = child.stdout.take().unwrap().into();
    Ok((io, JoinHandle::Cmd(child)))
  }

  fn end_pipe(&self, input: PipeIo, output: PipeIo) -> Result<JoinHandle> {
    let mut command = Command::from_str(self);
    let child = command
      .stdin(TryInto::<Stdio>::try_into(input)?)
      .stdout(TryInto::<Stdio>::try_into(output)?)
      .spawn()
      .map_err(|err| Error {
        name: command.get_program().to_string_lossy().into_owned(),
        kind: ErrorKind::Spawn(err),
      })?;
    Ok(JoinHandle::Cmd(child))
  }
}

impl PipeSection for String {
  fn do_pipe(&self, input: PipeIo) -> Result<(PipeIo, JoinHandle)> {
    self.as_str().do_pipe(input)
  }

  fn end_pipe(&self, input: PipeIo, output: PipeIo) -> Result<JoinHandle> {
    self.as_str().end_pipe(input, output)
  }
}

impl PipeSection for Cow<'_, str> {
  fn do_pipe(&self, input: PipeIo) -> Result<(PipeIo, JoinHandle)> {
    match self {
      Cow::Borrowed(s) => s.do_pipe(input),
      Cow::Owned(s) => s.do_pipe(input),
    }
  }

  fn end_pipe(&self, input: PipeIo, output: PipeIo) -> Result<JoinHandle> {
    match self {
      Cow::Borrowed(s) => s.end_pipe(input, output),
      Cow::Owned(s) => s.end_pipe(input, output),
    }
  }
}

pub enum PipeIo {
  Inherit,
  File(Cow<'static, Path>),
  Child(ChildStdout),
}

impl TryInto<Stdio> for PipeIo {
  type Error = Error;

  fn try_into(self) -> Result<Stdio> {
    match self {
      PipeIo::Inherit => Ok(Stdio::inherit()),
      PipeIo::File(path) => {
        let parent = path.parent().unwrap();
        fs::create_dir_all(parent).map_err(|err| Error {
          name: path.to_string_lossy().into_owned(),
          kind: ErrorKind::File(err),
        })?;

        match File::options()
          .write(true)
          .create(true)
          .truncate(true)
          .open(&path)
        {
          Ok(file) => Ok(Stdio::from(file)),
          Err(err) => Err(Error {
            name: path.to_string_lossy().into_owned(),
            kind: ErrorKind::File(err),
          }),
        }
      }
      PipeIo::Child(child) => Ok(Stdio::from(child)),
    }
  }
}

impl From<&'static Path> for PipeIo {
  fn from(path: &'static Path) -> Self {
    PipeIo::File(Cow::Borrowed(path))
  }
}

impl From<&'static str> for PipeIo {
  fn from(path: &'static str) -> Self {
    PipeIo::File(Cow::Borrowed(Path::new(path)))
  }
}

impl From<PathBuf> for PipeIo {
  fn from(path: PathBuf) -> Self {
    PipeIo::File(Cow::Owned(path))
  }
}

impl From<String> for PipeIo {
  fn from(path: String) -> Self {
    PipeIo::File(Cow::Owned(path.into()))
  }
}

impl From<ChildStdout> for PipeIo {
  fn from(child: ChildStdout) -> Self {
    Self::Child(child)
  }
}

#[must_use]
pub enum JoinHandle {
  Cmd(Child),
  Multiple(Vec<JoinHandle>),
}

impl JoinHandle {
  pub fn join(self) -> Result<Joined> {
    match self {
      JoinHandle::Cmd(mut cmd) => {
        Ok(Joined::Cmd(cmd.wait().map_err(|err| Error {
          name: "child process".to_string(),
          kind: ErrorKind::Wait(err),
        })?))
      }
      JoinHandle::Multiple(handles) => Ok(Joined::Multiple(
        handles
          .into_iter()
          .map(Self::join)
          .collect::<Vec<_>>()
          .into_iter()
          .collect::<Result<_>>()?,
      )),
    }
  }

  pub fn cancel(self) {
    match self {
      JoinHandle::Cmd(mut cmd) => {
        cmd.kill().ok();
      }
      JoinHandle::Multiple(handles) => {
        for handle in handles {
          handle.cancel();
        }
      }
    }
  }
}

#[must_use]
pub enum Joined {
  Cmd(ExitStatus),
  Multiple(Vec<Joined>),
}

impl Joined {
  pub fn success(&self) -> bool {
    match self {
      Joined::Cmd(status) => status.success(),
      Joined::Multiple(joined) => joined.iter().all(Joined::success),
    }
  }

  pub fn exit_on_err(self) {
    match self {
      Joined::Cmd(status) => status.exit_on_err(),
      Joined::Multiple(joined) => {
        for join in joined {
          join.exit_on_err();
        }
      }
    }
  }
}
