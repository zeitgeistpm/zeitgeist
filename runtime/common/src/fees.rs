// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#[macro_export]
macro_rules! impl_fee_types {
    () => {
        pub struct DealWithFees;

        type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
        impl OnUnbalanced<NegativeImbalance> for DealWithFees {
            fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
                if let Some(mut fees) = fees_then_tips.next() {
                    if let Some(tips) = fees_then_tips.next() {
                        tips.merge_into(&mut fees);
                    }
                    debug_assert!(
                        FEES_AND_TIPS_TREASURY_PERCENTAGE + FEES_AND_TIPS_BURN_PERCENTAGE == 100u32
                    );
                    let mut split = fees
                        .ration(FEES_AND_TIPS_TREASURY_PERCENTAGE, FEES_AND_TIPS_BURN_PERCENTAGE);
                    Treasury::on_unbalanced(split.0);
                }
            }
        }

        pub struct DealWithForeignFees;

        impl OnUnbalanced<CreditOf<AccountId, Tokens>> for DealWithForeignFees {
            fn on_unbalanced(fees_and_tips: CreditOf<AccountId, Tokens>) {
                // We have to manage the mint / burn ratio on the Zeitgeist chain,
                // but we do not have the responsibility and necessary knowledge to
                // manage the mint / burn ratio for any other chain.
                // Thus we should keep 100% of the foreign tokens in the treasury.
                // Handle the split imbalances
                // on_unbalanced is not implemented for other currencies than the native currency
                // https://github.com/paritytech/substrate/blob/85415fb3a452dba12ff564e6b093048eed4c5aad/frame/treasury/src/lib.rs#L618-L627
                // https://github.com/paritytech/substrate/blob/5ea6d95309aaccfa399c5f72e5a14a4b7c6c4ca1/frame/treasury/src/lib.rs#L490
                let _ = <Tokens as Balanced<AccountId>>::resolve(
                    &TreasuryPalletId::get().into_account_truncating(),
                    fees_and_tips,
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_foreign_fees {
    () => {
        use frame_support::{
            pallet_prelude::InvalidTransaction,
            traits::{
                fungibles::CreditOf,
                tokens::{
                    fungibles::{Balanced, Inspect},
                    BalanceConversion, WithdrawConsequence, WithdrawReasons,
                },
                ExistenceRequirement,
            },
            unsigned::TransactionValidityError,
        };
        use orml_traits::arithmetic::{One, Zero};
        use pallet_asset_tx_payment::HandleCredit;
        use sp_runtime::traits::{Convert, DispatchInfoOf, PostDispatchInfoOf};
        use zrml_swaps::check_arithm_rslt::CheckArithmRslt;

        // It does foreign fees by extending transactions to include an optional `AssetId` that specifies the asset
        // to be used for payment (defaulting to the native token on `None`). So for each transaction you can specify asset id
        // For real ZTG you use None and for orml_tokens ZTG you use `Some(Asset::Ztg)` and for DOT you use `Some(Asset::Foreign(0))`

        pub(crate) fn calculate_fee(
            native_fee: Balance,
            fee_factor: Balance,
        ) -> Result<Balance, TransactionValidityError> {
            // Assume a fee_factor of 143_120_520 for DOT, now divide by
            // BASE (10^10) = 0.0143120520 DOT per ZTG.
            // Keep in mind that ZTG BASE is 10_000_000_000, and because fee_factor is below that,
            // less DOT than ZTG is paid for fees.
            // Assume a fee_factor of 20_000_000_000, then the fee would result in
            // 20_000_000_000 / 10_000_000_000 = 2 units per ZTG
            let converted_fee = zrml_swaps::fixed::bmul(native_fee, fee_factor)
                .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(0u8)))?;

            Ok(converted_fee)
        }

        #[cfg(feature = "parachain")]
        pub(crate) fn get_fee_factor(
            asset_id: CurrencyId,
        ) -> Result<Balance, TransactionValidityError> {
            let location = AssetConvert::convert(asset_id);
            let metadata = location
                .and_then(|loc| {
                    <AssetRegistry as orml_traits::asset_registry::Inspect>::metadata_by_location(
                        &loc,
                    )
                })
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)))?;
            let fee_factor = metadata
                .additional
                .xcm
                .fee_factor
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(3u8)))?;
            Ok(fee_factor)
        }

        pub struct TTCBalanceToAssetBalance;
        impl BalanceConversion<Balance, CurrencyId, Balance> for TTCBalanceToAssetBalance {
            type Error = TransactionValidityError;

            fn to_asset_balance(
                native_fee: Balance,
                asset_id: CurrencyId,
            ) -> Result<Balance, Self::Error> {
                match asset_id {
                    Asset::Ztg => Ok(native_fee),
                    #[cfg(not(feature = "parachain"))]
                    Asset::ForeignAsset(_) => {
                        return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(
                            1u8,
                        )));
                    }
                    #[cfg(feature = "parachain")]
                    Asset::ForeignAsset(_) => {
                        let fee_factor = get_fee_factor(asset_id)?;
                        let converted_fee = calculate_fee(native_fee, fee_factor)?;
                        Ok(converted_fee)
                    }
                    Asset::CategoricalOutcome(_, _)
                    | Asset::ScalarOutcome(_, _)
                    | Asset::CombinatorialOutcome
                    | Asset::PoolShare(_) => {
                        return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(
                            2u8,
                        )));
                    }
                }
            }
        }

        pub struct TTCHandleCredit;
        impl HandleCredit<AccountId, Tokens> for TTCHandleCredit {
            fn handle_credit(final_fee: CreditOf<AccountId, Tokens>) {
                // Handle the final fee and tip, e.g. by transferring to the treasury.
                DealWithForeignFees::on_unbalanced(final_fee);
            }
        }
    };
}

