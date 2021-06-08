use crate::types::{EmaVolumeConfig, FeeSigmoidConfig, Timestamp};
use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::{Codec, Decode, Encode, EncodeLike};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;
use substrate_fixed::traits::FixedUnsigned;

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
    fn calculate() -> Option<Timestamp>;
}

// TODO: return types, parameters and docs
pub trait Lmsr<F: FixedUnsigned> {
    /// Return cost C(q) for all assets in q
    fn cost(assets: Vec<F>);
    /// Return price P_i(q) for asset q_i in q
    fn price(asset_in_question: F, asset_balances: Vec<F>);
}

// TODO: return types, parameters and docs
pub trait Lsdlmsr<F: FixedUnsigned>: Lmsr<F> {
    type Asset: Decode + Encode + EncodeLike + Eq + PartialEq;
    type Fee: Fee;

    // TODO: Add parameters and return values
    /// Create LSD-LMSR instance for specifc asset pool
    fn create(assets: Vec<Self::Asset>, fees: Self::Fee);
    /// Destroy
    fn destroy();
    /// Update market data
    fn update(asset: Self::Asset, volume: F);
}

pub trait LsdlmsrPallet {
    type Asset: Decode + Encode + EncodeLike + Eq + PartialEq;
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    /// Return cost C(q) for all assets in q
    fn cost(assets: Vec<Self::Balance>) -> Self::Balance;
    /// Create LSD-LMSR instance for specifc asset pool
    fn create(
        poolid: u128,
        assets: Vec<Self::Asset>,
        fee_config: FeeSigmoidConfig,
        ema_config_short: EmaVolumeConfig,
        ema_config_long: EmaVolumeConfig,
    );
    /// Destroy Lsdlmsr instance
    fn destroy(poolid: u128);
    /// Return price P_i(q) for asset q_i in q
    fn price(asset_in_question: Self::Balance, asset_balances: Vec<Self::Balance>)
        -> Self::Balance;

    /// Return price P_i(q) for asset q_i in q
    fn all_prices(asset_balances: Vec<Self::Balance>) -> Vec<Self::Balance>;
    /// Update market data
    fn update(poolid: u128, asset: Self::Asset, volume: Self::Balance);
}
