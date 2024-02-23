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

use parity_scale_codec::{HasCompact, MaxEncodedLen};

/// API that provides a signal on each market state transition
#[impl_trait_for_tuples::impl_for_tuples(8)]
pub trait MarketTransitionApi<MI: HasCompact + MaxEncodedLen> {
    fn on_proposal(_market_id: &MI) {}
    fn on_activation(_market_id: &MI) {}
    fn on_closure(_market_id: &MI) {}
    fn on_report(_market_id: &MI) {}
    fn on_dispute(_market_id: &MI) {}
    fn on_resolution(_market_id: &MI) {}
}
