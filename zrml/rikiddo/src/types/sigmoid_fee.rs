use super::Timespan;

use crate::{
    constants::{M, N, P},
    traits::RikiddoFee,
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{
        extra::{U24, U32},
        I9F23,
    },
    FixedI32,
};

impl Timespan {
    pub fn into_seconds(&self) -> u32 {
        match *self {
            Timespan::Seconds(d) => d,
            Timespan::Minutes(d) => d * 60,
            Timespan::Hours(d) => d * 60 * 60,
            Timespan::Days(d) => u32::from(d) * 60 * 60 * 24,
            Timespan::Weeks(d) => u32::from(d) * 60 * 60 * 24 * 7,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig<F: Fixed> {
    pub m: F,
    pub p: F,
    pub n: F,
}

impl<F: Fixed + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI32<U32>>> Default
    for FeeSigmoidConfig<F>
{
    fn default() -> Self {
        // To avoid a limitation of the generics, the values are hardcoded
        // instead of being fetched from constants.
        Self { m: M.lossy_into(), p: P.lossy_into(), n: N.lossy_into() }
    }
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct FeeSigmoid<FI: Fixed + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI32<U32>>> {
    pub config: FeeSigmoidConfig<FI>,
}

impl<F> RikiddoFee<F> for FeeSigmoid<F>
where
    F: FixedSigned + LossyFrom<FixedI32<U24>> + LossyFrom<FixedI32<U32>> + PartialOrd<I9F23>,
{
    // z(r) in https://files.kyber.network/DMM-Feb21.pdf
    fn calculate(&self, r: F) -> Result<F, &'static str> {
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

        let denominator = sqrt::<F, F>(p_plus_r_minus_n_squared)?;

        let _ = if let Some(res) = numerator.checked_div(denominator) {
            return Ok(res);
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: numerator / denominator");
        };
    }
}
