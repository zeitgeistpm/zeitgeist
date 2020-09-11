//! # One-Way Payment Channels
//!
//! TODO

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    traits::{
        BalanceStatus, Currency, ReservableCurrency,
    }
};
use frame_system::ensure_signed;
use sp_core::{Public, sr25519};
use sp_io::crypto::sr25519_verify;
use sp_runtime::traits::StaticLookup;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// The Channel struct contains the information associated with
/// the one-way channels which will have its open and close
/// logic defined in this pallet.
#[derive(Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Channel<AccountId, Balance, BlockNumber> {
    /// The creator is the sending party of the channel.
    creator: AccountId,
    /// The recipient gets the payment from this channel.
    recipient: AccountId,
    /// The signing key is the public key of the creator
    /// that they use to sign channel updates.
    signing_key: [u8; 32],
    /// The deadline after which the channel can be closed
    /// with the most recent update.
    deadline: BlockNumber,
    /// The amount of funds that are reserved on-chain
    /// for this channel.
    collateral: Balance,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: ReservableCurrency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OneWayChannels {
        Channels get(fn channels):
            map hasher(blake2_128_concat) u64 => Channel<T::AccountId, BalanceOf<T>, T::BlockNumber>;

        NextChannelId: u64;
    }
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new channel was created with (ChannelId, Creator).
        NewChannel(u64, AccountId),
    }
);

decl_error! {
    /// Errors for the one-way channels pallet.
    pub enum Error for Module<T: Trait> {
        /// The sender does not have enough balance to provide collateral
        /// for a new channel.
        NotEnoughBalance,
        /// The account attempting to close a channel is not the recipient.
        SenderNotChannelRecipient,
        /// An impossible state was submitted when closing a channel.
        ImpossibleState,
        /// The signature submitted does not verify.
        InvalidSignature,
        /// The message could not be properly decoded.
        MessageDecodingError,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Open a new channel.
        ///
        #[weight = 0]
        pub fn open_channel(
            origin,
            signing_key: [u8; 32],
            destination: <T::Lookup as StaticLookup>::Source,
            #[compact] collateral: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;
            let recipient = T::Lookup::lookup(destination)?;

            ensure!(
                T::Currency::can_reserve(&sender, collateral),
                Error::<T>::NotEnoughBalance,
            );

            let new_channel = Channel {
                creator: sender.clone(),
                recipient,
                signing_key,
                deadline: 0.into(),
                collateral,
            };

            // MUTATIONS START HERE
            T::Currency::reserve(&sender, collateral)?;

            let channel_id = Self::next_channel_id();
            <Channels<T>>::insert(channel_id, new_channel);

            Self::deposit_event(RawEvent::NewChannel(channel_id, sender));
        }

        /// Closes a channel.
        ///
        #[weight = 0]
        pub fn close_channel(
            origin,
            channel_id: u64,
            message: Vec<u8>, // Arbitrary string, but in this case an unsigned integer.
            signature: [u8; 64]
        ) {
            let sender = ensure_signed(origin)?;

            if let Some(amount) = BalanceOf::<T>::decode(&mut message.as_slice()).ok() {
                let channel = Self::channels(channel_id);

                // The closing party of the channel must be the recipient.
                ensure!(sender == channel.recipient, Error::<T>::SenderNotChannelRecipient);

                // Ensure that the message submitted can actual be paid from
                // the reserved collateral.
                ensure!(amount <= channel.collateral, Error::<T>::ImpossibleState);
                
                // Make sure the signature checks out.
                ensure!(
                    Self::is_signed(channel.signing_key, message, signature), 
                    Error::<T>::InvalidSignature
                );

                // Use repatriate to avoid needing to move the funds to free balance first.
                T::Currency::repatriate_reserved(
                    &channel.creator,
                    &channel.recipient,
                    amount,
                    BalanceStatus::Free,
                )?;

                <Channels<T>>::remove(channel_id);

            } else {
                Err(Error::<T>::MessageDecodingError)?;
            }
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_signed(public_key: [u8; 32], msg: Vec<u8>, signature: [u8; 64]) -> bool {
        let sig = sr25519::Signature::from_slice(&signature);
		let pub_key = sr25519::Public::from_slice(&public_key);
		sr25519_verify(&sig, msg.as_slice(), &pub_key)
    }
    
    /// DANGEROUS - MUTATES STORAGE
    fn next_channel_id() -> u64 {
        let id = NextChannelId::get();
        NextChannelId::mutate(|n| *n += 1);
        id
    }
}
