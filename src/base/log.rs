use std::io::Result;
use std::process::{Child, Command, ExitStatus, Output};

use log::debug;

pub trait CommandLogExt {
    fn spawn_with_log(&mut self) -> Result<Child>;

    fn output_with_log(&mut self) -> Result<Output>;

    fn status_with_log(&mut self) -> Result<ExitStatus> {
        Ok(self.output_with_log()?.status)
    }
}

pub trait ChildLogExt {
    fn wait_output_with_log(self) -> Result<Output>;
}

impl CommandLogExt for Command {
    fn spawn_with_log(&mut self) -> Result<Child> {
        debug!("spawn: {:?}", self);
        self.spawn()
    }

    fn output_with_log(&mut self) -> Result<Output> {
        let output = self.output()?;
        let command_line = format!("{:?}", self);
        log_output(&command_line, &output);
        Ok(output)
    }
}

impl ChildLogExt for Child {
    fn wait_output_with_log(self) -> Result<Output> {
        let output = self.wait_with_output()?;
        log_output("<spawned>", &output);
        Ok(output)
    }
}

pub fn log_output(command_line: &str, output: &Output) {
    let mut message = format!("command: {}\n  => {}", command_line, output.status);

    if !output.stdout.is_empty() {
        let stdout = format_lines(String::from_utf8_lossy(&output.stdout));
        message.push_str("\n  => stdout: ");
        message.push_str(&stdout);
    }
    if !output.stderr.is_empty() {
        let stderr = format_lines(String::from_utf8_lossy(&output.stderr));
        message.push_str("\n  => stderr: ");
        message.push_str(&stderr);
    }

    debug!("{}", message);
}

fn format_lines<S>(s: S) -> String
where
    S: AsRef<str>,
{
    let string = s.as_ref().trim();
    if string.lines().nth(1).is_some() {
        string
            .lines()
            .fold(String::new(), |acc, line| format!("{}\n  |  {}", acc, line))
    } else {
        string.to_owned()
    }
}
