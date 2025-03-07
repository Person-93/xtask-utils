//! run this example to see the generated help

use clap::Parser;


// call this macro with the names of all of your tasks
// this will:
//  - add `use my_task;` statements for each task
//  - make a `Task` enum that dervies `Parser` from the `clap` crate
xtask_utils::tasks!(
  // a task that prints a friendly greeing
  greet
  // a task that creates a new task by adding the new mod and adding
  // the task name to this list
  new_task
);

fn main() -> ! {
  // just parse the command line arguments and call the generated `run` method
  Task::parse().run();
}
