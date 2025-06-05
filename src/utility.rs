//! Miscellaneous generic utilities.

/// Did the results of the last operation (`+` for example), saturate or not.
///
/// This is supposed to behave like `Option<T>` of `checked_*` operations but more concrete and
/// uniquely defined for better usage in const-context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "c", repr(C))]
pub struct DidSaturate<T> {
    /// Whether the results was saturated or modified slightly to valid results or `+` would do.
    pub did_saturate: bool,
    /// The result of the add.
    pub result: T,
}

impl<T> DidSaturate<T> {
    /// Create an instance with [`Self::did_saturate`] set to true.
    pub const fn saturated(result: T) -> Self {
        Self::new(true, result)
    }

    /// Create an instance with [`Self::did_saturate`] set to false.
    pub const fn not_saturated(result: T) -> Self {
        Self::new(false, result)
    }

    /// A shorthand for creation.
    //
    // Sometimes the results are passed and did_saturate may use it, having did_saturate at the
    // start makes that possible without an extra binding/variable
    pub const fn new(did_saturate: bool, result: T) -> Self {
        Self {
            did_saturate,
            result,
        }
    }
}

impl<T> From<DidSaturate<T>> for Option<T> {
    fn from(value: DidSaturate<T>) -> Self {
        match value.did_saturate {
            true => Some(value.result),
            false => None,
        }
    }
}

impl<T> PartialEq<T> for DidSaturate<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.result.eq(other)
    }
}

impl<T> PartialOrd<T> for DidSaturate<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.result.partial_cmp(other)
    }
}
