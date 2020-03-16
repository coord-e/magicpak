use std::io::Result;
use std::process::{Command, ExitStatus, Output};

use log::debug;

pub trait CommandExt {
    fn output_with_log(&mut self) -> Result<Output>;

    fn status_with_log(&mut self) -> Result<ExitStatus> {
        Ok(self.output_with_log()?.status)
    }
}

impl CommandExt for Command {
    fn output_with_log(&mut self) -> Result<Output> {
        let command_line = format!("{:?}", self);
        let output = self.output()?;

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

        Ok(output)
    }
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
