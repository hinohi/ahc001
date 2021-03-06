use std::{
    io::BufRead,
    time::{Duration, Instant},
};

use proconio::{input, source::Source};
use rand::{
    distributions::{Distribution, Uniform},
    Rng, RngCore,
};
use rand_pcg::Mcg128Xsl64;
#[cfg(feature = "learn")]
use serde::Deserialize;

const L: i16 = 10_000;
const Q_MIN: i16 = L / 2 / 2 + 1;
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
    grid: Vec<Vec<u8>>,
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
            grid[gid].push(i as u8);
        }
        QTree { grid }
    }

    fn intersect_one_grid(&self, gid: u8, grow: &Rect, rects: &[Rect]) -> bool {
        unsafe {
            self.grid
                .get_unchecked(gid as usize)
                .iter()
                .any(|&j| grow.intersect(rects.get_unchecked(j as usize)))
        }
    }

    pub fn intersect_to_parent(&self, mut gid: u8, grow: &Rect, rects: &[Rect]) -> bool {
        loop {
            if self.intersect_one_grid(gid, grow, rects) {
                return true;
            }
            if gid == 0 {
                break false;
            }
            gid = parent_gid(gid);
        }
    }

    pub fn intersect_to_children(&self, gid: u8, grow: &Rect, rects: &[Rect]) -> bool {
        if gid >= LAYER2_OFFSET {
            return false;
        }
        children_gid_range(gid).any(|c| self.intersect_one_grid(c, grow, rects))
    }

    pub fn intersect(&self, grow: &Rect, rects: &[Rect]) -> bool {
        let gid = get_gid(grow);
        if gid == 0 {
            return intersect(grow, rects);
        }
        if self.intersect_to_parent(gid, grow, rects) {
            return true;
        }
        if self.intersect_to_children(gid, grow, rects) {
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
        let pos = self.grid[old_gid]
            .iter()
            .position(|j| *j == i as u8)
            .unwrap();
        self.grid[old_gid].swap_remove(pos);
        self.grid[new_gid].push(i as u8);
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

    pub fn slide_x(&self, d: i16) -> Option<Rect> {
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

    pub fn slide_y(&self, d: i16) -> Option<Rect> {
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

fn intersect(new: &Rect, rects: &[Rect]) -> bool {
    rects.iter().any(|rect| new.intersect(rect))
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "learn", derive(Deserialize))]
pub struct McParams {
    temp0: f64,
    temp1: f64,
    slide_d_start: f64,
    slide_d_end: f64,
    grow_d1_start: f64,
    grow_d1_end: f64,
    grow_d2_start: f64,
    grow_d2_end: f64,
    grow_d3_start: f64,
    grow_d3_end: f64,
    weight_slide_start: f64,
    weight_slide_end: f64,
    weight_d1_start: f64,
    weight_d1_end: f64,
    weight_d2_start: f64,
    weight_d2_end: f64,
    weight_d3_start: f64,
    weight_d3_end: f64,
}

fn calc_score(rects: &[Rect], sizes: &[i32]) -> (f64, Vec<f64>) {
    let mut score = 0.0;
    let mut scores = Vec::with_capacity(rects.len());
    for (rect, &r) in rects.iter().zip(sizes.iter()) {
        let s = rect.score(r);
        score += s;
        scores.push(s);
    }
    (score, scores)
}

fn mc(rng: &mut Mcg128Xsl64, params: McParams, input: &Input, limit: u64) -> (f64, Vec<Rect>) {
    let now = Instant::now();
    let limit = Duration::from_millis(limit);

    let mut rects = input.rects.to_vec();
    let (mut score, mut scores) = calc_score(&rects, &input.sizes);

    let index_sample = Uniform::new(0, rects.len());

    let mut qtree = QTree::new(&rects);
    let mut best = rects.clone();
    let mut best_score = score;
    loop {
        let elapsed = now.elapsed();
        if elapsed > limit {
            return (best_score / scores.len() as f64, best);
        }
        let t = elapsed.as_secs_f64() / limit.as_secs_f64();
        let beta = 1.0 / (params.temp0.powf(1.0 - t) * params.temp1.powf(t));

        let slide_d = Uniform::new(
            1,
            2 + (params.slide_d_start * (1.0 - t) + params.slide_d_end * t) as i16,
        );
        let grow_d1 = Uniform::new(
            1,
            2 + (params.grow_d1_start * (1.0 - t) + params.grow_d1_end * t) as i16,
        );
        let grow_d2 = Uniform::new(
            1,
            2 + (params.grow_d2_start * (1.0 - t) + params.grow_d2_end * t) as i16,
        );
        let grow_d3 = Uniform::new(
            1,
            2 + (params.grow_d3_start * (1.0 - t) + params.grow_d3_end * t) as i16,
        );
        let w_slide = params.weight_slide_start * (1.0 - t) + params.weight_slide_end * t;
        let w_d1 = params.weight_d1_start * (1.0 - t) + params.weight_d1_end * t;
        let w_d2 = params.weight_d2_start * (1.0 - t) + params.weight_d2_end * t;
        let w_d3 = params.weight_d3_start * (1.0 - t) + params.weight_d3_end * t;
        let p0 = w_slide / (w_slide + w_d1 + w_d2 + w_d3);
        let p1 = p0 + w_d1 / (w_slide + w_d1 + w_d2 + w_d3);
        let p2 = p1 + w_d2 / (w_slide + w_d1 + w_d2 + w_d3);

        let rect_slide = |rng: &mut Mcg128Xsl64, rect: &Rect| {
            let d = slide_d.sample(rng);
            match rng.next_u32() % 4 {
                0 => rect.slide_x(d),
                1 => rect.slide_x(-d),
                2 => rect.slide_y(d),
                3 => rect.slide_y(-d),
                _ => unreachable!(),
            }
        };
        let rect_grow_d1 = |rng: &mut Mcg128Xsl64, rect: &Rect| {
            let d = grow_d1.sample(rng);
            match rng.next_u32() % 8 {
                0 => rect.grow_x1(d),
                1 => rect.grow_x1(-d),
                2 => rect.grow_x2(d),
                3 => rect.grow_x2(-d),
                4 => rect.grow_y1(d),
                5 => rect.grow_y1(-d),
                6 => rect.grow_y2(d),
                7 => rect.grow_y2(-d),
                _ => unreachable!(),
            }
        };
        let rect_grow_d2 = |rng: &mut Mcg128Xsl64, rect: &Rect| {
            let d1 = grow_d2.sample(rng);
            match rng.next_u32() % 8 {
                0 => rect
                    .grow_x1(d1)
                    .and_then(|rect| rect.grow_y1(-grow_d2.sample(rng))),
                1 => rect
                    .grow_x1(-d1)
                    .and_then(|rect| rect.grow_y1(grow_d2.sample(rng))),
                2 => rect
                    .grow_x1(d1)
                    .and_then(|rect| rect.grow_y2(grow_d2.sample(rng))),
                3 => rect
                    .grow_x1(-d1)
                    .and_then(|rect| rect.grow_y2(-grow_d2.sample(rng))),
                4 => rect
                    .grow_x2(d1)
                    .and_then(|rect| rect.grow_y1(grow_d2.sample(rng))),
                5 => rect
                    .grow_x2(-d1)
                    .and_then(|rect| rect.grow_y1(-grow_d2.sample(rng))),
                6 => rect
                    .grow_x2(d1)
                    .and_then(|rect| rect.grow_y2(-grow_d2.sample(rng))),
                7 => rect
                    .grow_x2(-d1)
                    .and_then(|rect| rect.grow_y2(grow_d2.sample(rng))),
                _ => unreachable!(),
            }
        };
        let rect_grow_d3 = |rng: &mut Mcg128Xsl64, rect: &Rect| {
            let d1 = grow_d3.sample(rng);
            match rng.next_u32() % 4 {
                0 => rect
                    .grow_x1(d1)
                    .and_then(|rect| rect.grow_x2(grow_d3.sample(rng))),
                1 => rect
                    .grow_x1(-d1)
                    .and_then(|rect| rect.grow_x2(-grow_d3.sample(rng))),
                2 => rect
                    .grow_y1(d1)
                    .and_then(|rect| rect.grow_y2(grow_d3.sample(rng))),
                3 => rect
                    .grow_y1(-d1)
                    .and_then(|rect| rect.grow_y2(-grow_d3.sample(rng))),
                _ => unreachable!(),
            }
        };
        score = scores.iter().fold(0.0, |x, y| x + *y);

        for _ in 0..2000 {
            let i = index_sample.sample(rng);
            let rect = rects.get(i).unwrap();

            let p = rng.gen::<f64>();
            let new = if p < p0 {
                rect_slide(rng, rect)
            } else if p < p1 {
                rect_grow_d1(rng, rect)
            } else if p < p2 {
                rect_grow_d2(rng, rect)
            } else {
                rect_grow_d3(rng, rect)
            };
            if let Some(new) = new {
                if !new.contain(input.points[i].0, input.points[i].1) {
                    continue;
                }
                let new_score = new.score(input.sizes[i]);
                let score_diff = new_score - scores[i];
                if score_diff >= 0.0 || rng.gen::<f64>() < (score_diff * beta).exp() {
                    if let Some(grow) = rect.grow_rect(&new) {
                        if qtree.intersect(&grow, &rects) {
                            continue;
                        }
                    }
                    qtree.update(&new, rect, i);
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

const DEFAULT_PARAMS: McParams = McParams {
    temp0: 0.10868564634648839,
    temp1: 0.00029342425784192465,
    slide_d_start: 59.0,
    slide_d_end: 59.0,
    grow_d1_start: 661.4780032749206,
    grow_d1_end: 7.211273402804876,
    grow_d2_start: 1532.4395218778254,
    grow_d2_end: 3.178973798285788,
    grow_d3_start: 660.2448737846898,
    grow_d3_end: 5.668514832161116,
    weight_slide_start: 0.056353565358393205,
    weight_slide_end: 0.056353565358393205,
    weight_d1_start: 0.1745393427756753,
    weight_d1_end: 0.1745393427756753,
    weight_d2_start: 0.7691070918659315,
    weight_d2_end: 0.7691070918659315,
    weight_d3_start: 0.0,
    weight_d3_end: 0.0,
};

#[cfg(feature = "learn")]
fn get_params(arg: Option<String>) -> McParams {
    arg.and_then(|arg| Some(serde_json::de::from_str(&arg).unwrap()))
        .unwrap_or(DEFAULT_PARAMS)
}

#[cfg(not(feature = "learn"))]
fn get_params(_arg: Option<String>) -> McParams {
    DEFAULT_PARAMS
}

pub struct Input {
    pub rects: Vec<Rect>,
    pub points: Vec<(i16, i16)>,
    pub sizes: Vec<i32>,
}

pub fn parse_source<R: BufRead, S: Source<R>>(source: S) -> Input {
    input! {
        from source,
        n: usize,
        xyr: [(i16, i16, i32); n],
    }
    let mut rects = Vec::with_capacity(n);
    let mut points = Vec::with_capacity(n);
    let mut sizes = Vec::with_capacity(n);
    for &(x, y, r) in xyr.iter() {
        rects.push(Rect::new(x, x + 1, y, y + 1));
        points.push((x, y));
        sizes.push(r);
    }

    Input {
        rects,
        points,
        sizes,
    }
}

pub fn run(input: Input, arg: Option<String>) -> (f64, Vec<Rect>) {
    let mut rng = Mcg128Xsl64::new(1);
    let params = get_params(arg);
    mc(&mut rng, params.clone(), &input, 4970)
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

    #[test]
    fn test_children_grid_range() {
        assert_eq!(children_gid_range(1), 5..9);
        assert_eq!(children_gid_range(2), 9..13);
    }
}
