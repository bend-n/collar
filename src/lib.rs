#![no_std]
//! provides [`collect_array`](CollectArray::collect_array) and [`try_from_fn`].
//! allowing easy vec-free collection to an array.
//!
//! ```
//! # /*
//! use collar::*;
//! let Some([ty, path, http]) = request.split(' ').collect_array_checked() else {
//!     return;
//! };
//! # */
//! ```

use core::{
    mem::{ManuallyDrop as MD, MaybeUninit as MU, forget},
    ptr::drop_in_place,
};
use error::Error;
pub use error::Error as CollectorError;
mod error;
mod maybe;
use maybe::Maybe;

/// Collect to an array.
pub trait CollectArray: Iterator + Sized {
    /// Lets you collect an iterator into a fixed length array with no vec allocation.
    /// Handle remainder as you wish. Does not consume the iterator.
    ///
    /// # Panics
    ///
    /// when <code>[next](Iterator::next)() is [None]</code> before the array is filled.
    /// for a non panicking alternative, see [`collect_array_checked`](CollectArray::collect_array_checked).
    ///
    /// ```
    /// use collar::*;
    /// let array = (0usize..).map(|x| x * 2).collect_array();
    /// // indexes are:    0  1  2  3  4  5   6   7
    /// assert_eq!(array, [0, 2, 4, 6, 8, 10, 12, 14]);
    /// ```
    fn collect_array<const N: usize>(&mut self) -> [Self::Item; N] {
        self.collect_array_checked()
            .unwrap_or_else(|x| panic!("couldnt fill buffer of length {N} only had {x} elements"))
    }
    /// Non panicking version of [`collect_array`](CollectArray::collect_array).
    ///
    /// Lets you collect an iterator into a fixed length array with no vec allocation, with no panics.
    /// If the iterator returns [`None`] at any point, returns <code>[Err]\(elements filed\)</code>.
    ///
    /// If you wish to simply populate the array with [`None`] if the iterator returns [`None`], use [`items`](CollectArray::items).
    ///
    /// ```
    /// use collar::*;
    /// let array: Result<[u8; 10], usize> = std::iter::repeat(5).take(3).collect_array_checked();
    /// // does not fill array -> produces `Err`, with number of elements filled.
    /// assert_eq!(array, Err(3));
    /// ```
    fn collect_array_checked<const N: usize>(&mut self) -> Result<[Self::Item; N], usize> {
        try_from_fn(|elem| self.next().ok_or(elem))
    }

    /// Creates an array [T; N] where each fallible (i.e [`Option`] or [`Result`]) element is begotten from [`next`](Iterator::next).
    /// Unlike [`collect_array`](CollectArray::collect_array), where the element creation can't fail, this version will return an error if any element creation was unsuccessful (returned [`Err`] or [`None`]).
    /// In the case where the iterator ran out of elements, this returns a [`CollectorError`] containing the count.
    ///
    /// The return type of this function depends on the [`Item`](Iterator::Item) of this [`Iterator`].
    /// If you return `Result<T, E>` from the closure, you'll get a `Result<[T; N], CollectorError<E>>`.
    /// If you return `Option<T>` from the closure, you'll get an `Result<[T; N], CollectorError<()>>`.
    /// ```
    /// use collar::CollectArray;
    /// let array: Result<[i8; 200], _> = (0..).map(|x| x.try_into()).try_collect_array();
    /// assert_eq!(array.unwrap_err().at, 128);
    ///
    /// // note the ok(); the try trait is still unstable. (so this is a Result<_, ()>::ok)
    /// let array: Option<[_; 4]> = (0usize..).map(|i| i.checked_add(100)).try_collect_array().ok();
    /// assert_eq!(array, Some([100, 101, 102, 103]));
    ///
    /// let array: Option<[_; 4]> = (0usize..).map(|i| i.checked_sub(100)).try_collect_array().ok();
    /// assert_eq!(array, None);
    /// ```
    fn try_collect_array<const N: usize>(
        &mut self,
    ) -> Result<[<Self::Item as Maybe>::Unwrap; N], Error<N, <Self::Item as Maybe>::Or>>
    where
        Self::Item: Maybe,
    {
        try_from_fn(|elem| {
            self.next()
                // no error, ran out
                .ok_or(None)
                // some error, flattened
                .and_then(|x| x.asr().map_err(Some))
                .map_err(|x| Error { error: x, at: elem })
        })
    }

    /// This function fills an array with this iterators elements.
    /// It will always return (unless the iterator panics).
    /// ```
    /// use collar::*;
    /// assert_eq!(
    ///     (0..).items::<5>(),
    ///     (0..).map(Some).collect_array::<5>(),
    /// )
    /// ```
    fn items<const N: usize>(&mut self) -> [Option<Self::Item>; N] {
        from_fn(|_| self.next())
    }
}
impl<I: Iterator> CollectArray for I {}

struct OnDrop<F: FnOnce()> {
    f: MD<F>,
}
impl<F: FnOnce()> OnDrop<F> {
    fn guard(x: F) -> Self {
        Self { f: MD::new(x) }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        // SAFETY: `Drop::drop` is only called once.
        unsafe { MD::take(&mut self.f)() }
    }
}

const unsafe fn transmute_unchecked<T, U>(value: T) -> U {
    const { assert!(size_of::<T>() == size_of::<U>()) }
    #[repr(C)]
    union Transmute<T, U> {
        t: MD<T>,
        u: MD<U>,
    }
    unsafe { MD::into_inner(Transmute { t: MD::new(value) }.u) }
}

/// [`std::array::try_from_fn`] on stable.
///
/// Creates an array `[T; N]` where each fallible array element `T` is returned by the `cb` call.
/// Unlike [`from_fn`], where the element creation can't fail, this version will return an error
/// if any element creation was unsuccessful.
///
/// The return type of this function depends on the return type of the closure.
/// If you return `Result<T, E>` from the closure, you'll get a `Result<[T; N], E>`.
/// If you return `Option<T>` from the closure, you'll get an `Result<[T; N], ()>`.
///
/// # Arguments
///
/// * `cb`: Callback where the passed argument is the current array index.
///
/// # Example
///
/// ```rust
/// let array: Result<[u8; 5], _> = collar::try_from_fn(|i| i.try_into());
/// assert_eq!(array, Ok([0, 1, 2, 3, 4]));
///
/// let array: Result<[i8; 200], _> = collar::try_from_fn(|i| i.try_into());
/// assert!(array.is_err());
///
/// let array: Option<[_; 4]> = collar::try_from_fn(|i| i.checked_add(100)).ok();
/// assert_eq!(array, Some([100, 101, 102, 103]));
///
/// let array: Option<[_; 4]> = collar::try_from_fn(|i| i.checked_sub(100)).ok();
/// assert_eq!(array, None);
/// ```
pub fn try_from_fn<R: Maybe, const N: usize>(
    mut x: impl FnMut(usize) -> R,
) -> Result<[R::Unwrap; N], R::Or> {
    let mut out = [const { MU::uninit() }; N];
    // initialize each element of `out`
    for elem in 0..N {
        let guard = OnDrop::guard(|| unsafe {
            let p = &raw mut out[..elem] as *mut [R::Unwrap];
            let guard = OnDrop::guard(|| drop_in_place(p));
            drop_in_place(p);
            // dont drop! (again)
            forget(guard);
        });
        let e = x(elem).asr()?;
        // dont drop!
        forget(guard);
        out[elem] = MU::new(e);
    }
    // SAFETY: each element has been initialized
    Ok(unsafe { transmute_unchecked(out) })
}

#[doc(no_inline)]
pub use core::array::from_fn;
