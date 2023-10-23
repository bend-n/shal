// Process("echo a")[Io::Front] => Process(grep("a"))[Io::Middle] => Process(File)
/// # Safety
///
/// this will return the number of bytes passed to the buffer, and ret < buffer bounds
pub unsafe trait Process: std::fmt::Debug {
    fn run(&mut self, on: Io) -> anyhow::Result<usize>;
    fn done(&mut self, before: Option<bool>) -> anyhow::Result<bool> {
        Ok(before.unwrap_or(false))
    }
}

#[derive(Debug)]
pub struct Processor {
    pub(crate) processes: Vec<Box<dyn Process>>,
    buffer: Vec<u8>,
    buffer_size: u32,
}

pub enum Io<'a> {
    /// echo "a" => next
    First(&'a mut [u8]),
    /// => file
    Last(&'a [u8]),
    /// => this =>
    Middle(&'a [u8], &'a mut [u8]),
}

#[derive(Copy, Clone, Debug)]
enum Buf {
    A,
    B,
}

impl Processor {
    pub fn new(buffer: u32) -> Self {
        Processor {
            processes: Vec::with_capacity(2),
            buffer: vec![0; buffer as usize * 2],
            buffer_size: buffer,
        }
    }

    pub(crate) fn add(&mut self, p: impl Process + 'static) {
        self.processes.push(Box::new(p));
    }

    pub fn complete(&mut self) -> anyhow::Result<()> {
        while !self.done()? {
            self.step()?;
        }
        Ok(())
    }

    pub fn done(&mut self) -> anyhow::Result<bool> {
        let mut done = true;
        let mut last = None;
        for p in &mut self.processes {
            let result = p.done(last)?;
            done &= result;
            last = Some(result);
        }
        Ok(done)
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        macro_rules! a {
            () => {
                &mut self.buffer[..self.buffer_size as usize]
            };
        }
        macro_rules! b {
            () => {
                &mut self.buffer[self.buffer_size as usize..]
            };
        }
        // [a,b,c,d,e]
        // a (first(a_buf))
        // b (mid(a_buf, b_buf))
        // c (mid(b_buf, a_buf))
        // d (mid(a_buf, b_buf))
        // e (last(b_buf))
        let mut last = Buf::A;
        let mut read = 0;
        let size = self.processes.len();
        for (i, p) in self.processes.iter_mut().enumerate() {
            match i {
                0 => read = p.run(Io::First(a!()))?,
                n if n + 1 == size => {
                    p.run(Io::Last(
                        &match last {
                            Buf::A => a!(),
                            Buf::B => b!(),
                        }[..read],
                    ))?;
                }
                _ => match last {
                    Buf::A => {
                        let (a, b) = self.buffer.split_at_mut(self.buffer_size as usize);
                        read = p.run(Io::Middle(&a[..read], b))?;
                        last = Buf::B;
                    }
                    Buf::B => {
                        let (a, b) = self.buffer.split_at_mut(self.buffer_size as usize);
                        read = p.run(Io::Middle(&b[..read], a))?;
                        last = Buf::A;
                    }
                },
            }
        }
        Ok(())
    }
}
