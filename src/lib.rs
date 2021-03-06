pub const L: f64 = 10000.0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Rect) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        if self.x < other.x {
            Some(Less)
        } else if self.x == other.x {
            self.y.partial_cmp(&other.y)
        } else {
            Some(Greater)
        }
    }
}

impl Eq for Rect {}

impl Ord for Rect {
    fn cmp(&self, other: &Rect) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn round(&self) -> String {
        let top_left_x = (self.x - self.width * 0.5).ceil() as i32;
        let top_left_y = (self.y - self.height * 0.5).ceil() as i32;
        let bottom_right_x = (self.x + self.width * 0.5).floor() as i32;
        let bottom_right_y = (self.y + self.height * 0.5).floor() as i32;
        format!(
            "{} {} {} {}",
            top_left_x, top_left_y, bottom_right_x, bottom_right_y,
        )
    }

    pub fn gx(&self) -> f64 {
        self.x
    }

    pub fn gy(&self) -> f64 {
        self.y
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn calc_repulsive_force(&self, other: &Rect) -> (f64, f64) {
        if let Some(fx) = self.calc_repulsive_force_x(other) {
            if let Some(fy) = self.calc_repulsive_force_y(other) {
                return (fx, fy);
            }
        }
        (0.0, 0.0)
    }

    pub fn calc_attract_force(&self, x: f64, y: f64) -> (f64, f64) {
        if let Some(fx) = self.calc_attract_force_x(x) {
            if let Some(fy) = self.calc_attract_force_y(y) {
                return (fx, fy);
            }
        }
        (0.0, 0.0)
    }

    /// self に働くX方向の力
    pub fn calc_repulsive_force_x(&self, other: &Rect) -> Option<f64> {
        if self.x + self.width < other.x {
            None
        } else if other.x + other.width < self.x {
            None
        } else {
            if self.gx() < other.gx() {
                Some(other.x - self.x - self.width)
            } else {
                Some(other.x + other.width - self.x)
            }
        }
    }

    /// self に働くY方向の力
    pub fn calc_repulsive_force_y(&self, other: &Rect) -> Option<f64> {
        if self.y + self.height < other.y {
            None
        } else if other.y + other.height < self.y {
            None
        } else {
            if self.gy() < other.gy() {
                Some(other.y - self.y - self.height)
            } else {
                Some(other.y + other.height - self.y)
            }
        }
    }

    pub fn calc_attract_force_x(&self, x: f64) -> Option<f64> {
        if x < self.x {
            Some(x - self.x)
        } else if self.x + self.width < x {
            Some(x - self.x - self.width)
        } else {
            None
        }
    }

    pub fn calc_attract_force_y(&self, y: f64) -> Option<f64> {
        if y < self.y {
            Some(y - self.y)
        } else if self.y + self.height < y {
            Some(y - self.y - self.height)
        } else {
            None
        }
    }

    pub fn area_force(&self, r: f64) -> f64 {
        r - self.height * self.width
    }

    pub fn apply_force_x(
        &mut self,
        attract: f64,
        repulsive: f64,
        area: f64,
        pos_rate: f64,
        shape_rate: f64,
    ) {
        self.x += (attract + repulsive) * pos_rate;
        self.width += -repulsive.abs() * shape_rate + area;
        if self.width <= 1.0 {
            self.width = 1.0;
        }
    }

    pub fn apply_force_y(
        &mut self,
        attract: f64,
        repulsive: f64,
        area: f64,
        pos_rate: f64,
        shape_rate: f64,
    ) {
        self.y += (attract + repulsive) * pos_rate;
        self.height += -repulsive.abs() * shape_rate + area;
        if self.height <= 1.0 {
            self.height = 1.0;
        }
    }
}
