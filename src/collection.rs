use crate::Error;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// A vector of at most `N` values stored inline, without allocation.
///
/// It is the collection type available without any feature: a field of type `Values<T, N>`
/// collects the repeated values of an argument, and parsing fails with
/// [`Error::TooManyValues`] when more than `N` values are given.
/// It dereferences to a slice, so every slice method is available.
///
/// ```
/// use getargs_derive::Values;
///
/// let mut values: Values<u8, 2> = Values::new();
/// assert_eq!(values.push(1), Ok(()));
/// assert_eq!(values.push(2), Ok(()));
/// assert_eq!(values.push(3), Err(3));
/// assert_eq!(values, [1, 2]);
/// assert_eq!(values.iter().sum::<u8>(), 3);
/// ```
pub struct Values<T, const N: usize> {
    items: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Values<T, N> {
    /// Creates an empty vector.
    pub const fn new() -> Self {
        Values {
            items: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// The maximum number of values, `N`.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Appends a value, or gives it back when the vector already holds `N` values.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        self.items[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    /// The values, in insertion order.
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.items.as_ptr().cast::<T>(), self.len) }
    }

    /// The values, in insertion order, mutably.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.items.as_mut_ptr().cast::<T>(), self.len) }
    }
}

impl<T, const N: usize> Drop for Values<T, N> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}

impl<T, const N: usize> Default for Values<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for Values<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for Values<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Clone, const N: usize> Clone for Values<T, N> {
    fn clone(&self) -> Self {
        let mut clone = Self::new();
        for value in self.iter() {
            let _ = clone.push(value.clone());
        }
        clone
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for Values<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq<U>, U, const N: usize, const M: usize> PartialEq<Values<U, M>> for Values<T, N> {
    fn eq(&self, other: &Values<U, M>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for Values<T, N> {}

impl<T: PartialEq<U>, U, const N: usize> PartialEq<[U]> for Values<T, N> {
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq<U>, U, const N: usize> PartialEq<&[U]> for Values<T, N> {
    fn eq(&self, other: &&[U]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: PartialEq<U>, U, const N: usize, const M: usize> PartialEq<[U; M]> for Values<T, N> {
    fn eq(&self, other: &[U; M]) -> bool {
        self.as_slice() == other
    }
}

impl<T: Hash, const N: usize> Hash for Values<T, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<'v, T, const N: usize> IntoIterator for &'v Values<T, N> {
    type Item = &'v T;
    type IntoIter = core::slice::Iter<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'v, T, const N: usize> IntoIterator for &'v mut Values<T, N> {
    type Item = &'v mut T;
    type IntoIter = core::slice::IterMut<'v, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot collect repeated argument values",
    label = "unsupported collection type",
    note = "use `getargs_derive::Values<T, N>`, `heapless::Vec<T, N>` with the `heapless` feature of `getargs-derive`, or `Vec<T>` with its `alloc` feature"
)]
pub trait Collection: Sized {
    type Item;

    fn empty() -> Self;

    fn count(&self) -> usize;

    fn limit(&self) -> usize;

    fn append(&mut self, item: Self::Item);
}

#[diagnostic::do_not_recommend]
impl<T, const N: usize> Collection for Values<T, N> {
    type Item = T;

    fn empty() -> Self {
        Self::new()
    }

    fn count(&self) -> usize {
        self.len
    }

    fn limit(&self) -> usize {
        N
    }

    fn append(&mut self, item: T) {
        let _ = self.push(item);
    }
}

#[cfg(feature = "heapless")]
#[diagnostic::do_not_recommend]
impl<T, const N: usize, L: heapless::LenType> Collection for heapless::Vec<T, N, L> {
    type Item = T;

    fn empty() -> Self {
        Self::new()
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn limit(&self) -> usize {
        self.capacity()
    }

    fn append(&mut self, item: T) {
        let _ = self.push(item);
    }
}

#[cfg(feature = "alloc")]
#[diagnostic::do_not_recommend]
impl<T> Collection for alloc::vec::Vec<T> {
    type Item = T;

    fn empty() -> Self {
        Self::new()
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn limit(&self) -> usize {
        usize::MAX
    }

    fn append(&mut self, item: T) {
        self.push(item);
    }
}

fn append<'a, C: Collection>(
    collection: &mut C,
    value: &'a str,
    name: &'static str,
    max: usize,
    convert: &mut impl FnMut(&'a str) -> Result<C::Item, Error<'a>>,
) -> Result<(), Error<'a>> {
    if collection.count() >= max {
        return Err(Error::TooManyValues { name, value, max });
    }
    let item = convert(value)?;
    collection.append(item);
    Ok(())
}

#[doc(hidden)]
pub fn push_values<'a, C: Collection>(
    slot: &mut Option<C>,
    value: &'a str,
    delimiter: Option<char>,
    name: &'static str,
    max: usize,
    mut convert: impl FnMut(&'a str) -> Result<C::Item, Error<'a>>,
) -> Result<(), Error<'a>> {
    let collection = slot.get_or_insert_with(C::empty);
    let max = max.min(collection.limit());
    match delimiter {
        Some(delimiter) => value
            .split(delimiter)
            .try_for_each(|piece| append(collection, piece, name, max, &mut convert)),
        None => append(collection, value, name, max, &mut convert),
    }
}

#[doc(hidden)]
pub fn values_len<C: Collection>(slot: &Option<C>) -> usize {
    slot.as_ref().map_or(0, C::count)
}

#[doc(hidden)]
pub fn finish_values<'a, C: Collection>(
    slot: Option<C>,
    name: &'static str,
    min: usize,
) -> Result<C, Error<'a>> {
    let collection = slot.unwrap_or_else(C::empty);
    let actual = collection.count();
    if actual >= min {
        Ok(collection)
    } else if actual == 0 {
        Err(Error::MissingArgument(name))
    } else {
        Err(Error::TooFewValues { name, min, actual })
    }
}
