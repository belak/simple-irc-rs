use bencher::{benchmark_group, benchmark_main, Bencher};

use simple_irc::Message;

fn parse_simple(bench: &mut Bencher) {
    bench.iter(|| "PING :PONG".parse::<Message>());
}

fn parse_complex(bench: &mut Bencher) {
    bench.iter(|| "@a=b;c=d;e=\\\\ :hello-world PING PONG :EXTRA".parse::<Message>());
}

benchmark_group!(benches, parse_simple, parse_complex);
benchmark_main!(benches);
