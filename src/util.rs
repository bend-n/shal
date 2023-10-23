#![allow(non_camel_case_types)]

use std::io::Write;

use crate::{processor::Io, Process};

impl std::fmt::Debug for echo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "echo {}",
            String::from_utf8_lossy(self.what).escape_debug()
        )
    }
}
pub struct echo {
    what: &'static [u8],
    written: usize,
}

unsafe impl Process for echo {
    fn run(&mut self, on: Io) -> anyhow::Result<usize> {
        match on {
            Io::First(mut w) => {
                let writ = w.write(&self.what[..self.written])?;
                self.written += writ;
                Ok(writ)
            }
            Io::Middle(..) | Io::Last(_) => unreachable!(),
        }
    }

    fn done(&mut self, _: Option<bool>) -> anyhow::Result<bool> {
        Ok(self.written == self.what.len())
    }
}
