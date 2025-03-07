use clap::Parser;

xtask_utils::tasks!(task);

fn main() -> ! {
  Task::parse().run();
}
