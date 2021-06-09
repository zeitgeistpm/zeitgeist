use crate::types::{EmaVolumeConfig, FeeSigmoidConfig};
use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;
use substrate_fixed::traits::FixedUnsigned;

pub trait Fee<F: FixedUnsigned> {
    fn calculate() -> F;
}

pub trait MarketEma<F: FixedUnsigned> {
    // Update market volume
    fn update(volume: F);
    // Clear market data
    fn clear();
    // Calculate EMA of market volume
    fn calculate() -> Option<F>;
}

pub trait Lmsr<F: FixedUnsigned> {
    /// Return cost C(q) for all assets in q
    fn cost(asset_balances: Vec<F>) -> F;
    /// Return price P_i(q) for asset q_i in q
    fn price(asset_in_question_balance: F, asset_balances: Vec<F>) -> F;
    /// Return price P_i(q) for all assets in q
    fn all_prices(asset_balances: Vec<F>) -> Vec<F>;
}

pub trait Lsdlmsr<F: FixedUnsigned>: Lmsr<F> {
    type Fee: Fee<F>;

    /// Clear market data
    fn clear();
    /// Update market data
    fn update(volume: F);
}

pub trait LsdlmsrPallet {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    /// Return cost C(q) for all assets in q
    fn cost(asset_balances: Vec<Self::Balance>) -> Self::Balance;
    /// Create LSD-LMSR instance for specifc asset pool
    fn create(
        poolid: u128,
        fee_config: FeeSigmoidConfig,
        ema_config_short: EmaVolumeConfig,
        ema_config_long: EmaVolumeConfig,
        balance_fractional_bits: u8,
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
