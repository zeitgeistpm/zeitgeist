use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::{AtLeast32BitUnsigned, Hash};
use sp_std::fmt::Debug;

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

    // TODO: Add parameters and return values
    /// Create LSD-LSMR instance for specifc asset pair
    fn create();
    /// Return cost for asset pair
    fn cost();
    /// Destroy LSD-LSMR instance for specific asset pair
    fn destroy();
    /// Return price for asset pair
    fn price();
    /// Update volume for asset pair
    fn update();
}
