use std::{
    io::stdin,
    time::{Duration, Instant},
};

use proconio::{input, source::once::OnceSource};
use rand::{
    distributions::{Distribution, Uniform},
    RngCore,
};
use rand_pcg::Mcg128Xsl64;

const L: i32 = 10_000;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x1: i32, x2: i32, y1: i32, y2: i32) -> Rect {
        Rect { x1, x2, y1, y2 }
    }

    pub fn size(&self) -> i32 {
        (self.x2 - self.x1) * (self.y2 - self.y1)
    }

    pub fn intersect(&self, other: &Rect) -> bool {
        self.x2.min(other.x2) > self.x1.max(other.x1)
            && self.y2.min(other.y2) > self.y1.max(other.y1)
    }

    /// (x + 0.5, y + 0.5) が含まれているかチェック
    pub fn contain(&self, x: i32, y: i32) -> bool {
        self.x1 <= x && x < self.x2 && self.y1 <= y && y < self.y2
    }

    pub fn score(&self, r: i32) -> f64 {
        let s = self.size().min(r) as f64 / self.size().max(r) as f64;
        1.0 - (1.0 - s) * (1.0 - s)
    }

    pub fn move_x(&self, d: i32) -> Option<Rect> {
        if self.x1 + d < 0 || L < self.x2 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1 + d,
                x2: self.x2 + d,
                y1: self.y1,
                y2: self.y2,
            })
        }
    }

    pub fn move_y(&self, d: i32) -> Option<Rect> {
        if self.y1 + d < 0 || L < self.y2 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1,
                x2: self.x2,
                y1: self.y1 + d,
                y2: self.y2 + d,
            })
        }
    }

    pub fn grow_x1(&self, d: i32) -> Option<Rect> {
        if self.x1 + d < 0 || self.x2 <= self.x1 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1 + d,
                x2: self.x2,
                y1: self.y1,
                y2: self.y2,
            })
        }
    }

    pub fn grow_x2(&self, d: i32) -> Option<Rect> {
        if self.x2 + d <= self.x1 || L < self.x2 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1,
                x2: self.x2 + d,
                y1: self.y1,
                y2: self.y2,
            })
        }
    }

    pub fn grow_y1(&self, d: i32) -> Option<Rect> {
        if self.y1 + d < 0 || self.y2 <= self.y1 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1,
                x2: self.x2,
                y1: self.y1 + d,
                y2: self.y2,
            })
        }
    }
    pub fn grow_y2(&self, d: i32) -> Option<Rect> {
        if self.y2 + d <= self.y1 || L < self.y2 + d {
            None
        } else {
            Some(Rect {
                x1: self.x1,
                x2: self.x2,
                y1: self.y1,
                y2: self.y2 + d,
            })
        }
    }
}

fn intersect(new: &Rect, i: usize, rects: &[Rect]) -> bool {
    rects
        .iter()
        .enumerate()
        .any(|(j, rect)| i != j && new.intersect(rect))
}

enum IntersectDirection {
    X,
    Y,
    None,
}