#[macro_export]
macro_rules! fee_tests {
    () => {
        use crate::*;
        use frame_support::{dispatch::DispatchClass, weights::Weight};
        use sp_core::H256;
        use sp_runtime::traits::Convert;
        use orml_traits::MultiCurrency;
        use zeitgeist_primitives::constants::BASE;
        use pallet_asset_tx_payment::OnChargeAssetTransaction;
        use frame_support::{assert_noop, assert_ok};
        #[cfg(feature = "parachain")]
        use orml_asset_registry::AssetMetadata;

        fn run_with_system_weight<F>(w: Weight, mut assertions: F)
        where
            F: FnMut(),
        {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                System::set_block_consumed_resources(w, 0);
                assertions()
            });
        }

        #[test]
        fn treasury_receives_correct_amount_of_native_fees_and_tips() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                let fee_balance = 3 * ExistentialDeposit::get();
                let fee_imbalance = Balances::issue(fee_balance);
                let tip_balance = 7 * ExistentialDeposit::get();
                let tip_imbalance = Balances::issue(tip_balance);
                assert_eq!(Balances::free_balance(Treasury::account_id()), 0);
                DealWithFees::on_unbalanceds(vec![fee_imbalance, tip_imbalance].into_iter());
                assert_eq!(
                    Balances::free_balance(Treasury::account_id()),
                    fee_balance + tip_balance,
                );
            });
        }

        #[test]
        fn treasury_receives_correct_amount_of_foreign_fees_and_tips() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                let fee_and_tip_balance = 10 * ExistentialDeposit::get();
                let fees_and_tips = <Tokens as Balanced<AccountId>>::issue(Asset::ForeignAsset(0), fee_and_tip_balance);
                assert!(Tokens::free_balance(Asset::ForeignAsset(0), &Treasury::account_id()).is_zero());
                DealWithForeignFees::on_unbalanced(fees_and_tips);
                assert_eq!(
                    Tokens::free_balance(Asset::ForeignAsset(0), &Treasury::account_id()),
                    fee_and_tip_balance,
                );
            });
        }

        #[test]
        #[cfg(feature = "parachain")]
        fn withdraws_correct_dot_foreign_asset_fee() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                let fee_factor = 143_120_520;
                let custom_metadata = CustomMetadata {
                    xcm: XcmMetadata { fee_factor: Some(fee_factor) },
                    ..Default::default()
                };
                let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
                    decimals: 10,
                    name: "Polkadot".into(),
                    symbol: "DOT".into(),
                    existential_deposit: ExistentialDeposit::get(),
                    location: Some(xcm::VersionedMultiLocation::V1(xcm::latest::MultiLocation::parent())),
                    additional: custom_metadata,
                };
                let dot = Asset::ForeignAsset(0);

                assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(dot)));

                let fees_and_tips = <Tokens as Balanced<AccountId>>::issue(dot, 0);
                assert_ok!(<Tokens as MultiCurrency<AccountId>>::deposit(dot, &Treasury::account_id(), BASE));

                let mock_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
                let mock_dispatch_info = frame_support::dispatch::DispatchInfo {
                    weight:  frame_support::dispatch::Weight::zero(),
                    class: DispatchClass::Normal,
                    pays_fee:  frame_support::dispatch::Pays::Yes,
                };
                assert_eq!(<TokensTxCharger as OnChargeAssetTransaction<Runtime>>::withdraw_fee(
                    &Treasury::account_id(),
                    &mock_call,
                    &mock_dispatch_info,
                    dot,
                    BASE / 2,
                    0,
                ).unwrap().peek(), 71_560_260);
            });
        }

        #[test]
        fn correct_and_deposit_fee_dot_foreign_asset() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    let alice =  AccountId::from([0u8; 32]);
                    let fee_factor = 143_120_520;
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(fee_factor) },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
                        decimals: 10,
                        name: "Polkadot".into(),
                        symbol: "DOT".into(),
                        existential_deposit: ExistentialDeposit::get(),
                        location: Some(xcm::VersionedMultiLocation::V1(xcm::latest::MultiLocation::parent())),
                        additional: custom_metadata,
                    };
                    let dot = Asset::ForeignAsset(0);

                    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta.clone(), Some(dot)));


                    assert_ok!(<Tokens as MultiCurrency<AccountId>>::deposit(dot, &Treasury::account_id(), BASE));

                    let mock_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
                    let mock_dispatch_info = frame_support::dispatch::DispatchInfo {
                        weight:  frame_support::dispatch::Weight::zero(),
                        class: DispatchClass::Normal,
                        pays_fee:  frame_support::dispatch::Pays::Yes,
                    };
                    let mock_post_info = frame_support::dispatch::PostDispatchInfo {
                        actual_weight:  Some(frame_support::dispatch::Weight::zero()),
                        pays_fee:  frame_support::dispatch::Pays::Yes,
                    };

                    let free_balance_treasury_before = Tokens::free_balance(dot, &Treasury::account_id());
                    let free_balance_alice_before = Tokens::free_balance(dot, &alice);
                    let corrected_native_fee = BASE;
                    let paid = <Tokens as Balanced<AccountId>>::issue(dot, 2 * BASE);
                    let paid_balance = paid.peek();
                    let tip = 0u128;
                    assert_ok!(<TokensTxCharger as OnChargeAssetTransaction<Runtime>>::correct_and_deposit_fee(
                        &alice,
                        &mock_dispatch_info,
                        &mock_post_info,
                        corrected_native_fee,
                        tip,
                        paid,
                    ));

                    let treasury_gain = Tokens::free_balance(dot, &Treasury::account_id()) - free_balance_treasury_before;
                    let alice_gain = Tokens::free_balance(dot, &alice) - free_balance_alice_before;

                    let decimals = meta.decimals;
                    let base = 10u128.checked_pow(decimals).unwrap();

                    let dot_fee = ((corrected_native_fee * fee_factor) + (base / 2)) / base;
                    assert_eq!(dot_fee, treasury_gain);
                    assert_eq!(143_120_520, treasury_gain);
                    assert_eq!(paid_balance - treasury_gain, alice_gain);
                }
            });
        }

        #[test]
        fn get_fee_factor_metadata_not_found() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    // no registering of dot
                    assert_noop!(get_fee_factor(Asset::ForeignAsset(0)), TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_fee_factor_not_found() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: None },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
                        decimals: 10,
                        name: "Polkadot".into(),
                        symbol: "DOT".into(),
                        existential_deposit: ExistentialDeposit::get(),
                        location: Some(xcm::VersionedMultiLocation::V1(xcm::latest::MultiLocation::parent())),
                        additional: custom_metadata,
                    };
                    let dot = Asset::ForeignAsset(0);

                    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(dot)));

                    assert_noop!(get_fee_factor(dot), TransactionValidityError::Invalid(InvalidTransaction::Custom(3u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_none_location() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(10_393) },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
                        decimals: 10,
                        name: "NoneLocationToken".into(),
                        symbol: "NONE".into(),
                        existential_deposit: ExistentialDeposit::get(),
                        location: None,
                        additional: custom_metadata,
                    };
                    let non_location_token = Asset::ForeignAsset(1);

                    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(non_location_token)));

                    assert_noop!(get_fee_factor(non_location_token), TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_decimals_overflow() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(102_993) },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
                        decimals: u32::MAX,
                        name: "OverflowCurrency".into(),
                        symbol: "OVER".into(),
                        existential_deposit: ExistentialDeposit::get(),
                        location: Some(xcm::VersionedMultiLocation::V1(xcm::latest::MultiLocation::parent())),
                        additional: custom_metadata,
                    };
                    let overflow_currency = Asset::ForeignAsset(1);

                    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(overflow_currency)));

                    assert_noop!(get_fee_factor(overflow_currency), TransactionValidityError::Invalid(InvalidTransaction::Custom(4u8)));
                }
            });
        }

        #[test]
        fn fee_multiplier_can_grow_from_zero() {
            let minimum_multiplier = MinimumMultiplier::get();
            let target = TargetBlockFullness::get()
                * RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
            // if the min is too small, then this will not change, and we are doomed forever.
            // the weight is 1/100th bigger than target.
            run_with_system_weight(target * 101 / 100, || {
                let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
                assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
            })
        }
    }
}
