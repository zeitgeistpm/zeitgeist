use crate::types::AssetPair;
use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::{AtLeast32BitUnsigned, Hash};
use sp_std::fmt::Debug;

// TODO: Add parameters, return values and docs
pub trait Fee {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    fn calculate() -> Self::Balance;
}

// TODO: Add parameters, return values and docs
pub trait Ema {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    fn update(volume: Self::Balance);
    fn clear();
    fn calculate();
}

// TODO: return types, parameters and docs
pub trait LSMR {
    type Asset: Eq + Hash + PartialEq;
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    /// Return cost for asset pair
    fn cost();
    /// Return price for asset pair
    fn price();
}

// TODO: return types, parameters and docs
pub trait LsdLsmrPallet {
    type Asset: Eq + Hash + PartialEq;
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;
    type Fee: Fee;

    // TODO: Add parameters and return values
    /// Create LSD-LSMR instance for specifc asset pair
    fn create(assets: AssetPair<Self::Asset>, fees: Self::Fee);
    /// Destroy
    fn destroy(assets: AssetPair<Self::Asset>);
    /// Update market data
    fn update(assets: AssetPair<Self::Asset>, volume: Self::Balance);
    /// Return cost for asset pair
    fn cost();
    /// Return price for asset pair
    fn price();
}
