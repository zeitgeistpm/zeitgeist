// Copyright 2025 Forecasting Technologies LTD.
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

mod collection_id_error;
pub(crate) mod cryptographic_id_manager;
pub(crate) mod hash;
mod transmutation_type;

pub use collection_id_error::CollectionIdError;
pub use cryptographic_id_manager::{CryptographicIdManager, Fuel};
pub(crate) use hash::Hash256;
pub use transmutation_type::TransmutationType;
