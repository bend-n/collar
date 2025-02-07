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

impl<const N: usize, E: std::fmt::Display> std::fmt::Display for Error<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.error, &self.at) {
            (Some(x), at) => write!(f, "{x} @ {at} of {N}"),
            (None, at) => write!(
                f,
                "couldnt fill array of length {N}, only had {at} elements.",
            ),
        }
    }
}

impl<const N: usize, E: std::fmt::Debug> std::fmt::Debug for Error<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.error, &self.at) {
            (Some(x), at) => write!(f, "{x:?} @ {at} of {N}"),
            (None, at) => write!(f, "Size(wanted {N}, had {at})"),
        }
    }
}

impl<const N: usize, E: std::error::Error + 'static> std::error::Error for Error<N, E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.error.as_ref()?)
    }
}
