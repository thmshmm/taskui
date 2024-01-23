use anyhow::{Ok, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::process::{ChildStderr, ChildStdout, Command, Stdio};
use std::thread;

pub fn run_task(name: String) -> Result<()> {
    let proc = Command::new("task")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = proc.stdout.unwrap();
    let stderr = proc.stderr.unwrap();

    let thread_print_out = thread::spawn(move || print_stdout(stdout));
    let thread_print_err = thread::spawn(move || print_stderr(stderr));

    let _ = thread_print_out.join();
    let _ = thread_print_err.join();

    Ok(())
}

fn print_stdout(stream: ChildStdout) {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let l = line.unwrap();

        if l.ends_with("is up to date") {
            println!("{}", l.purple());
        } else if l.starts_with("task: ") {
            println!("{}", l.green());
        } else {
            println!("{}", l);
        }
    }
}

fn print_stderr(stream: ChildStderr) {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let l = line.unwrap();
        println!("{}", l);
    }
}
