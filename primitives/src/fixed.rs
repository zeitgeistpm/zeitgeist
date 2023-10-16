use crate::{
    checked_ops_res::{CheckedAddRes, CheckedDivRes, CheckedMulRes, CheckedSubRes},
    constants::BASE,
};
use core::marker::PhantomData;
use frame_support::dispatch::DispatchError;
use sp_arithmetic::traits::AtLeast32BitUnsigned;

pub trait BaseProvider<T> {
    fn get() -> Result<T, DispatchError>;
}

pub struct ZeitgeistBase<T>(PhantomData<T>);

// Used to avoid saturating operations.
impl<T> BaseProvider<T> for ZeitgeistBase<T>
where
    T: AtLeast32BitUnsigned,
{
    fn get() -> Result<T, DispatchError> {
        BASE.try_into()
            .map_err(|_| DispatchError::Other("ZeitgeistBase failed to convert BASE to Balance"))
    }
}

pub trait FixedMul
where
    Self: Sized,
{
    fn bmul(&self, other: Self) -> Result<Self, DispatchError>;

    fn bmul_floor(&self, other: Self) -> Result<Self, DispatchError>;

    fn bmul_ceil(&self, other: Self) -> Result<Self, DispatchError>;
}

pub trait FixedDiv
where
    Self: Sized,
{
    fn bdiv(&self, other: Self) -> Result<Self, DispatchError>;
    fn bdiv_floor(&self, other: Self) -> Result<Self, DispatchError>;
    fn bdiv_ceil(&self, other: Self) -> Result<Self, DispatchError>;
}

impl<T> FixedMul for T
where
    T: AtLeast32BitUnsigned,
{
    fn bmul(&self, other: Self) -> Result<Self, DispatchError> {
        let c0 = self.checked_mul_res(&other)?;
        let c1 = c0.checked_add_res(&ZeitgeistBase::<T>::get()?.checked_div_res(&2u8.into())?)?;
        c1.checked_div_res(&ZeitgeistBase::get()?)
    }

    fn bmul_floor(&self, other: Self) -> Result<Self, DispatchError> {
        self.bmul(other) // TODO
    }

    fn bmul_ceil(&self, other: Self) -> Result<Self, DispatchError> {
        self.bmul(other) // TODO
    }
}

impl<T> FixedDiv for T
where
    T: AtLeast32BitUnsigned,
{
    fn bdiv(&self, other: Self) -> Result<Self, DispatchError> {
        let c0 = self.checked_mul_res(&ZeitgeistBase::get()?)?;
        let c1 = c0.checked_add_res(&other.checked_div_res(&2u8.into())?)?;
        c1.checked_div_res(&other)
    }

    fn bdiv_floor(&self, other: Self) -> Result<Self, DispatchError> {
        self.bdiv(other) // TODO
    }

    fn bdiv_ceil(&self, other: Self) -> Result<Self, DispatchError> {
        self.bdiv(other) // TODO
    }
}
