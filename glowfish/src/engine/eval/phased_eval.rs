#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PhasedEval(pub i16, pub i16);

impl PhasedEval {
    pub const ZERO: PhasedEval = PhasedEval(0, 0);
}

macro_rules! impl_math_ops {
    ($($trait:ident::$fn:ident),*) => {
        $(
            impl core::ops::$trait for PhasedEval {
                type Output = Self;

                #[inline(always)]
                fn $fn(self, other: Self) -> Self::Output {
                    Self(
                        core::ops::$trait::$fn(self.0, other.0),
                        core::ops::$trait::$fn(self.1, other.1)
                    )
                }
            }
        )*
    };
}
impl_math_ops! {
    Add::add,
    Sub::sub,
    Mul::mul,
    Div::div
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident::$fn:ident),*) => {
        $(impl core::ops::$trait for PhasedEval {
            #[inline(always)]
            fn $fn(&mut self, other: Self) {
                core::ops::$trait::$fn(&mut self.0, other.0);
                core::ops::$trait::$fn(&mut self.1, other.1);
            }
        })*
    };
}
impl_math_assign_ops! {
    AddAssign::add_assign,
    SubAssign::sub_assign,
    MulAssign::mul_assign,
    DivAssign::div_assign
}

impl core::ops::Neg for PhasedEval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}
