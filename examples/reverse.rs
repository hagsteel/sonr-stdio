use std::io::{stdin, stdout};
use std::os::unix::io::AsRawFd;

use bytes::{Bytes, BytesMut};
use sonr_stdio;
use sonr::prelude::*;

fn main() {
    System::init();
    let out = sonr_stdio::Stdout::new().unwrap();
    let run = sonr_stdio::Stdin::new(1024).unwrap()
        .map(|mut out: BytesMut| {
            out.reverse();
            out.freeze()
        })
        .chain(out);

    System::start(run);
}
