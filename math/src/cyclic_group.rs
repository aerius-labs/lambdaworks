use crate::unsigned_integer::traits::IsUnsignedInteger;
#[cfg(feature = "constant-time")]
use subtle::ConditionallySelectable;

macro_rules! define_group_trait {
    ($($bound:ident),*) => {
        pub trait IsGroup: $($bound +)* {
            /// Returns the neutral element of the group. The equality
            /// `neutral_element().operate_with(g) == g` must hold
            /// for every group element `g`.
            fn neutral_element() -> Self;

            /// Check if an element the neutral element.
            fn is_neutral_element(&self) -> bool {
                self == &Self::neutral_element()
            }

            /// Applies the group operation `times` times with itself
            /// The operation can be addition or multiplication depending on
            /// the notation of the particular group.
            #[cfg(not(feature = "constant-time"))]
            fn operate_with_self<T: IsUnsignedInteger>(&self, mut exponent: T) -> Self {
                let mut result = Self::neutral_element();
                let mut base = self.clone();

                while exponent != T::from(0) {
                    if exponent & T::from(1) == T::from(1) {
                        result = Self::operate_with(&result, &base);
                    }
                    exponent = exponent >> 1;
                    base = Self::operate_with(&base, &base);
                }
                result
            }
            #[cfg(feature = "constant-time")]
            fn operate_with_self<T: IsUnsignedInteger>(&self, exponent: T) -> Self {
                let mut r0 = Self::neutral_element();
                let mut r1 = self.clone();

                let num_bits = core::mem::size_of::<T>() * 8;

                for i in (0..num_bits).rev() {
                    let mask = T::from(1) << i;
                    let bit = (((exponent & mask) >> i) == T::from(1)) as u8;

                    Self::conditional_swap(&mut r0, &mut r1, bit.into());
                    r1 = Self::operate_with(&r0, &r1);            // r1 = r0 + r1;
                    r0 = Self::operate_with(&r0, &r0);            // r0 = 2r0;
                    Self::conditional_swap(&mut r0, &mut r1, bit.into());
                }
                r0
            }

            /// Applies the group operation between `self` and `other`.
            /// The operation can be addition or multiplication depending on
            /// the notation of the particular group.
            fn operate_with(&self, other: &Self) -> Self;

            fn neg(&self) -> Self;
        }

    };
}

#[cfg(not(feature = "constant-time"))]
define_group_trait!(Clone, PartialEq, Eq);

#[cfg(feature = "constant-time")]
define_group_trait!(Clone, PartialEq, Eq, ConditionallySelectable);
