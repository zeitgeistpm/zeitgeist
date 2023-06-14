#[macro_export]
macro_rules! impl_fee_types {
    {} => {
        pub struct DealWithFees;

        type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
        impl OnUnbalanced<NegativeImbalance> for DealWithFees
        {
            fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
                if let Some(mut fees) = fees_then_tips.next() {
                    if let Some(tips) = fees_then_tips.next() {
                        tips.merge_into(&mut fees);
                    }
                    let mut split = fees.ration(
                        FEES_AND_TIPS_TREASURY_PERCENTAGE,
                        FEES_AND_TIPS_BURN_PERCENTAGE,
                    );
                    Treasury::on_unbalanced(split.0);
                }
            }
        }

        pub struct DealWithForeignFees;

        impl OnUnbalanced<CreditOf<AccountId, Tokens>> for DealWithForeignFees
        {
            fn on_unbalanced(fees_and_tips: CreditOf<AccountId, Tokens>) {
                debug_assert!(FEES_AND_TIPS_TREASURY_PERCENTAGE + FEES_AND_TIPS_BURN_PERCENTAGE == 100u32);
                let total_percentage = (FEES_AND_TIPS_TREASURY_PERCENTAGE.saturating_add(FEES_AND_TIPS_BURN_PERCENTAGE)) as u128;
                let fees_and_tips_value = fees_and_tips.peek().clone();
                // Split the merged imbalance into two parts
                let (split_for_treasury, split_for_burn) = fees_and_tips.split(
                    fees_and_tips_value * FEES_AND_TIPS_TREASURY_PERCENTAGE as u128 / total_percentage
                );
                // Handle the split imbalances
                // on_unbalanced is not implemented for other currencies than the native currency
                // https://github.com/paritytech/substrate/blob/85415fb3a452dba12ff564e6b093048eed4c5aad/frame/treasury/src/lib.rs#L618-L627
                // https://github.com/paritytech/substrate/blob/5ea6d95309aaccfa399c5f72e5a14a4b7c6c4ca1/frame/treasury/src/lib.rs#L490
                let _ = <Tokens as Balanced<AccountId>>::resolve(&TreasuryPalletId::get().into_account_truncating(), split_for_treasury);
                // Burn the remaining part
                drop(split_for_burn);
            }
        }
    }
}

