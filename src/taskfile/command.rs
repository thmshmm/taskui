use anyhow::{Ok, Result};
use colored::Colorize;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
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

    let thread_print_out = thread::spawn(move || print_output(stdout));
    let thread_print_err = thread::spawn(move || print_output(stderr));

    let _ = thread_print_out.join();
    let _ = thread_print_err.join();

    Ok(())
}

fn print_output<T: Read>(stream: T) {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let l = line.unwrap();

        if l.ends_with("is up to date") {
            println!("{}", l.purple());
        } else if l.starts_with("task: Failed to run task") {
            println!("{}", l.red());
        } else if l.starts_with("task: ") {
            println!("{}", l.green());
        } else {
            println!("{}", l);
        }
    }
}
