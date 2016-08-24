use std::ops::{Add, AddAssign, Sub, Mul, Div, Rem, Neg};

pub type Scalar = f32;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Vector {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vector {
    pub fn zero() -> Vector {
        Vector { x: 0.0, y: 0.0 }
    }

    pub fn len_sq(&self) -> Scalar {
        self.x * self.x + self.y * self.y
    }

    pub fn len(&self) -> Scalar {
        self.len_sq().sqrt()
    }

    pub fn normalized(&self) -> Vector {
        let len = self.len();
        vector(self.x / len, self.y / len)
    }
}

#[inline(always)]
pub fn vector(x: Scalar, y: Scalar) -> Vector {
    Vector { x: x, y: y }
}

impl Add<Scalar> for Vector {
    type Output = Vector;

    fn add(self, rhs: Scalar) -> Vector {
        vector(self.x + rhs, self.y + rhs)
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, other: Vector) {
        *self = Vector {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        vector(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        vector(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<Scalar> for Vector {
    type Output = Vector;

    fn mul(self, rhs: Scalar) -> Vector {
        vector(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector> for Scalar {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        vector(self * rhs.x, self * rhs.y)
    }
}

impl Div<Scalar> for Vector {
    type Output = Vector;

    fn div(self, rhs: Scalar) -> Vector {
        vector(self.x / rhs, self.y / rhs)
    }
}

impl Div<Vector> for Scalar {
    type Output = Vector;

    fn div(self, rhs: Vector) -> Vector {
        vector(self / rhs.x, self / rhs.y)
    }
}

impl Rem<Vector> for Vector {
    type Output = Vector;

    // Component-wise product
    fn rem(self, rhs: Vector) -> Vector {
        vector(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        vector(-self.x, -self.y)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rect {
    min: Vector,
    max: Vector,
}

impl Rect {
    pub fn with_min_max(min: Vector, max: Vector) -> Rect {
        Rect {
            min: min,
            max: max,
        }
    }

    pub fn with_min_size(min: Vector, size: Vector) -> Rect {
        Rect {
            min: min,
            max: min + size,
        }
    }

    pub fn with_center_size(center: Vector, size: Vector) -> Rect {
        let half_size = size / 2.0;
        Rect {
            min: center - half_size,
            max: center + half_size,
        }
    }

    pub fn with_size(size: Vector) -> Rect {
        Rect::with_center_size(Vector::zero(), size)
    }

    pub fn min(&self) -> Vector {
        self.min
    }

    pub fn max(&self) -> Vector {
        self.max
    }

    pub fn size(&self) -> Vector {
        self.max - self.min
    }

    pub fn left(&self) -> Scalar {
        self.min.x
    }

    pub fn right(&self) -> Scalar {
        self.max.x
    }

    pub fn bottom(&self) -> Scalar {
        self.min.y
    }

    pub fn top(&self) -> Scalar {
        self.max.y
    }

    pub fn offset(&self, offset: Vector) -> Rect {
        Rect::with_min_max(self.min + offset, self.max + offset)
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let min = vector(if self.min.x < other.min.x { other.min.x } else { self.min.x },
                         if self.min.y < other.min.y { other.min.y } else { self.min.y });
        let max = vector(if self.max.x > other.max.x { other.max.x } else { self.max.x },
                         if self.max.y > other.max.y { other.max.y } else { self.max.y });

        if min.x < max.x && min.y < max.y {
            Some(Rect::with_min_max(min, max))
        } else {
            None
        }
    }
}

/// The affine transform matrix:
///
///     | a c x |    | x y o |
///     | b d y | or | x y o |
///     | 0 0 1 |    | x y o |
///
/// This is matrix is used to multiply by column vector:
///
///     | a c x |   | x |
///     | b d y | * | y |
///     | 0 0 1 |   | 1 |
///
#[derive(Copy, Clone, Default, PartialEq)]
pub struct Transform {
    a: Scalar,
    b: Scalar,
    c: Scalar,
    d: Scalar,
    x: Scalar,
    y: Scalar,
}

impl Transform {
    pub fn identity() -> Transform {
        Transform {
            a: 1.0, c: 0.0, x: 0.0,
            b: 0.0, d: 1.0, y: 0.0,
        }
    }

    pub fn offset(offset: Vector) -> Transform {
        Transform {
            a: 1.0, c: 0.0, x: offset.x,
            b: 0.0, d: 1.0, y: offset.y,
        }
    }

    pub fn scale(scale: Vector) -> Transform {
        Transform {
            a:  scale.x, c: 0.0, x: 0.0,
            b: 0.0, d:  scale.y, y: 0.0,
        }
    }

    pub fn rotate(rad: Scalar) -> Transform {
        let cos = rad.cos();
        let sin = rad.sin();
        Transform {
            a: cos, c: -sin, x: 0.0,
            b: sin, d:  cos, y: 0.0,
        }
    }

    pub fn ortho(rect: Rect) -> Transform {
        // x -> (left, right)
        // x - left -> (0, right - left)
        // (x - left) / (right - left) * 2 - 1  -> (-1, 1)
        //
        // y -> (bottom, top)
        // y - bottom -> (0, top - bottom)
        // (y - bottom) / (top - bottom) * 2 - 1 -> (-1, 1)
        //
        let trans = Transform::offset(-rect.min());
        let trans = Transform::scale(2.0 / rect.size()) * trans;
        Transform::offset(vector(-1.0, -1.0)) * trans
    }

    pub fn xaxis(&self) -> Vector {
        Vector {
            x: self.a,
            y: self.b,
        }
    }

    pub fn yaxis(&self) -> Vector {
        Vector {
            x: self.c,
            y: self.d,
        }
    }

    pub fn position(&self) -> Vector {
        vector(self.x, self.y)
    }

    pub fn set_position(&mut self, position: Vector) {
        self.x = position.x;
        self.y = position.y;
    }

    pub fn offset_by(&mut self, offset: Vector) {
        self.x += offset.x;
        self.y += offset.y;
    }

    pub fn rotate_by(&mut self, rad: Scalar) {
        *self = Transform::rotate(rad) * *self;
    }

    /// If the given transform cannot be inverted, return the unchanged one.
    pub fn invert(&self) -> Transform {
        let det = self.a * self.d - self.c * self.b;

        if det == 0.0 {
            *self
        } else {
            let inv_det = 1.0 / det;

            Transform {
                a: inv_det * self.d,
                c: inv_det * -self.c,
                x: inv_det * (self.c * self.y - self.x * self.d),
                b: inv_det * -self.b,
                d: inv_det * self.a,
                y: inv_det * (self.x * self.b - self.a * self.y),
            }
        }
    }

    pub fn to_gl_mat3(&self) -> [f32; 9] {
        [
            self.a as f32, self.b as f32, 0.0,
            self.c as f32, self.d as f32, 0.0,
            self.x as f32, self.y as f32, 1.0,
        ]
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Transform {
        Transform {
            a: self.a * rhs.a + self.c * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            x: self.a * rhs.x + self.c * rhs.y + self.x,

            b: self.b * rhs.a + self.d * rhs.b,
            d: self.b * rhs.c + self.d * rhs.d,
            y: self.b * rhs.x + self.d * rhs.y + self.y,
        }
    }
}

impl Mul<Vector> for Transform {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        Vector {
            x: rhs.x * self.a + rhs.y * self.c + self.x,
            y: rhs.x * self.b + rhs.y * self.d + self.y,
        }
    }
}
