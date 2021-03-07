use std::time::{Duration, Instant};

use proconio::{input, source::once::OnceSource};
use rand::{
    distributions::{Distribution, Uniform},
    RngCore,
};
use rand_pcg::Mcg128Xsl64;

const L: i16 = 10_000;
const Q_MIN: i16 = L / 2 / 2 / 2;
const LAYER1_OFFSET: u8 = (4 - 1) / 3;
const LAYER2_OFFSET: u8 = (4 * 4 - 1) / 3;
const LAYER3_OFFSET: u8 = (4 * 4 * 4 - 1) / 3;

/// 最大4bitの情報がある n を受け取る
/// 00 00 ab cd
/// ↓
/// 00 ab ** cd
/// ↓
/// 00 ab 00 cd
/// ↓
/// 0a *b 0c *d
/// ↓
/// 0a 0b 0c 0d
fn bit_separate(mut n: u8) -> u8 {
    n = (n | (n << 2)) & 0b_0011_0011;
    n = (n | (n << 1)) & 0b_0101_0101;
    n
}

fn point_to_leaf_gid(x: i16, y: i16) -> u8 {
    let x = (x / Q_MIN) as u8;
    let y = (y / Q_MIN) as u8;
    bit_separate(x) | (bit_separate(y) << 1)
}

fn leaf_leaf_to_gid(i: u8, j: u8) -> u8 {
    let mut n = i ^ j;
    // layer 3
    if n == 0 {
        return LAYER3_OFFSET + j;
    }
    // layer 2
    n >>= 2;
    if n == 0 {
        return LAYER2_OFFSET + (j >> 2);
    }
    // layer 1
    n >>= 2;
    if n == 0 {
        return LAYER1_OFFSET + (j >> 4);
    }
    // layer 0 = root
    0
}

pub fn get_gid(rect: &Rect) -> u8 {
    leaf_leaf_to_gid(
        point_to_leaf_gid(rect.x1, rect.y1),
        point_to_leaf_gid(rect.x2, rect.y2),
    )
}

pub fn parent_gid(gid: u8) -> u8 {
    if gid >= LAYER3_OFFSET {
        return LAYER2_OFFSET + (gid - LAYER3_OFFSET) / 4;
    }
    if gid >= LAYER2_OFFSET {
        return LAYER1_OFFSET + (gid - LAYER2_OFFSET) / 4;
    }
    0
}

pub fn children_gid_range(gid: u8) -> std::ops::Range<u8> {
    // gid == 0 と gid >= LAYER3_OFFSET はこない
    if gid < LAYER2_OFFSET {
        let start = (gid - LAYER1_OFFSET) * 4 + LAYER2_OFFSET;
        let end = (gid - LAYER1_OFFSET + 1) * 4 + LAYER2_OFFSET;
        return start..end;
    }
    let start = (gid - LAYER2_OFFSET) * 4 + LAYER3_OFFSET;
    let end = (gid - LAYER2_OFFSET + 1) * 4 + LAYER3_OFFSET;
    return start..end;
}

#[derive(Debug)]
pub struct QTree {
    grid: Vec<Vec<usize>>,
}

impl QTree {
    pub fn new(rects: &[Rect]) -> QTree {
        const N: usize = 1 + 4 + 4 * 4 + 4 * 4 * 4;
        let mut grid = Vec::with_capacity(N);
        for _ in 0..N {
            grid.push(Vec::new());
        }
        for (i, rect) in rects.iter().enumerate() {
            let gid = get_gid(rect) as usize;
            grid[gid].push(i);
        }
        QTree { grid }
    }

    fn intersect_one_grid(&self, gid: u8, new: &Rect, i: usize, rects: &[Rect]) -> bool {
        for &j in self.grid[gid as usize].iter() {
            if i == j {
                continue;
            }
            if new.intersect(&rects[j]) {
                return true;
            }
        }
        false
    }

    pub fn intersect_to_parent(&self, mut gid: u8, new: &Rect, i: usize, rects: &[Rect]) -> bool {
        loop {
            if self.intersect_one_grid(gid, new, i, rects) {
                return true;
            }
            if gid == 0 {
                break false;
            }
            gid = parent_gid(gid);
        }
    }

    pub fn intersect_to_children(&self, gid: u8, new: &Rect, i: usize, rects: &[Rect]) -> bool {
        if gid >= LAYER3_OFFSET {
            return false;
        }
        let mut stack = vec![children_gid_range(gid)];
        while let Some(children) = stack.pop() {
            for c in children {
                if self.intersect_one_grid(c, new, i, rects) {
                    return true;
                }
                if c < LAYER3_OFFSET {
                    stack.push(children_gid_range(c));
                }
            }
        }
        false
    }

