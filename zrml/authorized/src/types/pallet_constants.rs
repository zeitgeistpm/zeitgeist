// Copyright 2024 Forecasting Technologies LTD.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use crate::Config;
use frame_support::ensure;
use sp_runtime::traits::Zero;

struct PalletConstants<T>
where
    T: Config,
{
    correction_period: T::BlockNumber,
}

pub(crate) struct SetPalletConstantsParams<T>
where
    T: Config,
{
    correction_period: Option<T::BlockNumber>,
}

enum SetPalletConstantsError {
    InvalidCorrectionPeriod,
}

impl<T> PalletConstants<T>
where
    T: Config,
{
    fn set(self, params: SetPalletConstantsParams<T>) -> Result<(), SetPalletConstantsError> {
        if let Some(correction_period) = params.correction_period {
            ensure!(!correction_period.is_zero(), SetPalletConstantsError::InvalidCorrectionPeriod);
            self.correction_period = correction_period;
        }
        Ok(())
    }
}
