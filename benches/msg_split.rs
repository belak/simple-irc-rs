use bencher::{benchmark_group, benchmark_main, Bencher};

use simple_irc::Message;
use std::convert::TryFrom;

fn parse_simple(bench: &mut Bencher) {
    bench.iter(|| Message::try_from("PING :PONG"));
}

fn parse_complex(bench: &mut Bencher) {
    bench.iter(|| Message::try_from("@a=b;c=d;e=\\\\ :hello-world PING PONG :EXTRA"));
}

benchmark_group!(benches, parse_simple, parse_complex);
benchmark_main!(benches);
