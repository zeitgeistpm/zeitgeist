use crate::{
    constants::{INITIAL_FEE, M, MINIMAL_REVENUE, N, P},
    traits::Sigmoid,
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{FixedSigned, FixedUnsigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{
        extra::{U24, U32},
        I9F23,
    },
    FixedI32, FixedU32,
};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig<FS: FixedSigned, FU: FixedUnsigned> {
    pub m: FS,
    pub p: FS,
    pub n: FS,
    pub initial_fee: FU,
    pub min_revenue: FU,
}

impl<FS, FU> Default for FeeSigmoidConfig<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
{
    fn default() -> Self {
        Self {
            m: M.lossy_into(),
            p: P.lossy_into(),
            n: N.lossy_into(),
            initial_fee: INITIAL_FEE.lossy_into(),
            min_revenue: MINIMAL_REVENUE.lossy_into(),
        }
    }
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
{
    pub config: FeeSigmoidConfig<FS, FU>,
}

impl<FS, FU> FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
{
    pub fn new(config: FeeSigmoidConfig<FS, FU>) -> Self {
        Self { config }
    }
}

impl<FS, FU> Sigmoid for FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + PartialOrd<I9F23>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + PartialOrd<FS>
{
    type FIN = FS;
    type FOUT = FU;

    // z(r) in https://files.kyber.network/DMM-Feb21.pdf
    fn calculate(&self, r: Self::FIN) -> Result<Self::FOUT, &'static str> {
        let r_minus_n = if let Some(res) = r.checked_sub(self.config.n) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: r - n");
        };

        let numerator = if let Some(res) = r_minus_n.checked_mul(self.config.m) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: m * (r-n)");
        };

        let r_minus_n_squared = if let Some(res) = r_minus_n.checked_mul(r_minus_n) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: (r-n)^2");
        };

        let p_plus_r_minus_n_squared =
            if let Some(res) = self.config.p.checked_add(r_minus_n_squared) {
                res
            } else {
                return Err("[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
            };

        let denominator = sqrt::<FS, FS>(p_plus_r_minus_n_squared)?;

        let sigmoid_result = if let Some(res) = numerator.checked_div(denominator) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: numerator / denominator");
        };

        // TODO: Add min revenue comparison, and proper return max(min_revenue, sigmoid_result);
        // Also figure out if we want to return intial_fee here if r = 0
        Err("Unimplemented!")
    }
}
