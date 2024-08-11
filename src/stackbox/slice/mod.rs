use super::*;

pub
mod iter;

impl<'frame, Item : 'frame> Default for StackBox<'frame, [Item; 0]> {
    fn default()
      -> Self
    {
        unsafe {
            // Safety: empty slice.
            StackBox::assume_owns_all(&mut [])
        }
            .try_into()
            .unwrap()
    }
}

impl<'frame, Item : 'frame> Default for StackBox<'frame, [Item]> {
    fn default()
      -> Self
    {
        StackBox::<[_; 0]>::default()
            .into_slice()
    }
}

impl<'frame, Item : 'frame> StackBox<'frame, [Item]> {
    /// # Safety
    ///
    /// Same requirements as [`StackBox::assume_owns`].
    #[inline]
    unsafe
    fn assume_owns_all (
        slice: &'frame mut [ManuallyDrop<Item>]
    ) -> StackBox<'frame, [Item]>
    {
        let fat_ptr: *mut [ManuallyDrop<Item>] = slice;
        let fat_ptr: *mut ManuallyDrop<[Item]> = fat_ptr as _;
        let slice: &'frame mut ManuallyDrop<[Item]> = &mut *fat_ptr;
        StackBox::assume_owns(slice)
    }

    /// [`Vec`]-like behavior for [`StackBox`]: pop its first item.
    pub
    fn stackbox_pop (self: &'_ mut StackBox<'frame, [Item]>)
      -> Option<Item>
    {
        if self.is_empty() {
            return None;
        }
        let placeholder = unsafe {
            // Safety: empty slice.
            StackBox::assume_owns_all(&mut [])
        };
        let this = ::core::mem::replace(self, placeholder);
        let (hd, tl) = this.stackbox_split_at(1);
        *self = tl;
        Some(unsafe {
            ::core::ptr::read(&ManuallyDrop::new(hd)[0])
        })
    }

    /// [`StackBox`] / owned equivalent of the `slice` splitting methods.
    #[inline]
    pub
    fn stackbox_split_at (self: StackBox<'frame, [Item]>, mid: usize)
      -> (
            StackBox<'frame, [Item]>,
            StackBox<'frame, [Item]>,
        )
    {
        assert!(mid <= self.len()); // before the MD
        let mut this = ::core::mem::ManuallyDrop::new(self);
        let (hd, tl): (&'_ mut [Item], &'_ mut [Item]) =
            this.split_at_mut(mid)
        ;
        unsafe {
            // Safety: recovering back the ownership initially yielded.
            (
                Self::assume_owns_all(
                    ::core::slice::from_raw_parts_mut(
                        hd.as_mut_ptr().cast(),
                        hd.len(),
                    )
                ),
                Self::assume_owns_all(
                    ::core::slice::from_raw_parts_mut(
                        tl.as_mut_ptr().cast(),
                        tl.len(),
                    )
                ),
            )
        }
    }
}

impl<'frame, Item : 'frame, const N: usize> StackBox<'frame, [Item; N]> {
    /// Coerces a `StackBox<[T; N]>` into a `StackBox<[T]>`.
    ///
    ///   - Note that you may not need to use `.into_slice()` if instead of
    ///     [`StackBox::new_in`] you use [`stackbox!`] to construct it:
    ///
    ///     ```rust
    ///     use ::stackbox::prelude::*;
    ///
    ///     mk_slots!(slot);
    ///     //      boxed_slice: StackBox<'_, [String]> = StackBox::new_in(slot, [
    ///     let mut boxed_slice: StackBox<'_, [String]> = stackbox!(slot, [
    ///         "Hello, World!".into()
    ///     ]);
    ///     let _: String = boxed_slice.stackbox_pop().unwrap();
    ///     ```
    #[inline]
    pub
    fn into_slice(self: StackBox<'frame, [Item; N]>)
      -> StackBox<'frame, [Item]>
    {
        unsafe {
            let ptr: ptr::NonNull<[Item; N]> =
                <*const _>::read(
                    &::core::mem::ManuallyDrop::new(self).unique_ptr
                )
                    .into_raw_nonnull()
            ;
            let ptr: ptr::NonNull<[Item]> = ptr;
            StackBox {
                unique_ptr: ptr::Unique::from_raw(ptr.as_ptr()),
                _covariant_lt: <_>::default(),
            }
        }
    }
}

impl<'frame, Item : 'frame> StackBox<'frame, [Item; 1]> {
    /// Convert a [`StackBox`] 1-array into a [`Stackbox`] of its single item.
    #[inline]
    pub
    fn stackbox_unwrap_1_array(self: StackBox<'frame, [Item; 1]>)
      -> StackBox<'frame, Item>
    {
        unsafe {
            // Safety: same layout, validity and safety invariants.
            ::core::mem::transmute(self)
        }
    }
}

impl<'frame, Item : 'frame, const N: usize>
    ::core::convert::TryFrom<StackBox<'frame, [Item]>>
for
    StackBox<'frame, [Item; N]>
{
    type Error = TryFromSliceError<StackBox<'frame, [Item]>>;

    #[inline]
    fn try_from(
        stackbox: StackBox<'frame, [Item]>,
    ) -> Result<StackBox<'frame, [Item; N]>, Self::Error>
    {
        if stackbox.len() != N {
            return Err(TryFromSliceError(stackbox));
        }
        Ok(unsafe {
            let wide_ptr: *mut [Item] =
                <*const _>::read(
                    &::core::mem::ManuallyDrop::new(stackbox).unique_ptr
                )
                    .into_raw_nonnull()
                    .as_ptr()
            ;
            let thin_ptr: *mut [Item; N] = wide_ptr as _;
            StackBox {
                unique_ptr: ptr::Unique::from_raw(thin_ptr),
                _covariant_lt: <_>::default(),
            }
        })
    }
}

#[non_exhaustive]
pub
struct TryFromSliceError<T>(
    pub T,
    // non_exhaustive,
);

impl<T> ::core::fmt::Display for TryFromSliceError<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>)
      -> ::core::fmt::Result
    {
        "could not convert slice to array".fmt(f)
    }
}

impl<T> ::core::fmt::Debug for TryFromSliceError<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>)
      -> ::core::fmt::Result
    {
        f.debug_struct("TryFromSliceError").finish_non_exhaustive()
    }
}

#[cfg(feature = "std")]
impl<T> ::std::error::Error for TryFromSliceError<T> {}
