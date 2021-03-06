use ahc001::{Rect, L};
use proconio::input;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::env::args;

fn main() {
    let steps = args().skip(1).next().unwrap().parse::<u64>().unwrap();
    input! {
        n: usize,
        xyr: [(f64, f64, f64); n],
    }

    static OUTER: [Rect; 4] = [
        Rect::new(-L, -L, L, L * 3.0),
        Rect::new(-L, -L, L * 3.0, L),
        Rect::new(L, -L, L, L * 3.0),
        Rect::new(-L, L, L * 3.0, L),
    ];

    let mut rng = SmallRng::seed_from_u64(1);
    let mut rects = Vec::with_capacity(n);
    for &(x, y, r) in xyr.iter() {
        let l = r.sqrt();
        rects.push(Rect::new(
            x - rng.gen_range(0.0, l),
            y - rng.gen_range(0.0, l),
            l,
            l,
        ));
    }

    let factor = 1.0 / 512.0;
    for _ in 0..steps {
        let i = rng.gen_range(0, n);
        let (x, y, r) = xyr[i];
        let rect = rects.get(i).unwrap();
        let attract = rect.calc_attract_force(x, y);
        let mut repulsive = (0.0, 0.0);
        for (j, other) in rects.iter().enumerate() {
            if i == j {
                continue;
            }
            let f = rect.calc_repulsive_force(other);
            repulsive.0 += f.0;
            repulsive.1 += f.1;
        }
        // for other in OUTER.iter() {
        //     let f = rect.calc_repulsive_force(other);
        //     repulsive.0 += f.0 * 10.0;
        //     repulsive.1 += f.1 * 10.0;
        // }
        let area = rect.area_force(r);
        let pos_rate = 0.5 * factor;
        let shape_rate = 0.5 * factor;
        let area_rate = 0.1 * factor;

        let rect = rects.get_mut(i).unwrap();
        rect.apply_force_x(
            attract.0 * 100.0,
            repulsive.0,
            area * area_rate,
            pos_rate,
            shape_rate,
        );
        rect.apply_force_y(
            attract.1 * 100.0,
            repulsive.1,
            area * area_rate,
            pos_rate,
            shape_rate,
        );
    }
    for rect in rects {
        println!("{}", rect.round());
    }
}
