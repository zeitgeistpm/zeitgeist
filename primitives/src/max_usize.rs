use core::ops::Deref;

/// The biggest pointer size taking into consideration all known targeting machine architectures.
///
/// Useful to cast `usize` into `MaxUsize` with the guarantee that no truncation will occur.
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    parity_scale_codec::CompactAs,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
)]
pub struct MaxUsize(u64);

impl AsRef<u64> for MaxUsize {
    #[inline]
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Deref for MaxUsize {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &u64 {
        &self.0
    }
}

// As per contract, `usize` will never be greater than `u64`.
impl From<usize> for MaxUsize {
    #[inline]
    fn from(n: usize) -> Self {
        Self(n as u64)
    }
}

macro_rules! impl_from {
    ($n:ty) => {
        impl From<$n> for MaxUsize {
            #[inline]
            fn from(n: $n) -> Self {
                Self(n.into())
            }
        }
    };
}

impl_from!(u8);
impl_from!(u16);
impl_from!(u32);
impl_from!(u64);
