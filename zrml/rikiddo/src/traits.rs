use crate::types::{EmaConfig, FeeSigmoidConfig, TimestampedVolume};
use frame_support::{
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::fmt::Debug;
use substrate_fixed::traits::{Fixed, FixedSigned, FixedUnsigned};

pub trait Sigmoid {
    type FIN: Fixed;
    type FOUT: Fixed;

    /// Calculate fee
    fn calculate(&self, r: Self::FIN) -> Result<Self::FOUT, &'static str>;
}

pub trait MarketAverage {
    type FU: FixedUnsigned;

    /// Get average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn get(&self) -> Option<Self::FU>;

    /// Clear market data
    fn clear(&mut self);

    /// Update market volume
    fn update(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;
}

pub trait Lmsr {
    type FU: FixedUnsigned;

    /// Return price P_i(q) for all assets in q
    fn all_prices(&self, asset_balances: &Vec<Self::FU>) -> Result<Vec<Self::FU>, &'static str>;

    /// Return cost C(q) for all assets in q
    fn cost(&self, asset_balances: &Vec<Self::FU>) -> Result<Self::FU, &'static str>;

    /// Return price P_i(q) for asset q_i in q
    fn price(
        &self,
        asset_in_question_balance: &Self::FU,
        asset_balances: &Vec<Self::FU>,
    ) -> Result<Self::FU, &'static str>;
}

pub trait RikiddoMV: Lmsr {
    /// Clear market data
    fn clear(&mut self);

    /// Update market data
    fn update(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;
}

pub trait RikiddoSigmoidMVPallet {
    type Balance: Parameter
        + Member
        + AtLeast32BitUnsigned
        + Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug;

    type FS: FixedSigned;
    type FU: FixedUnsigned;

    /// Clear market data for specific asset pool
    fn clear(poolid: u128);

    /// Return cost C(q) for all assets in q
    fn cost(
        poolid: u128,
        asset_balances: Vec<Self::Balance>,
    ) -> Result<Self::Balance, &'static str>;

    /// Create Rikiddo instance for specifc asset pool
    fn create(
        poolid: u128,
        fee_config: FeeSigmoidConfig<Self::FS, Self::FU>,
        ema_config_short: EmaConfig<Self::FU>,
        ema_config_long: EmaConfig<Self::FU>,
        balance_one_unit: Self::Balance,
    );

    /// Destroy Rikiddo instance
    fn destroy(poolid: u128);

    /// Return price P_i(q) for asset q_i in q
    fn price(
        poolid: u128,
        asset_in_question: Self::Balance,
        asset_balances: Vec<Self::Balance>,
    ) -> Result<Self::Balance, &'static str>;

    /// Return price P_i(q) for all assets in q
    fn all_prices(
        poolid: u128,
        asset_balances: Vec<Self::Balance>,
    ) -> Result<Vec<Self::Balance>, &'static str>;

    /// Update market data
    fn update(poolid: u128, volume: Self::Balance) -> Result<Option<Self::Balance>, &'static str>;
}
