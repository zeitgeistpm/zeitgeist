use frame_support::Parameter;
use sp_runtime::traits::{AtLeast32Bit, Hash};
use substrate_fixed::{types::extra::U32, types::extra::U24, FixedU32, FixedU64};

// TODO: derive essential trait implementations
// TODO: implement default trait
// TODO: docs


pub struct AssetPair<A: Eq + Hash + PartialEq> {
    pub asset1: A,
    pub asset2: A,
}

// TODO: docs 
pub struct FeeConfig {
    pub initial_fee: FixedU32<U32>,
    pub minimal_revenue: FixedU32<U32>,
    pub m: FixedU64<U32>,
    pub p: FixedU64<U32>,
    pub n: FixedU64<U32>,
}

// TODO: docs
pub struct EmaConfig<M: AtLeast32Bit + Parameter + Default + Copy> {
    pub ema_long: M,
    pub ema_short: M,
    pub smoothing: FixedU32<U24>,
}

// TODO: docs
pub struct LsdLsmrConfig<A, M>
    where
        A: Eq + Hash + PartialEq,
        M: AtLeast32Bit + Parameter + Default + Copy
{
    pub assets: AssetPair<A>,
    pub fees: FeeConfig,
    pub indicators: EmaConfig<M>,
}