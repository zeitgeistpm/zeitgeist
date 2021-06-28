use crate::types::{EmaVolumeConfig, FeeSigmoidConfig};
use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;
use substrate_fixed::traits::{Fixed, FixedUnsigned};

pub trait LsdlmsrFee<F: Fixed> {
    type Output: Fixed;

    /// Calculate fee
    fn calculate(&self, r: F) -> Result<Self::Output, &'static str>;
}

pub trait MarketAverage<F: FixedUnsigned> {
    /// Update market volume
    fn update(&self, volume: F);

    /// Clear market data
    fn clear(&self);

    /// Calculate average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn calculate(&self) -> Option<F>;
}

pub trait Lmsr<F: FixedUnsigned> {
    /// Return cost C(q) for all assets in q
    fn cost(asset_balances: Vec<F>) -> F;

    /// Return price P_i(q) for asset q_i in q
    fn price(asset_in_question_balance: F, asset_balances: Vec<F>) -> F;

    /// Return price P_i(q) for all assets in q
    fn all_prices(asset_balances: Vec<F>) -> Vec<F>;
}

pub trait LsdlmsrMV<F: FixedUnsigned>: Lmsr<F> {
    /// Clear market data
    fn clear();

    /// Update market data
    fn update(volume: F);
}

pub trait LsdlmsrSigmoidMVPallet {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    /// Clear market data for specific asset pool
    fn clear(poolid: u128);

    /// Return cost C(q) for all assets in q
    fn cost(poolid: u128, asset_balances: Vec<Self::Balance>) -> Self::Balance;

    /// Create LSD-LMSR instance for specifc asset pool
    fn create(
        poolid: u128,
        fee_config: FeeSigmoidConfig,
        ema_config_short: EmaVolumeConfig,
        ema_config_long: EmaVolumeConfig,
        balance_one_unit: Self::Balance,
    );

    /// Destroy Lsdlmsr instance
    fn destroy(poolid: u128);

    /// Return price P_i(q) for asset q_i in q
    fn price(
        poolid: u128,
        asset_in_question: Self::Balance,
        asset_balances: Vec<Self::Balance>,
    ) -> Self::Balance;

    /// Return price P_i(q) for all assets in q
    fn all_prices(poolid: u128, asset_balances: Vec<Self::Balance>) -> Vec<Self::Balance>;

    /// Update market data
    fn update(poolid: u128, volume: Self::Balance);
}
