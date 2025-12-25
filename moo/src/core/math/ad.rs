use std::ops::{Add, Sub, Mul, Div, Neg};

/// A Dual number for Forward-Mode Automatic Differentiation.
/// Represents values in the form `a + bε` where `ε² = 0`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dual {
    /// The primal value (f(x))
    pub val: f64,
    /// The derivative value (f'(x))
    pub der: f64,
}

impl Dual {
    pub const fn new(val: f64, der: f64) -> Self {
        Self { val, der }
    }

    /// Creates a variable w.r.t which we are differentiating (seed = 1.0)
    pub const fn variable(val: f64) -> Self {
        Self { val, der: 1.0 }
    }

    /// Creates a constant value (derivative = 0.0)
    pub const fn constant(val: f64) -> Self {
        Self { val, der: 0.0 }
    }
}

impl Add for Dual {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.val + rhs.val, self.der + rhs.der)
    }
}

impl Sub for Dual {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.val - rhs.val, self.der - rhs.der)
    }
}

impl Mul for Dual {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // Product rule: (a + bε)(c + dε) = ac + (ad + bc)ε
        Self::new(self.val * rhs.val, self.val * rhs.der + self.der * rhs.val)
    }
}

impl Div for Dual {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        // Quotient rule: (a/c) + ((b*c - a*d) / c^2)ε
        let val = self.val / rhs.val;
        let der = (self.der * rhs.val - self.val * rhs.der) / (rhs.val * rhs.val);
        Self::new(val, der)
    }
}

impl Neg for Dual {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.val, -self.der)
    }
}

// TODO: Add Sin, Cos, Exp, Log via a trait like `RealField`
