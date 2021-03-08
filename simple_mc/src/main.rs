use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use proconio::{input, source::once::OnceSource};
use rand::{
    distributions::{Distribution, Uniform},
    RngCore,
};
use rand_pcg::Mcg128Xsl64;
use serde::Deserialize;

const L: i16 = 10_000;
const Q_MIN: i16 = L / 2 / 2;
const LAYER1_OFFSET: u8 = (4 - 1) / 3;
const LAYER2_OFFSET: u8 = (4 * 4 - 1) / 3;

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
    // layer 2
    if n == 0 {
        return LAYER2_OFFSET + j;
    }
    // layer 1
    n >>= 2;
    if n == 0 {
        return LAYER1_OFFSET + (j >> 2);
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
    if gid >= LAYER2_OFFSET {
        return LAYER1_OFFSET + (gid - LAYER2_OFFSET) / 4;
    }
    0
}

pub fn children_gid_range(gid: u8) -> std::ops::Range<u8> {
    let start = (gid - LAYER1_OFFSET) * 4 + LAYER2_OFFSET;
    let end = (gid - LAYER1_OFFSET + 1) * 4 + LAYER2_OFFSET;
    return start..end;
}

#[derive(Debug)]
pub struct QTree {
    grid: Vec<Vec<usize>>,
}

