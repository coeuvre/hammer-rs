use std::ops::Mul;

pub type Scalar = f32;

#[derive(Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
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
#[derive(Copy, Clone, PartialEq)]
pub struct Trans {
    a: Scalar,
    b: Scalar,
    c: Scalar,
    d: Scalar,
    x: Scalar,
    y: Scalar,
}

impl Trans {
    pub fn identity() -> Trans {
        Trans {
            a: 1.0, c: 0.0, x: 0.0,
            b: 0.0, d: 1.0, y: 0.0,
        }
    }

    pub fn offset(x: Scalar, y: Scalar) -> Trans {
        Trans {
            a: 1.0, c: 0.0, x: x,
            b: 0.0, d: 1.0, y: y,
        }
    }

    pub fn scale(sx: Scalar, sy: Scalar) -> Trans {
        Trans {
            a:  sx, c: 0.0, x: 0.0,
            b: 0.0, d:  sy, y: 0.0,
        }
    }

    pub fn rotate(rad: Scalar) -> Trans {
        let cos = rad.cos();
        let sin = rad.sin();
        Trans {
            a: cos, c: -sin, x: 0.0,
            b: sin, d:  cos, y: 0.0,
        }
    }

    pub fn xaxis(&self) -> Vec2 {
        Vec2 {
            x: self.a,
            y: self.b,
        }
    }

    pub fn yaxis(&self) -> Vec2 {
        Vec2 {
            x: self.c,
            y: self.d,
        }
    }

    pub fn origin(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    /// If the given transform cannot be inverted, return the unchanged one.
    pub fn invert(&self) -> Trans {
        let det = self.a * self.d - self.c * self.b;

        if det == 0.0 {
            *self
        } else {
            let inv_det = 1.0 / det;

            Trans {
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
            self.a, self.b, 0.0,
            self.c, self.d, 0.0,
            self.x, self.y, 1.0,
        ]
    }
}

impl Mul for Trans {
    type Output = Trans;

    fn mul(self, rhs: Trans) -> Trans {
        Trans {
            a: self.a * rhs.a + self.c * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            x: self.a * rhs.x + self.c * rhs.y + self.x,

            b: self.b * rhs.a + self.d * rhs.b,
            d: self.b * rhs.c + self.d * rhs.d,
            y: self.b * rhs.x + self.d * rhs.y + self.y,
        }
    }
}

impl Mul<Vec2> for Trans {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: rhs.x * self.a + rhs.y * self.c + self.x,
            y: rhs.x * self.b + rhs.y * self.d + self.y,
        }
    }
}
