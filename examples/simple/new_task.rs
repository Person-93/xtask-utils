use std::{fs, path::Path};

#[derive(clap::Parser)]
/// Add a new xtask
pub struct Cli {
  /// Name of the new task (should be in snake case)
  name: String,
}

pub fn main(Cli { name }: Cli) -> ! {
  // in this example, we put the new task file in this example's directory
  // in non-example code, it will have to go in your xtasks crate
  // use the commented out line instead of the following line to do so
  let xtasks_src_root =
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/simple"));
  // let xtasks_src_root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src").unwrap();

  let mut path = xtasks_src_root.join(&name);
  path.set_extension("rs");

  if path.exists() {
    eprintln!("A task with the name `{name}` already exists");
    std::process::exit(1);
  }

  fs::write(path, NEW_FILE).unwrap();

  let path = xtasks_src_root.join("main.rs");
  let mut main = fs::read_to_string(&path).unwrap();

  // get the index of just past the start of the tasks macro
  let search = "xtask_utils::tasks!(";
  let idx = main
    .find(search)
    .expect("could not find usage of xtask_utils::tasks!")
    + search.len();

  // use proper line ending
  let (idx, has_carriage_return) = if &main[idx..idx + 1] == "\r" {
    (idx + 2, true)
  } else {
    (idx + 1, false)
  };

  // respect existing indentation
  let indentation = {
    let start_idx = idx;
    let mut indentation = "";
    for (idx, char) in main[idx..].char_indices() {
      if !char.is_whitespace() {
        let idx = idx + start_idx;
        indentation = &main[start_idx..idx];
        break;
      }
    }
    indentation
  };

  let newline = if has_carriage_return { "\r\n" } else { "\n" };
  main.insert_str(idx, &format!("{indentation}{name}{newline}"));

  fs::write(&path, &main).unwrap();

  std::process::exit(0);
}

const NEW_FILE: &str = "use clap::Parser;

#[derive(Parser)]
pub struct Cli {}

pub fn main(Cli {}: Cli) -> ! {
    std::process::exit(0);
}";
