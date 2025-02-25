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

#![cfg(test)]

use super::*;

mod decompress_collection_id;
mod decompress_hash;
mod get_collection_id;
mod matching_y_coordinate;
mod pow_magic_number;

trait FromHexStr {
    fn from_hex_str(hex_str: &str) -> Self
    where
        Self: Sized;
}

impl FromHexStr for Fq {
    fn from_hex_str(hex_str: &str) -> Fq {
        let hex_str_sans_prefix = &hex_str[2..];

        // Pad with zeroes on the left.
        let hex_str_padded = format!("{:0>64}", hex_str_sans_prefix);

        let bytes: Vec<u8> = (0..hex_str_padded.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex_str_padded[i..i + 2], 16).unwrap())
            .collect();

        let fixed_bytes: [u8; 32] = bytes.try_into().unwrap();

        Fq::from_be_bytes_mod_order(&fixed_bytes)
    }
}