impl QTree {
    pub fn new(rects: &[Rect]) -> QTree {
        const N: usize = 1 + 4 + 4 * 4;
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
        self.grid[gid as usize]
            .iter()
            .any(|&j| i != j && new.intersect(&rects[j]))
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
        if gid >= LAYER2_OFFSET {
            return false;
        }
        let mut queue = VecDeque::new();
        queue.push_front(children_gid_range(gid));
        while let Some(children) = queue.pop_front() {
            for c in children {
                if self.intersect_one_grid(c, new, i, rects) {
                    return true;
                }
                if c < LAYER2_OFFSET {
                    queue.push_back(children_gid_range(c));
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
        s * (2.0 - s)
    }

    pub fn move_x(&self, d: i16) -> Option<Rect> {
        if self.x1 + d < 0 || L <= self.x2 + d {
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
        if self.y1 + d < 0 || L <= self.y2 + d {
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
        if self.x2 + d <= self.x1 || L <= self.x2 + d {
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
        if self.y2 + d <= self.y1 || L <= self.y2 + d {
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

    pub fn grow_rect(&self, new: &Rect) -> Option<Rect> {
        if self.x1 > new.x1 {
            return Some(Rect {
                x1: new.x1,
                x2: self.x1,
                y1: new.y1,
                y2: new.y2,
            });
        }
        if self.y1 > new.y1 {
            return Some(Rect {
                x1: new.x1,
                x2: new.x2,
                y1: new.y1,
                y2: self.y1,
            });
        }
        if self.x2 < new.x2 {
            return Some(Rect {
                x1: self.x2,
                x2: new.x2,
                y1: new.y1,
                y2: new.y2,
            });
        }
        if self.y2 < new.y2 {
            return Some(Rect {
                x1: new.x1,
                x2: new.x2,
                y1: self.y2,
                y2: new.y2,
            });
        }
        None
    }
}

fn intersect(new: &Rect, i: usize, rects: &[Rect]) -> bool {
    rects
        .iter()
        .enumerate()
        .any(|(j, rect)| i != j && new.intersect(rect))
}

#[derive(Debug, Deserialize)]
pub struct McParams {
    temp0: f64,
    temp1: f64,
    move_d_max: i16,
    grow_d1_start: f64,
    grow_d1_end: f64,
    grow_d2_start: f64,
    grow_d2_end: f64,
    rect_move_weight: f64,
    rect_grow_d1_weight: f64,
    rect_grow_d2_weight: f64,
}

fn mc(
    params: McParams,
    rng: &mut Mcg128Xsl64,
    rects: &[Rect],
    target: &[(i16, i16)],
    size: &[i32],
) -> Vec<Rect> {
    let now = Instant::now();
    const TIME_LIMIT: Duration = Duration::from_millis(4950);

    let mut score = 0.0;
    let mut scores = Vec::with_capacity(rects.len());
    for (rect, &r) in rects.iter().zip(size.iter()) {
        let s = rect.score(r);
        score += s;
        scores.push(s);
    }

    #[derive(Debug, Copy, Clone)]
    enum MoveType {
        Move,
        Grow1,
        Grow2,
    }

    let weight = {
        let s = params.rect_move_weight + params.rect_grow_d1_weight + params.rect_grow_d2_weight;
        let mut weight = [MoveType::Move; 128];
        let a = (params.rect_move_weight * weight.len() as f64 / s).ceil() as usize;
        let b = a + (params.rect_grow_d1_weight * weight.len() as f64 / s).ceil() as usize;
        let c = b + (params.rect_grow_d2_weight * weight.len() as f64 / s).ceil() as usize;
        for i in 0..a {
            if i < weight.len() {
                weight[i] = MoveType::Move;
            }
        }
        for i in a..b {
            if i < weight.len() {
                weight[i] = MoveType::Grow1;
            }
        }
        for i in b..c {
            if i < weight.len() {
                weight[i] = MoveType::Grow2;
            }
        }
        weight
    };

    let move_d = Uniform::new(1, params.move_d_max + 1);
    let prob_d = Uniform::new(0.0, 1.0);

    let mut count = (0, 0, 0);
    let mut qtree = QTree::new(rects);
    let mut rects = rects.to_vec();
    let mut best = rects.clone();
    let mut best_score = score;
    loop {
        let elapsed = now.elapsed();
        if elapsed > TIME_LIMIT {
            eprintln!("{:?}", count);
            break best;
        }
        let t = elapsed.as_secs_f64() / TIME_LIMIT.as_secs_f64();
        let beta = 1.0 / (params.temp0.powf(1.0 - t) * params.temp1.powf(t));

        let grow_d1 = Uniform::new(
            1,
            (params.grow_d1_start * (1.0 - t) + params.grow_d1_end * t) as i16 + 2,
        );
        let grow_d2 = Uniform::new(
            1,
            (params.grow_d2_start * (1.0 - t) + params.grow_d2_end * t) as i16 + 2,
        );

        let rect_move = |rng: &mut Mcg128Xsl64, rect: &Rect| match rng.next_u32() % 4 {
            0 => rect.move_x(move_d.sample(rng)),
            1 => rect.move_x(-move_d.sample(rng)),
            2 => rect.move_y(move_d.sample(rng)),
            3 => rect.move_y(-move_d.sample(rng)),
            _ => unreachable!(),
        };
        let rect_grow_d1 = |rng: &mut Mcg128Xsl64, rect: &Rect| match rng.next_u32() % 8 {
            0 => rect.grow_x1(grow_d1.sample(rng)),
            1 => rect.grow_x1(-grow_d1.sample(rng)),
            2 => rect.grow_x2(grow_d1.sample(rng)),
            3 => rect.grow_x2(-grow_d1.sample(rng)),
            4 => rect.grow_y1(grow_d1.sample(rng)),
            5 => rect.grow_y1(-grow_d1.sample(rng)),
            6 => rect.grow_y2(grow_d1.sample(rng)),
            7 => rect.grow_y2(-grow_d1.sample(rng)),
            _ => unreachable!(),
        };
        let rect_grow_d2 = |rng: &mut Mcg128Xsl64, rect: &Rect| match rng.next_u32() % 8 {
            0 => rect
                .grow_x1(grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y1(-grow_d2.sample(rng))),
            1 => rect
                .grow_x1(-grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y1(grow_d2.sample(rng))),
            2 => rect
                .grow_x1(grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y2(grow_d2.sample(rng))),
            3 => rect
                .grow_x1(-grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y2(-grow_d2.sample(rng))),
            4 => rect
                .grow_x2(grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y1(grow_d2.sample(rng))),
            5 => rect
                .grow_x2(-grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y1(-grow_d2.sample(rng))),
            6 => rect
                .grow_x2(grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y2(-grow_d2.sample(rng))),
            7 => rect
                .grow_x2(-grow_d2.sample(rng))
                .and_then(|rect| rect.grow_y2(grow_d2.sample(rng))),
            _ => unreachable!(),
        };

        score = scores.iter().fold(0.0, |x, y| x + *y);

        for _ in 0..1000 {
            count.0 += 1;
            let i = (rng.next_u32() % rects.len() as u32) as usize;
            let w = weight[(rng.next_u32() % weight.len() as u32) as usize];
            let new = match w {
                MoveType::Move => rect_move(rng, &rects[i]),
                MoveType::Grow1 => rect_grow_d1(rng, &rects[i]),
                MoveType::Grow2 => rect_grow_d2(rng, &rects[i]),
            };
            if let Some(new) = new {
                count.1 += 1;
                if !new.contain(target[i].0, target[i].1) {
                    continue;
                }
                let new_score = new.score(size[i]);
                let score_diff = new_score - scores[i];
                if score_diff >= 0.0 || prob_d.sample(rng) < (score_diff * beta).exp() {
                    if let Some(grow) = rects[i].grow_rect(&new) {
                        if qtree.intersect(&grow, i, &rects) {
                            continue;
                        }
                    }
                    count.2 += 1;
                    qtree.update(&new, &rects[i], i);
                    scores[i] = new_score;
                    rects[i] = new;
                    score += score_diff;
                    if score > best_score {
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

    let default_params = McParams {
        temp0: 0.10776805748978419,
        temp1: 0.00017098824773959434,
        move_d_max: 59,
        grow_d1_start: 757.0413842816848,
        grow_d1_end: 8.632905527414328,
        grow_d2_start: 1275.2877955712484,
        grow_d2_end: 6.087155403694206,
        rect_move_weight: 0.0034894679456486492,
        rect_grow_d1_weight: 0.2550967075941691,
        rect_grow_d2_weight: 0.7587268147916909,
    };
    let params = std::env::args()
        .skip(1)
        .next()
        .and_then(|arg| Some(serde_json::de::from_str(&arg).unwrap()))
        .unwrap_or(default_params);

    let mut rng = Mcg128Xsl64::new(1);
    let best = mc(params, &mut rng, &rects, &target, &size);
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
        assert_eq!(point_to_leaf_gid(L - 1, L - 1), 4 * 4 - 1);
    }
}
