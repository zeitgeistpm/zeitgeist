//! Small pallet responsible for storing a set of accounts, and their associated session keys.
//! This is a minimal solution where staking would be used in practice.
//! The accounts are set and genesis and never change.
//!
//! The Substrate ecosystem has a wide variety of real-world solutions and examples of what this
//! pallet could be replaced with.
//! Gautam's validator set pallet - https://github.com/paritytech/substrate/tree/master/frame/staking/
//! Parity's pallet staking - https://github.com/paritytech/substrate/tree/master/frame/staking/
//! Moonbeam's Parachain Staking - https://github.com/PureStake/moonbeam/tree/master/pallets/parachain-staking
//! Recipe for AccountSet, VecSet, and MapSet

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	#[cfg(feature = "std")]
	use log::warn;
	use nimbus_primitives::{AccountLookup, CanAuthor, NimbusId};
	use sp_std::vec::Vec;

	/// The Account Set pallet
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	/// The set of accounts that is stored in this pallet.
	#[pallet::storage]
	pub type StoredAccounts<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			StoredAccounts::<T>::get()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn account_id_of)]
	/// A mapping from the AuthorIds used in the consensus layer
	/// to the AccountIds runtime.
	type Mapping<T: Config> = StorageMap<_, Twox64Concat, NimbusId, T::AccountId, OptionQuery>;

	#[pallet::genesis_config]
	/// Genesis config for author mapping pallet
	pub struct GenesisConfig<T: Config> {
		/// The associations that should exist at chain genesis
		pub mapping: Vec<(T::AccountId, NimbusId)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { mapping: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if self.mapping.is_empty() {
				warn!(target: "account-set", "No mappings at genesis. Your chain will have no valid authors.");
			}
			for (account_id, author_id) in &self.mapping {
				Mapping::<T>::insert(author_id, account_id);
				StoredAccounts::<T>::append(account_id);
			}
		}
	}

	/// This pallet is compatible with nimbus's author filtering system. Any account stored in this pallet
	/// is a valid author. Notice that this implementation does not have an inner filter, so it
	/// can only be the beginning of the nimbus filter pipeline.
	impl<T: Config> CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(author: &T::AccountId, _slot: &u32) -> bool {
			StoredAccounts::<T>::get().contains(author)
		}
	}

	impl<T: Config> AccountLookup<T::AccountId> for Pallet<T> {
		fn lookup_account(author: &NimbusId) -> Option<T::AccountId> {
			Mapping::<T>::get(&author)
		}
	}
}
