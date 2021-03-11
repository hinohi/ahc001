#![allow(non_snake_case)]

use rand::prelude::*;
use std::{collections::BTreeSet, io::prelude::*};

fn gen(path: &str, seed: u64, n: usize) -> std::io::Result<()> {
    eprintln!("generating {}", path);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    // let n = (50.0 * 4.0f64.powf(rng.gen::<f64>())).round() as usize;
    let mut ps = vec![];
    let mut used = BTreeSet::new();
    for _ in 0..n {
        loop {
            let x = rng.gen_range(0, 10000);
            let y = rng.gen_range(0, 10000);
            if used.insert((x, y)) {
                ps.push((x, y));
                break;
            }
        }
    }
    let mut q = rand::seq::index::sample(&mut rng, 10000 * 10000 - 1, n - 1)
        .into_iter()
        .map(|a| a + 1)
        .collect::<Vec<_>>();
    q.sort();
    q.insert(0, 0);
    q.push(10000 * 10000);
    let mut r = vec![];
    for i in 0..n {
        r.push(q[i + 1] - q[i]);
    }
    writeln!(f, "{}", n)?;
    for i in 0..n {
        writeln!(f, "{} {} {}", ps[i].0, ps[i].1, r[i])?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(1);
    for n in 50..=200 {
        std::fs::create_dir(format!("in2/{:03}", n))?;
        for id in 0..100 {
            gen(&format!("in2/{:03}/{:04}.txt", n, id), rng.next_u64(), n)?;
        }
    }
    Ok(())
}
