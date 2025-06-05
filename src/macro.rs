//! Private macros specific to this crate.
//
// TODO remove `jelal_proc` and add the final method here

/// Create a primitive (no constructor) that has conversions and add/sub saturating at min..=max.
///
/// This assumes that `Self::MAX` when unsigned is small not enough to fit a `$signed::MAX` value
/// otherwise some values may be lost when adding or removing near those boundaries.
///
/// Positive does not mean "unsigned". If an `unsigned: ALIAS = INNER_PRIMITIVE` is given, the type
/// will be primarily positive and if not given, a more limited implementation supporting or signed
/// values is implemented. Since "signed" covers a wider range of operations, an unsigned may be
/// more inclined to convert to signed but the vice versa is rarely true that is why the signed
/// conversion traits are more limited.
///
/// The sturcts traits here assume a `const fn new(PRIMITIVE) -> Self` method. Use
/// [`impl_new_saturate`] if unsure what to implement.
///
/// Any tokens to `skip_i32_helpers` will skip automatic `From<i32>` and arithmatic `i32`
/// implementations. Since `i32` is basically the default unsuffixed type for numbers (or it seems
/// so without investigating), automatic `i32` implementations are important for a seemless usage of
/// the transparent types.
//
// TODO add tests for each generated MIN and MAX to make sure "as X" used so frequently won't
// overflow or else.
macro_rules! int_wrapper {
    (
        ident: $ident:ident,
        signed: $signed:ident,
        $(unsigned: $unsigned:ident,)?
        $(skip_i32_helpers: $skip_i32_helpers:tt,)?
    ) => {
        impl $ident {
            /// Add another value to this, also ensure its valid and if this would fail normally.
            ///
            /// If the normal calculation of results would produce and invalid instance, this will
            /// return true.
            #[must_use]
            pub const fn add_strict(self, rhs: $signed) -> DidSaturate<Self> {
                match int_wrapper!(
                    if $($unsigned)? {
                        self.0.checked_add_signed(rhs)
                    } else {
                        self.0.checked_add(rhs)
                    }
                ){
                    Some(v) => {
                        let result = Self::new(v);
                        DidSaturate::new(result.0 != v, result)
                    }
                    None if rhs.is_negative() => DidSaturate::saturated(Self::MIN),
                    None => DidSaturate::saturated(Self::MAX),
                }
            }
        }

        impl core::fmt::Display for $ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl core::ops::Deref for $ident {
            type Target = int_wrapper!(
                if $($unsigned)? { $($unsigned)? } else { $signed }
            );

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<int_wrapper!($($unsigned)? or $signed)> for $ident {
            fn from(value: int_wrapper!($($unsigned)? or $signed)) -> Self {
                Self::new(value)
            }
        }

        impl From<$ident> for int_wrapper!($($unsigned)? or $signed) {
            fn from(value: $ident) -> Self {
                value.0
            }
        }

        impl<T> core::ops::Add<T> for $ident
        where
            T: Into<$signed>,
        {
            type Output = Self;

            fn add(self, rhs: T) -> Self::Output {
                self.add_strict(rhs.into()).result
            }
        }

        impl<T> core::ops::AddAssign<T> for $ident
        where
            Self: core::ops::Add<T, Output = Self>,
        {
            fn add_assign(&mut self, rhs: T) {
                *self = (*self + rhs);
            }
        }

        impl<T> core::ops::Sub<T> for $ident
        where
            T: Into<$signed>,
        {
            type Output = Self;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn sub(self, rhs: T) -> Self::Output {
                self + rhs.into().saturating_neg()
            }
        }

        impl<T> core::ops::SubAssign<T> for $ident
        where
            Self: core::ops::Sub<T, Output = Self>,
        {
            fn sub_assign(&mut self, rhs: T) {
                *self = (*self - rhs);
            }
        }

        // This might be already implemented so this is a way to skip it.
        int_wrapper!(
            if $($skip_i32_helpers)? {
            } else {
                int_wrapper!(
                    if $($unsigned)? {
                        impl From<i32> for $ident {
                            fn from(value: i32) -> Self {
                                Self::new(value.saturating_abs() as $($unsigned)?)
                            }
                        }
                    } else {
                        impl From<i32> for $ident {
                            fn from(value: i32) -> Self {
                                Self::new(value as $signed)
                            }
                        }
                    }
                );
            }
        );

        // unsigned only operations
        $(
            impl From<$ident> for $signed {
                fn from(value: $ident) -> Self {
                    value.0 as $signed
                }
            }

            impl From<$signed> for $ident {
                fn from(value: $signed) -> Self {
                    if value.is_negative() {
                        Self::MIN
                    } else {
                        Self::new(value as $unsigned)
                    }
                }
            }
        )?
    };

    (if { $($_:tt)* } else { $($this:tt)* }) => {
        $($this)*
    };

    (if $criterion:tt { $($this:tt)* } else { $($_:tt)* }) => {
        $($this)*
    };

    ($this:tt or $_:tt) => {
        $this
    };

    ( or $this:tt) => {
        $this
    };
}
