use std::io::{self, stdout, Write};

use sonr::reactor::{Reaction, Reactor, EventedReactor};
use sonr::{Ready, Evented, Token, Poll, PollOpt};
use bytes::{Bytes, BytesMut};
use mio::unix::EventedFd;

struct InnerStdout<'a> {
    stdout: io::Stdout,
    evented_fd: EventedFd<'a>,
}

impl<'a> InnerStdout<'a> {
    pub fn new() -> Self {
        Self {  
            stdout: stdout(),
            evented_fd: EventedFd(&1),
        }
    }
}

impl<'a> Evented for InnerStdout<'a> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.register(&self.evented_fd, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.reregister(&self.evented_fd, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(&self.evented_fd)
    }
}

impl<'a> Write for InnerStdout<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

pub struct Stdout<'a> {
    inner: EventedReactor<InnerStdout<'a>>,
    buffer: Option<Bytes>,
}

impl<'a> Stdout<'a> {
    pub fn new() -> Self {
        Self { 
            inner: EventedReactor::new(InnerStdout::new(), Ready::writable()).unwrap(),
            buffer: None,
        }
    }
}

impl<'a> Reactor for Stdout<'a> {
    type Input = Bytes;
    type Output = usize;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Event(ev) => {
                if self.inner.token() != ev.token() {
                    return ev.into();
                }
                Continue
            }
            Value(val) => {
                match self.inner.write(&val) {
                    Ok(0) => Continue,
                    Ok(n) => {
                        if n == val.len() {
                            let _ = self.inner.flush();
                        } else {
                            match &mut self.buffer {
                                None => self.buffer = Some(Bytes::from(&val[n..])),
                                Some(existing_buffer) => {
                                    let len = existing_buffer.len() + val.len() - n;
                                    let mut b = BytesMut::with_capacity(len);
                                    b.extend(&existing_buffer[..]);
                                    b.extend(&val[n..]);
                                    self.buffer = Some(b.freeze());
                                }
                            }
                        }
                        Value(n)
                    }
                    Err(_) => Continue
                }
            },
            Continue => Continue,
        }
    }
}