    pub fn intersect(&self, new: &Rect, i: usize, rects: &[Rect]) -> bool {
        let gid = get_gid(new);
        if gid == 0 {
            return intersect(new, i, rects);
        }
        if self.intersect_to_parent(gid, new, i, rects) {
            return true;
        }
        if self.intersect_to_children(gid, new, i, rects) {
            return true;
        }
        false
    }

    pub fn update(&mut self, new: &Rect, old: &Rect, i: usize) {
        let old_gid = get_gid(old) as usize;
        let new_gid = get_gid(new) as usize;
        if old_gid == new_gid {
            return;
        }
        let pos = self.grid[old_gid].iter().position(|j| *j == i).unwrap();
        self.grid[old_gid].swap_remove(pos);
        self.grid[new_gid].push(i);
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x1: i16,
    pub x2: i16,
    pub y1: i16,
    pub y2: i16,
}

impl Rect {
    pub fn new(x1: i16, x2: i16, y1: i16, y2: i16) -> Rect {
        Rect { x1, x2, y1, y2 }
    }

    pub fn size(&self) -> i32 {
        (self.x2 - self.x1) as i32 * (self.y2 - self.y1) as i32
    }

    pub fn intersect(&self, other: &Rect) -> bool {
        self.x2.min(other.x2) > self.x1.max(other.x1)
            && self.y2.min(other.y2) > self.y1.max(other.y1)
    }

    /// (x + 0.5, y + 0.5) が含まれているかチェック
    pub fn contain(&self, x: i16, y: i16) -> bool {
        self.x1 <= x && x < self.x2 && self.y1 <= y && y < self.y2
    }

    pub fn score(&self, r: i32) -> f64 {
        let s = self.size().min(r) as f64 / self.size().max(r) as f64;
        1.0 - (1.0 - s) * (1.0 - s)
    }

    pub fn move_x(&self, d: i16) -> Option<Rect> {
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

    pub fn move_y(&self, d: i16) -> Option<Rect> {
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

    pub fn grow_x1(&self, d: i16) -> Option<Rect> {
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

    pub fn grow_x2(&self, d: i16) -> Option<Rect> {
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

    pub fn grow_y1(&self, d: i16) -> Option<Rect> {
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
    pub fn grow_y2(&self, d: i16) -> Option<Rect> {
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
    target: &[(i16, i16)],
    size: &[i32],
) -> (f64, Vec<Rect>) {
    let now = Instant::now();
    const TIME_LIMIT: Duration = Duration::from_millis(4950);

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

    let mut qtree = QTree::new(rects);
    let mut rects = rects.to_vec();
    let mut best = rects.clone();
    let mut best_score = score;
    let temp0: f64 = 10.0;
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
                let new_score = new.score(size[i]);
                let score_diff = new_score - scores[i];
                if score_diff >= 0.0 || prob_d.sample(rng) < (score_diff * beta).exp() {
                    let intersected = match id {
                        IntersectDirection::X | IntersectDirection::Y => {
                            qtree.intersect(&new, i, &rects)
                        }
                        IntersectDirection::None => false,
                    };
                    if intersected {
                        continue;
                    }

                    qtree.update(&new, &rects[i], i);
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
    let stdin = std::io::stdin();
    let f = stdin.lock();
    // let f = std::io::BufReader::new(std::fs::File::open("./tools/in/0001.txt").unwrap());
    let source = OnceSource::new(f);
    input! {
        from source,
        n: usize,
        xyr: [(i16, i16, i32); n],
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
    let (_, best) = mc(&mut rng, &rects, &target, &size);
    for rect in best {
        println!("{} {} {} {}", rect.x1, rect.y1, rect.x2, rect.y2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_separate() {
        assert_eq!(bit_separate(0b000000), 0b000000);
        assert_eq!(bit_separate(0b000001), 0b000001);
        assert_eq!(bit_separate(0b000111), 0b010101);
        assert_eq!(bit_separate(0b000100), 0b010000);
        assert_eq!(bit_separate(0b000101), 0b010001);
    }

    #[test]
    fn test_point_to_leaf_gid() {
        assert_eq!(point_to_leaf_gid(0, 0), 0);
        assert_eq!(point_to_leaf_gid(L - 1, L - 1), 4 * 4 * 4 - 1);
    }
}
