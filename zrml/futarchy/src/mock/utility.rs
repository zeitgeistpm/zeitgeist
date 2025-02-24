// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::mock::runtime::{Balances, Futarchy, System};
use frame_support::traits::Hooks;
use zeitgeist_primitives::types::BlockNumber;

pub fn run_to_block(to: BlockNumber) {
    while System::block_number() < to {
        let now = System::block_number();

        Futarchy::on_finalize(now);
        Balances::on_finalize(now);
        System::on_finalize(now);

        let next = now + 1;
        System::set_block_number(next);

        System::on_initialize(next);
        Balances::on_initialize(next);
        Futarchy::on_initialize(next);
    }
}
