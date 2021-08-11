use crate::{
    constants::{INITIAL_FEE, M, MINIMAL_REVENUE, N, P},
    traits::Sigmoid,
};
use frame_support::dispatch::{Decode, Encode};
use sp_core::RuntimeDebug;
use substrate_fixed::{
    traits::{FixedSigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{
        extra::{U127, U24, U32},
        I9F23,
    },
    FixedI128, FixedI32, FixedU32,
};

use super::convert_to_signed;

#[derive(Clone, RuntimeDebug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig<FS: FixedSigned> {
    pub m: FS,
    pub p: FS,
    pub n: FS,
    pub initial_fee: FS,
    pub min_revenue: FS,
}

impl<FS> Default for FeeSigmoidConfig<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    fn default() -> Self {
        Self {
            m: M.lossy_into(),
            p: P.lossy_into(),
            n: N.lossy_into(),
            // Only case this can panic is, when INITIAL_FEE is >= 1.0 and FS integer bits < 2
            initial_fee: convert_to_signed::<FixedU32<U32>, FS>(INITIAL_FEE.lossy_into()).unwrap(),
            // Only case this can panic is, when MIN_REVENUE is >= 1.0 and FS integer bits < 2
            min_revenue: convert_to_signed::<FixedU32<U32>, FS>(MINIMAL_REVENUE.lossy_into())
                .unwrap(),
        }
    }
}

#[derive(Clone, RuntimeDebug, Decode, Default, Encode, Eq, PartialEq)]
pub struct FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    pub config: FeeSigmoidConfig<FS>,
}

impl<FS> FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI128<U127>>,
{
    pub fn new(config: FeeSigmoidConfig<FS>) -> Self {
        Self { config }
    }
}

impl<FS> Sigmoid for FeeSigmoid<FS>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + PartialOrd<I9F23> + LossyFrom<FixedI128<U127>>,
{
    type FS = FS;

    // z(r) in https://files.kyber.network/DMM-Feb21.pdf
    fn calculate_fee(&self, r: Self::FS) -> Result<Self::FS, &'static str> {
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

        let result = if let Some(res) = sigmoid_result.checked_add(self.config.initial_fee) {
            res
        } else {
            return Err("[FeeSigmoid] initial_fee + sigmoid_result");
        };

        if self.config.min_revenue >= result {
            return Ok(self.config.min_revenue);
        }

        Ok(result)
    }
}
