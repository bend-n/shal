#![allow(non_camel_case_types)]
use std::{
    ffi::{OsStr, OsString},
    io::{self, Read, Write},
    ops::{BitOr, Shr},
    process::{self, Child, ChildStderr, ChildStdin, ChildStdout, Stdio},
};
mod output;
mod processor;
mod util;
pub use output::null;
use processor::Io;
pub use processor::{Process, Processor};
pub use util::echo;

/// generates a impl of $a | $b => [`Processor`].
macro_rules! imp {
    (Processor,$b:ty) => {
        impl BitOr<$b> for Processor {
            type Output = Processor;
            fn bitor(mut self, rhs: $b) -> Self::Output {
                self.add(rhs);
                self
            }
        }
    };
    ($a:ty,$b:ty) => {
        impl BitOr<$b> for $a {
            type Output = Processor;
            fn bitor(self, rhs: $b) -> Self::Output {
                let mut p = Processor::new(512);
                p.add(self);
                p.add(rhs);
                p
            }
        }
    };
    (output $from:ty) => {
        impl Shr<&mut String> for $from {
            type Output = anyhow::Result<()>;
            fn shr(self, rhs: &mut String) -> Self::Output {
                let mut p = Processor::new(512);
                p.add(self);
                p >> rhs
            }
        }

        impl Shr<()> for $from {
            type Output = anyhow::Result<String>;
            fn shr(self, _: ()) -> Self::Output {
                let mut s = String::new();
                (self >> &mut s)?;
                Ok(s)
            }
        }
    };
}

// imp!(Command, grep); // cargo | grep
// imp!(echo, grep); // echo | grep
// imp!(Processor, grep); // a | b | grep
imp!(Command, Command); // command | command
imp!(echo, Command); // echo | command
imp!(Processor, Command); // a | b | command
imp!(output Command);
imp!(output echo);

impl Shr<&mut String> for Processor {
    type Output = anyhow::Result<()>;

    fn shr(mut self, rhs: &mut String) -> Self::Output {
        self.add(output::StringOut(rhs as *mut _));
        self.complete()?;
        Ok(())
    }
}

impl Shr<()> for Processor {
    type Output = anyhow::Result<String>;

    fn shr(self, _: ()) -> Self::Output {
        let mut s = String::new();
        (self >> &mut s)?;
        Ok(s)
    }
}

#[derive(Debug)]
pub struct Command {
    stdout: ChildStdout,
    stderr: ChildStderr,
    stdin: ChildStdin,
    proc: Child,
}

unsafe impl Process for Command {
    fn run(&mut self, on: processor::Io) -> anyhow::Result<usize> {
        match on {
            Io::First(b) => Ok(self.stdout.read(b)?),
            Io::Middle(i, o) => {
                self.stdin.write(i)?;
                Ok(self.stdout.read(o)?)
            }
            Io::Last(i) => {
                self.stdin.write(i)?;
                Ok(0)
            }
        }
    }

    fn done(&mut self, _: Option<bool>) -> anyhow::Result<bool> {
        Ok(self.proc.try_wait()?.is_some())
    }
}

#[derive(Debug)]
pub struct CommandBuilder {
    command: OsString,
    args: Vec<OsString>,
}

impl CommandBuilder {
    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut CommandBuilder {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>> + ExactSizeIterator) {
        self.args.reserve(self.args.len() + args.len());
        for arg in args {
            self.args.push(arg.as_ref().to_os_string());
        }
    }

    pub fn spawn(&mut self) -> io::Result<Command> {
        let mut proc = process::Command::new(&self.command)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;
        Ok(Command {
            stdout: proc.stdout.take().unwrap(),
            stderr: proc.stderr.take().unwrap(),
            stdin: proc.stdin.take().unwrap(),
            proc,
        })
    }
}

impl Command {
    pub fn new(command: impl AsRef<OsStr>) -> CommandBuilder {
        CommandBuilder {
            command: command.as_ref().to_os_string(),
            args: vec![],
        }
    }
}

#[test]
fn usage() {
    let o = ((Command::new("cargo").spawn().unwrap()
        | Command::new("grep").arg("test").spawn().unwrap())
        >> ())
        .unwrap();
    assert_eq!(o, "");
}