fn mc(
    rng: &mut Mcg128Xsl64,
    rects: &[Rect],
    target: &[(i32, i32)],
    size: &[i32],
) -> (f64, Vec<Rect>) {
    let now = Instant::now();
    const TIME_LIMIT: Duration = Duration::from_millis(990);

    let mut score = 0.0;
    let mut scores = Vec::with_capacity(rects.len());
    for (rect, &r) in rects.iter().zip(size.iter()) {
        let s = rect.score(r);
        score += s;
        scores.push(s);
    }

    let move_d = Uniform::new(1, 2 + 1);
    let grow_d = Uniform::new(1, 8 + 1);
    let prob_d = Uniform::new(0.0, 1.0);

    let mut rects = rects.to_vec();
    let mut best = rects.clone();
    let mut best_score = score;
    let temp0: f64 = 1.0;
    let temp1: f64 = 0.0001;
    loop {
        let elapsed = now.elapsed();
        if elapsed > TIME_LIMIT {
            break (best_score, best);
        }
        let t = elapsed.as_secs_f64() / TIME_LIMIT.as_secs_f64();
        let beta = 1.0 / (temp0.powf(1.0 - t) * temp1.powf(t));

        score = scores.iter().fold(0.0, |x, y| x + *y);

        for _ in 0..1000 {
            let i = (rng.next_u32() % rects.len() as u32) as usize;
            let (new, id) = match rng.next_u32() % 12 {
                0 => (rects[i].move_x(move_d.sample(rng)), IntersectDirection::X),
                1 => (rects[i].move_x(-move_d.sample(rng)), IntersectDirection::X),
                2 => (rects[i].move_y(move_d.sample(rng)), IntersectDirection::Y),
                3 => (rects[i].move_y(-move_d.sample(rng)), IntersectDirection::Y),
                4 => (
                    rects[i].grow_x1(grow_d.sample(rng)),
                    IntersectDirection::None,
                ),
                5 => (rects[i].grow_x1(-grow_d.sample(rng)), IntersectDirection::X),
                6 => (rects[i].grow_x2(grow_d.sample(rng)), IntersectDirection::X),
                7 => (
                    rects[i].grow_x2(-grow_d.sample(rng)),
                    IntersectDirection::None,
                ),
                8 => (
                    rects[i].grow_y1(grow_d.sample(rng)),
                    IntersectDirection::None,
                ),
                9 => (rects[i].grow_y1(-grow_d.sample(rng)), IntersectDirection::Y),
                10 => (rects[i].grow_y2(grow_d.sample(rng)), IntersectDirection::Y),
                11 => (
                    rects[i].grow_y2(-grow_d.sample(rng)),
                    IntersectDirection::None,
                ),
                _ => unreachable!(),
            };
            if let Some(new) = new {
                if !new.contain(target[i].0, target[i].1) {
                    continue;
                }
                let intersected = match id {
                    IntersectDirection::X | IntersectDirection::Y => intersect(&new, i, &rects),
                    IntersectDirection::None => false,
                };
                if intersected {
                    continue;
                }
                let new_score = new.score(size[i]);
                let score_diff = new_score - scores[i];
                if score_diff >= 0.0 || prob_d.sample(rng) < (score_diff * beta).exp() {
                    scores[i] = new_score;
                    rects[i] = new;
                    score += score_diff;
                    if score > best_score {
                        // eprintln!("best {}", best_score);
                        best_score = score;
                        best = rects.clone();
                    }
                }
            }
        }
    }
}

fn main() {
    let stdin = stdin();
    let f = stdin.lock();
    // let f = std::io::BufReader::new(std::fs::File::open("./tools/in/0001.txt").unwrap());
    let source = OnceSource::new(f);
    input! {
        from source,
        n: usize,
        xyr: [(i32, i32, i32); n],
    }
    let mut rects = Vec::with_capacity(n);
    let mut target = Vec::with_capacity(n);
    let mut size = Vec::with_capacity(n);
    for &(x, y, r) in xyr.iter() {
        rects.push(Rect::new(x, x + 1, y, y + 1));
        target.push((x, y));
        size.push(r);
    }

    for i in 0..n {
        for j in i + 1..n {
            if rects[i].intersect(&rects[j]) {
                eprintln!("{} - {} are intersect", i, j);
            }
        }
    }

    let mut rng = Mcg128Xsl64::new(1);
    let mut best = rects.clone();
    let mut best_score = 0.0;
    for _ in 0..5 {
        let (s, r) = mc(&mut rng, &rects, &target, &size);
        if s > best_score {
            best_score = s;
            best = r;
        }
    }

    for rect in best {
        println!("{} {} {} {}", rect.x1, rect.y1, rect.x2, rect.y2);
    }
}
