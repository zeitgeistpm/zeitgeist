#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
    log,
    traits::{Get, Randomness},
};
use parity_scale_codec::Encode;
use sp_runtime::traits::Hash;

#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
)]
pub struct RandomnessData<BlockNumber> {
    pub current_block_randomness: Option<([u8; 32], BlockNumber)>,
    pub one_epoch_ago_randomness: Option<([u8; 32], BlockNumber)>,
    pub two_epochs_ago_randomness: Option<([u8; 32], BlockNumber)>,
}

#[frame_support::pallet]
pub mod pallet {
    use cumulus_primitives_core::ParaId;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + cumulus_pallet_parachain_system::Config {
        type SelfParaId: Get<ParaId>;
    }

    pub type RandomnessOf<T> = crate::RandomnessData<<T as frame_system::Config>::BlockNumber>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type RandomnessSource<T> = StorageValue<_, RandomnessOf<T>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {}
}

pub struct CustomSystemEventHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> cumulus_pallet_parachain_system::OnSystemEvent for CustomSystemEventHandler<T> {
    fn on_validation_data(data: &cumulus_primitives_core::PersistedValidationData) {
        let relay_chain_state = if let Some(relay_chain_state) =
            cumulus_pallet_parachain_system::Pallet::<T>::relay_state_proof()
        {
            relay_chain_state
        } else {
            log::warn!(
                "Relaychain Randomness: No relay chain state proof available (set in \
                 `set_validation_data`)"
            );
            return;
        };

        let proof = if let Some(proof) = cumulus_pallet_parachain_system::RelayChainStateProof::new(
            <T as pallet::Config>::SelfParaId::get(),
            data.relay_parent_storage_root,
            relay_chain_state,
        )
        .ok()
        {
            proof
        } else {
            log::warn!("Relaychain Randomness: Invalid relay chain proof.");
            return;
        };

        let block_number = <frame_system::Pallet<T>>::block_number();

        let mut randomness_source = <RandomnessSource<T>>::get();
        let mut is_changed = false;

        use cumulus_primitives_core::relay_chain::well_known_keys as relay_well_known_keys;
        // https://github.com/paritytech/substrate/blob/5abf6c8a015fac28d33967800da7c5c8d53002e3/frame/babe/src/randomness.rs#L27-L121
        let fallback: Option<[u8; 32]> = None;
        let current_block_randomness: Option<[u8; 32]> = proof
            .read_entry::<sp_consensus_vrf::schnorrkel::Randomness>(
                relay_well_known_keys::CURRENT_BLOCK_RANDOMNESS,
                fallback,
            )
            .ok();
        if let Some(r) = current_block_randomness {
            randomness_source.current_block_randomness = Some((r, block_number));
            is_changed = true;
        }

        let one_epoch_ago_randomness: Option<[u8; 32]> = proof
            .read_entry::<sp_consensus_vrf::schnorrkel::Randomness>(
                relay_well_known_keys::ONE_EPOCH_AGO_RANDOMNESS,
                fallback,
            )
            .ok();
        if let Some(r) = one_epoch_ago_randomness {
            randomness_source.one_epoch_ago_randomness = Some((r, block_number));
            is_changed = true;
        }

        let two_epochs_ago_randomness: Option<[u8; 32]> = proof
            .read_entry::<sp_consensus_vrf::schnorrkel::Randomness>(
                relay_well_known_keys::TWO_EPOCHS_AGO_RANDOMNESS,
                fallback,
            )
            .ok();
        if let Some(r) = two_epochs_ago_randomness {
            randomness_source.two_epochs_ago_randomness = Some((r, block_number));
            is_changed = true;
        }

        if is_changed {
            <RandomnessSource<T>>::put(randomness_source);
        }
    }

    fn on_validation_code_applied() {}
}

pub struct CurrentBlockRandomness<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for CurrentBlockRandomness<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let block_number = <frame_system::Pallet<T>>::block_number();

        let randomness_source = <RandomnessSource<T>>::get();
        if let Some(r) = randomness_source.current_block_randomness {
            ((r.0, subject).using_encoded(T::Hashing::hash), r.1)
        } else {
            (T::Hash::default(), block_number)
        }
    }
}

pub struct OneEpochAgoRandomness<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for OneEpochAgoRandomness<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let block_number = <frame_system::Pallet<T>>::block_number();

        let randomness_source = <RandomnessSource<T>>::get();
        if let Some(r) = randomness_source.one_epoch_ago_randomness {
            ((r.0, subject).using_encoded(T::Hashing::hash), r.1)
        } else {
            (T::Hash::default(), block_number)
        }
    }
}

pub struct TwoEpochsAgoRandomness<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for TwoEpochsAgoRandomness<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let block_number = <frame_system::Pallet<T>>::block_number();

        let randomness_source = <RandomnessSource<T>>::get();
        if let Some(r) = randomness_source.two_epochs_ago_randomness {
            ((r.0, subject).using_encoded(T::Hashing::hash), r.1)
        } else {
            (T::Hash::default(), block_number)
        }
    }
}
