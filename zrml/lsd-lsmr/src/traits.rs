use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned, Hash};
use sp_std::fmt::Debug;

// TODO: Add parameters, return values and docs
pub trait Fee {
    fn calculate();
}

// TODO: Add parameters, return values and docs
pub trait Ema {
    // timestamp type
    type Moment: AtLeast32Bit + Parameter + Default + Copy;

    fn update();
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

    // TODO: Add parameters and return values
    // TODO: Move out of trait and into pallet implementation
    /// Create LSD-LSMR instance for specifc asset pair
    fn create();
    /// Destroy
    fn destroy();
    /// Update market data
    fn update();
    /// Return cost for asset pair
    fn cost();
    /// Return price for asset pair
    fn price();
}
