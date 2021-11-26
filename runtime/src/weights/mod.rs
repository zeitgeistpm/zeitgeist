cfg_if::cfg_if!{
    if #[cfg(feature = "parachain")] {
        pub mod pallet_author_mapping;
        pub mod parachain_staking;
    } else {
        pub mod pallet_grandpa;
    }
}


pub mod frame_system;
pub mod orml_currencies;
pub mod orml_tokens;
pub mod pallet_balances;
pub mod pallet_collective;
pub mod pallet_identity;
pub mod pallet_membership;
pub mod pallet_timestamp;
pub mod pallet_treasury;
pub mod pallet_utility;
pub mod pallet_vesting;