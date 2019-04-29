use std::io::{self, stdin, Read};

use sonr::reactor::{Reaction, Reactor, EventedReactor};
use sonr::errors::Result;
use sonr::{Ready, Evented, Token, Poll, PollOpt};
use bytes::Bytes;
use mio::unix::EventedFd;

struct InnerStdin<'a> {
    stdin: io::Stdin,
    evented_fd: EventedFd<'a>
}

impl<'a> InnerStdin<'a> {
    pub fn new() -> Self {
        Self {  
            stdin: stdin(),
            evented_fd: EventedFd(&0),
        }
    }
}

impl<'a> Evented for InnerStdin<'a> {
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

impl<'a> Read for InnerStdin<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

pub struct Stdin<'a> {
    inner: EventedReactor<InnerStdin<'a>>
}

impl<'a> Stdin<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(InnerStdin::new(), Ready::readable())?,
        })
    }
}

impl<'a> Reactor for Stdin<'a> {
    type Input = ();
    type Output = Bytes;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        use Reaction::*;
        match reaction {
            Event(ev) => {
                if self.inner.token() != ev.token() {
                    return ev.into();
                }

                if ev.readiness().is_readable() {
                    let mut buf = [0u8; 1024];
                    match self.inner.read(&mut buf) {
                        Ok(0) => Continue,
                        Ok(n) => Value(Bytes::from(&buf[..n])),
                        Err(_) => Continue
                    }
                } else {
                    Continue
                }
            }
            Value(_) | Continue => Continue,
        }
    }
}

