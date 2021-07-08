use crate::{
    constants::{M, MINIMAL_REVENUE, N, P},
    traits::Sigmoid,
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{FixedI128, FixedI32, FixedU128, FixedU32, traits::{FixedSigned, FixedUnsigned, FromFixed, LossyFrom, LossyInto, ToFixed}, transcendental::sqrt, types::{
        extra::{U24, U32, U128},
        I9F23,
    }};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig<FS: FixedSigned, FU: FixedUnsigned> {
    pub m: FS,
    pub p: FS,
    pub n: FS,
    pub min_revenue: FU,
}

impl<FS, FU> Default for FeeSigmoidConfig<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    i128: From<FS::Bits>
{
    fn default() -> Self {
        Self {
            m: M.lossy_into(),
            p: P.lossy_into(),
            n: N.lossy_into(),
            min_revenue: MINIMAL_REVENUE.lossy_into(),
        }
    }
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    i128: From<FS::Bits>
{
    pub config: FeeSigmoidConfig<FS, FU>,
}

impl<FS, FU> FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>>,
    i128: From<FS::Bits>,
    u128: ToFixed
{
    pub fn new(config: FeeSigmoidConfig<FS, FU>) -> Self {
        Self { config }
    }
}

impl<FS, FU> Sigmoid for FeeSigmoid<FS, FU>
where
    FS: FixedSigned + LossyFrom<FixedI32<U24>> + PartialOrd<I9F23>,
    FU: FixedUnsigned + LossyFrom<FixedU32<U32>> + LossyFrom<FixedU128<U128>> + PartialOrd<FS>,
    i128: From<FS::Bits>,
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

        if self.config.min_revenue >= sigmoid_result {
            return Ok(self.config.min_revenue);
        }

        if Self::FOUT::max_value() < sigmoid_result.int() {
            return Err("[FeeSigmoid] Overflow during conversion: Result does not fit in specified output type");
        }

        let integer_part_signed = i128::from_fixed(sigmoid_result.int());
        // We can safely cast because until here we know that the integer part is unsigned.
        let integer_part: Self::FOUT = (integer_part_signed as u128).to_fixed();
        let fractional_part: FixedU128<U128> = sigmoid_result.frac().to_fixed();

        if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
            return Ok(res);
        } else {
            // This error should be impossible to reach.
            return Err("[FeeSigmoid] Something went wrong during FIN to FOUT type conversion.")
        };
    }
}
