

use crate::{processor::Io, Process};

trait Output: std::fmt::Debug {
    fn take(&mut self, bytes: &[u8]) -> anyhow::Result<()>;
}

unsafe impl<T: Output> Process for T {
    fn run(&mut self, on: Io) -> anyhow::Result<usize> {
        match on {
            Io::Last(bytes) => self.take(bytes)?,
            Io::First(_) | Io::Middle(_, _) => unreachable!("outputs must be at the end"),
        }
        Ok(0)
    }
}

#[derive(Debug)]
pub(crate) struct StringOut(pub *mut String);

impl Output for StringOut {
    fn take(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        unsafe { &mut *self.0 }.push_str(std::str::from_utf8(bytes)?);
        Ok(())
    }
}

/// `/dev/null`
pub struct null {}

impl std::fmt::Debug for null {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/dev/null")
    }
}

impl Output for null {
    fn take(&mut self, _: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
}