#[macro_export]
macro_rules! impl_foreign_fees {
    {} => {
        use frame_support::unsigned::TransactionValidityError;
        use orml_traits::arithmetic::One;
        use orml_traits::arithmetic::Zero;
        use frame_support::traits::fungibles::CreditOf;
        use frame_support::traits::tokens::fungibles::Inspect;
        use frame_support::traits::tokens::WithdrawReasons;
        use frame_support::traits::ExistenceRequirement;
        use frame_support::traits::tokens::WithdrawConsequence;
        use sp_runtime::traits::DispatchInfoOf;
        use sp_runtime::traits::PostDispatchInfoOf;
        use frame_support::pallet_prelude::InvalidTransaction;
        use frame_support::traits::tokens::fungibles::Balanced;
        use zrml_swaps::check_arithm_rslt::CheckArithmRslt;
        use sp_runtime::traits::Convert;

        // TODO: test functions

        // It does foreign fees by extending transactions to include an optional `AssetId` that specifies the asset
        // to be used for payment (defaulting to the native token on `None`). So for each transaction you can specify asset id
        // For real ZTG you use None and for orml_tokens ZTG you use `Some(Asset::Ztg)` and for DOT you use `Some(Asset::Foreign(0))`
        pub struct TokensTxCharger;

        type TTCBalance = <TokensTxCharger as pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime>>::Balance;
        type TTCAsset = <TokensTxCharger as pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime>>::AssetId;
        type TTCLiquidityInfo = <TokensTxCharger as pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime>>::LiquidityInfo;

        pub(crate) fn calculate_fee(
            native_fee: TTCBalance,
            fee_factor: TTCBalance,
            base: TTCBalance,
        ) -> Result<TTCBalance, TransactionValidityError> {
            // We don't know the precision of the underlying asset. Because the converted fee could be
            // less than one (e.g. 0.5) but gets rounded down by integer division we introduce a minimum
            // fee.
            let min_converted_fee: TTCBalance = if native_fee.is_zero() {
                Zero::zero()
            } else {
                One::one()
            };

            let bmul = |a: TTCBalance, b: TTCBalance, base: TTCBalance| -> Option<TTCBalance> {
                let c0 = a.check_mul_rslt(&b).ok()?;
                // The addition of base / 2 before the final division is a way to round to the nearest whole number
                // if the fractional part of (a * b) / base is 0.5 or greater, it rounds up, otherwise, it rounds down.
                let c1 = c0.check_add_rslt(&base.check_div_rslt(&2).ok()?).ok()?;
                c1.check_div_rslt(&base).ok()
            };

            // example DOT: decimals = 10, base = 10^10 = 10_000_000_000
            // assume fee_factor of DOT is 143_120_520
            // now assume fee is 1 ZTG (1 * 10^10),
            // means DOT fee = 1 * 10^10 * 143_120_520 / dot base (10^10) = 143_120_520
            // which is 143_120_520 / 10^10 = 0.0143120520 DOT
            let converted_fee = bmul(native_fee, fee_factor, base)
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(0u8)))?;
            let converted_fee = converted_fee.max(min_converted_fee);

            Ok(converted_fee)
        }

        pub(crate) fn handle_withdraw(
            asset_id: TTCAsset,
            who: &AccountId,
            converted_fee: TTCBalance,
        ) -> Result<TTCLiquidityInfo, TransactionValidityError> {
            let can_withdraw =
                <Tokens as frame_support::traits::fungibles::Inspect<AccountId>>::can_withdraw(asset_id, who, converted_fee);
            if !matches!(can_withdraw, WithdrawConsequence::Success) {
                return Err(InvalidTransaction::Custom(9u8).into());
            }

            <Tokens as Balanced<AccountId>>::withdraw(asset_id, who, converted_fee)
                .map_err(|_| TransactionValidityError::from(InvalidTransaction::Custom(1u8)))
        }

        #[cfg(feature = "parachain")]
        pub(crate) fn get_fee_factor_and_base(
            asset_id: TTCAsset,
        ) -> Result<(TTCBalance, TTCBalance), TransactionValidityError> {
            let location = AssetConvert::convert(asset_id);
            let metadata = location.and_then(|loc| <AssetRegistry as orml_traits::asset_registry::Inspect>::metadata_by_location(&loc))
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)))?;
            let fee_factor = metadata.additional.xcm.fee_factor
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(3u8)))?;
            let base = 10u128.checked_pow(metadata.decimals)
                .ok_or(TransactionValidityError::Invalid(InvalidTransaction::Custom(4u8)))?;
            Ok((fee_factor, base))
        }

        impl pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime> for TokensTxCharger {
            type AssetId = CurrencyId;
            type Balance = Balance;
            type LiquidityInfo = CreditOf<AccountId, Tokens>;

            fn withdraw_fee(
                who: &AccountId,
                call: &RuntimeCall,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                asset_id: Self::AssetId,
                native_fee: Self::Balance,
                _tip: Self::Balance,
            ) -> Result<Self::LiquidityInfo, TransactionValidityError>
            {
                match asset_id {
                    Asset::Ztg => {
                        // this is the case that `Some(Asset::Ztg)` is used,
                        // but we can't deal with pallet_balances here,
                        // because LiquidityInfo is based on orml_tokens
                        // so we use orml_tokens `Asset::Ztg`, which is not the real ZTG of pallet_balances
                        // to get the real ZTG from pallet_balances, you need to specify `None` as asset_id for each transaction
                        let min_converted_fee: TTCBalance = if native_fee.is_zero() {
                            Zero::zero()
                        } else {
                            One::one()
                        };
                        let converted_fee = native_fee.max(min_converted_fee);
                        handle_withdraw(Asset::Ztg, who, converted_fee)
                    },
                    #[cfg(not(feature = "parachain"))]
                    Asset::ForeignAsset(_) => return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(5u8))),
                    #[cfg(feature = "parachain")]
                    Asset::ForeignAsset(_) => {
                        let (fee_factor, base) = get_fee_factor_and_base(asset_id)?;
                        let converted_fee = calculate_fee(native_fee, fee_factor, base)?;
                        handle_withdraw(asset_id, who, converted_fee)
                    },
                    Asset::CategoricalOutcome(_, _) |
                    Asset::ScalarOutcome(_, _) |
                    Asset::CombinatorialOutcome |
                    Asset::PoolShare(_) => return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(6u8))),
                }
            }

            fn correct_and_deposit_fee(
                who: &AccountId,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                _post_info: &PostDispatchInfoOf<RuntimeCall>,
                corrected_native_fee: Self::Balance,
                _tip: Self::Balance,
                paid: Self::LiquidityInfo,
            ) -> Result<(), TransactionValidityError> {
                let asset_id = paid.asset();
                let final_fee = match asset_id {
                    Asset::Ztg => {
                        // Calculate how much refund we should return.
                        let (fee, refund) = paid.split(corrected_native_fee);

                        // Refund to the account that paid the fees. If this fails, the account might have dropped
                        // below the existential balance. In that case we don't refund anything.
                        let _ = <Tokens as Balanced<AccountId>>::resolve(who, refund);

                        fee
                    },
                    #[cfg(not(feature = "parachain"))]
                    Asset::ForeignAsset(_) => return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(7u8))),
                    #[cfg(feature = "parachain")]
                    Asset::ForeignAsset(_) => {
                        let (fee_factor, base) = get_fee_factor_and_base(asset_id)?;
                        let converted_fee = calculate_fee(corrected_native_fee, fee_factor, base)?;

                        // Calculate how much refund we should return.
                        let (fee, refund) = paid.split(converted_fee);

                        // Refund to the account that paid the fees. If this fails, the account might have dropped
                        // below the existential balance. In that case we don't refund anything.
                        let _ = <Tokens as Balanced<AccountId>>::resolve(who, refund);

                        fee
                    },
                    Asset::CategoricalOutcome(_, _) |
                    Asset::ScalarOutcome(_, _) |
                    Asset::CombinatorialOutcome |
                    Asset::PoolShare(_) => return Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(8u8))),
                };

                // Handle the final fee and tip, e.g. by transferring to the treasury.
                // Note: The `corrected_native_fee` already includes the `tip`.
                DealWithForeignFees::on_unbalanced(final_fee);

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! fee_tests {
    {} => {
        use crate::*;
        use frame_support::{dispatch::DispatchClass, weights::Weight};
        use sp_core::H256;
        use sp_runtime::traits::Convert;
        use orml_traits::MultiCurrency;
        use zeitgeist_primitives::constants::BASE;
        use pallet_asset_tx_payment::OnChargeAssetTransaction;
        use frame_support::{assert_noop, assert_ok};
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
        fn withdraws_correct_dot_foreign_asset_fee() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
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
                    assert_eq!(TokensTxCharger::withdraw_fee(
                        &Treasury::account_id(),
                        &mock_call,
                        &mock_dispatch_info,
                        dot,
                        BASE / 2,
                        0,
                    ).unwrap().peek(), 71_560_260);
                }
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
                
                    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(dot)));

                    
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
                    let corrected_native_fee = 71_560_260u128;
                    let paid = <Tokens as Balanced<AccountId>>::issue(dot, 143_120_520u128);
                    let tip = 0u128;
                    assert_ok!(TokensTxCharger::correct_and_deposit_fee(
                        &alice,
                        &mock_dispatch_info,
                        &mock_post_info,
                        corrected_native_fee,
                        tip,
                        paid,
                    ));

                    assert_eq!(1_024_174, Tokens::free_balance(dot, &Treasury::account_id()) - free_balance_treasury_before);
                    assert_eq!(142_096_346, Tokens::free_balance(dot, &alice) - free_balance_alice_before);
                }
            });
        }

        #[test]
        fn get_fee_factor_and_base_metadata_not_found() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into();
            t.execute_with(|| {
                #[cfg(feature = "parachain")]
                {
                    // no registering of dot
                    assert_noop!(get_fee_factor_and_base(Asset::ForeignAsset(0)), TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_and_base_fee_factor_not_found() {
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

                    assert_noop!(get_fee_factor_and_base(dot), TransactionValidityError::Invalid(InvalidTransaction::Custom(3u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_and_base_none_location() {
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

                    assert_noop!(get_fee_factor_and_base(non_location_token), TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8)));
                }
            });
        }

        #[test]
        fn get_fee_factor_and_base_decimals_overflow() {
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

                    assert_noop!(get_fee_factor_and_base(overflow_currency), TransactionValidityError::Invalid(InvalidTransaction::Custom(4u8)));
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
