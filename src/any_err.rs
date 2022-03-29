use std::{
  error::Error,
  fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub struct AnyErr {
  message: String,
  source: Option<Box<dyn Error + 'static>>,
}

impl AnyErr {
  pub fn new(message: impl ToString) -> Self {
    Self {
      message: message.to_string(),
      source: None,
    }
  }

  pub fn with_source(
    message: impl ToString,
    source: impl Error + 'static,
  ) -> Self {
    Self {
      message: message.to_string(),
      source: Some(Box::new(source)),
    }
  }
}

impl Error for AnyErr {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    self.source.as_deref()
  }
}

impl Display for AnyErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    Display::fmt(&self.message, f)
  }
}
