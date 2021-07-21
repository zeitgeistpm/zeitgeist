use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use sp_runtime::traits::AtLeast32Bit;

pub trait MarketId:
    AtLeast32Bit + Copy + Default + MaybeSerializeDeserialize + Member + Parameter
{
}

impl<T> MarketId for T where
    T: AtLeast32Bit + Copy + Default + MaybeSerializeDeserialize + Member + Parameter
{
}
