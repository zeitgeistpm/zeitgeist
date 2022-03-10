use core::ops::Deref;

/// The biggest pointer size taking into consideration all known targeting machine
/// architectures. Useful to cast `usize` into `MaxRuntimeUsize` with the guarantee that no
/// truncation will occur.
///
/// As stated by the name, this struct is only valid in a runtime environment.
#[derive(
    parity_scale_codec::CompactAs,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    scale_info::TypeInfo,
    Clone,
    Debug,
    Eq,
    PartialEq,
)]
pub struct MaxRuntimeUsize(u64);

impl AsRef<u64> for MaxRuntimeUsize {
    #[inline]
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Deref for MaxRuntimeUsize {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &u64 {
        &self.0
    }
}

// As per contract, `usize` will never be greater than `u64`.
impl From<usize> for MaxRuntimeUsize {
    #[inline]
    fn from(from: usize) -> Self {
        Self(from as _)
    }
}

// As per contract, `usize` will never be greater than `u64`.
impl From<MaxRuntimeUsize> for usize {
    #[inline]
    fn from(from: MaxRuntimeUsize) -> Self {
        from.0 as _
    }
}

macro_rules! impl_from_primitive {
    ($n:ty) => {
        impl From<$n> for MaxRuntimeUsize {
            #[inline]
            fn from(from: $n) -> Self {
                Self(from.into())
            }
        }
    };
}

impl_from_primitive!(u8);
impl_from_primitive!(u16);
impl_from_primitive!(u32);
impl_from_primitive!(u64);

macro_rules! impl_to_primitive {
    ($n:ty) => {
        impl From<MaxRuntimeUsize> for $n {
            #[inline]
            fn from(from: MaxRuntimeUsize) -> Self {
                from.0.into()
            }
        }
    };
}

impl_to_primitive!(u64);
impl_to_primitive!(u128);
