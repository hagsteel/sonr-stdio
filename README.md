# Async Stdin / Stdout reactors


```
use std::io::{stdin, stdout};
use std::os::unix::io::AsRawFd;

use bytes::{Bytes, BytesMut};
use sonr_stdio;
use sonr::prelude::*;

fn main() {
    System::init();
    let out = sonr_stdio::Stdout::new();
    let run = sonr_stdio::Stdin::new(1024).map(|out: Bytes| {
        let mut b = BytesMut::from(out);
        b.reverse();
        b.freeze()
    }).chain(out);

    System::start(run);
}
```
