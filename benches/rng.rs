use criterion::{criterion_group, criterion_main, Criterion};
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use rand_pcg::Mcg128Xsl64;

fn uniform_01(c: &mut Criterion) {
    let mut rng = Mcg128Xsl64::new(1);
    let p = Uniform::new(0.0, 1.0);
    c.bench_function("uniform_01", |b| {
        b.iter(|| p.sample(&mut rng));
    });
}

fn gen_f64(c: &mut Criterion) {
    let mut rng = Mcg128Xsl64::new(1);
    c.bench_function("gen_f64", |b| {
        b.iter(|| rng.gen::<f64>());
    });
}

criterion_group!(benches, uniform_01, gen_f64);
criterion_main!(benches);
