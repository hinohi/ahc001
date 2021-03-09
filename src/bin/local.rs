use proconio::source::once::OnceSource;

use simulated_annealing::{parse_source, run};

fn main() {
    let mut args = std::env::args().skip(1);
    let stdin = std::io::stdin();
    let f = stdin.lock();
    // let f = std::io::BufReader::new(std::fs::File::open("./tools/in/0001.txt").unwrap());
    let source = OnceSource::new(f);
    let input = parse_source(source);

    let (_, best) = run(input, args.next());
    for rect in best {
        println!("{} {} {} {}", rect.x1, rect.y1, rect.x2, rect.y2);
    }
}
