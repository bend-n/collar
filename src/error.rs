/// This error is returned by [`try_collect_array`](super::CollectArray::try_collect_array)
#[derive(Clone, Copy, Hash)]
pub struct Error<const N: usize, E> {
    /// Error returned by <code>[next](Iterator::next)()?.error</code> (`()` if [`None`]).
    pub error: Option<E>,
    /// Point of error.
    pub at: usize,
}

impl<const N: usize, const O: usize, E: PartialEq> PartialEq<Error<O, E>> for Error<N, E> {
    fn eq(&self, other: &Error<O, E>) -> bool {
        (self.error == other.error) & (self.at == other.at)
    }
}

impl<const N: usize, E: core::fmt::Display> core::fmt::Display for Error<N, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match (&self.error, &self.at) {
            (Some(x), at) => write!(f, "{x} @ {at} of {N}"),
            (None, at) => write!(
                f,
                "couldnt fill array of length {N}, only had {at} elements.",
            ),
        }
    }
}

impl<const N: usize, E: core::fmt::Debug> core::fmt::Debug for Error<N, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match (&self.error, &self.at) {
            (Some(x), at) => write!(f, "{x:?} @ {at} of {N}"),
            (None, at) => write!(f, "Size(wanted {N}, had {at})"),
        }
    }
}

impl<const N: usize, E: core::error::Error + 'static> core::error::Error for Error<N, E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(self.error.as_ref()?)
    }
}
