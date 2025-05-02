// Copyright 2022-2025 Forecasting Technologies LTD.
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
        use frame_support::{
            pallet_prelude::TypedGet,
            traits::{fungible::Credit as FungibleCredit, tokens::imbalance::ResolveTo},
        };
        use pallet_treasury::TreasuryAccountId;

        /// Deal with substrate based fees and tip. This should be used with pallet_transaction_payment.
        pub struct DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>(
            sp_std::marker::PhantomData<(R, FeesTreasuryProportion)>,
        );
        impl<R, FeesTreasuryProportion> DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>
        where
            R: pallet_balances::Config + pallet_treasury::Config,
            FeesTreasuryProportion: Get<Perbill>,
        {
            fn deal_with_fees(amount: FungibleCredit<R::AccountId, pallet_balances::Pallet<R>>) {
                // Balances pallet automatically burns dropped Credits by decreasing
                // total_supply accordingly
                let treasury_proportion = FeesTreasuryProportion::get();
                let treasury_part = treasury_proportion.deconstruct();
                let burn_part = Perbill::one().deconstruct() - treasury_part;
                let (_, to_treasury) = amount.ration(burn_part, treasury_part);
                ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(
                    to_treasury,
                );
            }

            fn deal_with_tip(amount: FungibleCredit<R::AccountId, pallet_balances::Pallet<R>>) {
                ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(
                    amount,
                );
            }
        }

        impl<R, FeesTreasuryProportion>
            OnUnbalanced<FungibleCredit<R::AccountId, pallet_balances::Pallet<R>>>
            for DealWithSubstrateFeesAndTip<R, FeesTreasuryProportion>
        where
            R: pallet_balances::Config + pallet_treasury::Config,
            FeesTreasuryProportion: Get<Perbill>,
        {
            fn on_unbalanceds(
                mut fees_then_tips: impl Iterator<
                    Item = FungibleCredit<R::AccountId, pallet_balances::Pallet<R>>,
                >,
            ) {
                if let Some(fees) = fees_then_tips.next() {
                    Self::deal_with_fees(fees);
                    if let Some(tip) = fees_then_tips.next() {
                        Self::deal_with_tip(tip);
                    }
                }
            }
        }

        pub struct DealWithForeignFees;

        impl OnUnbalanced<Credit<AccountId, Tokens>> for DealWithForeignFees {
            fn on_unbalanced(fees_and_tips: Credit<AccountId, Tokens>) {
                // We have to manage the mint / burn ratio on the Zeitgeist chain,
                // but we do not have the responsibility and necessary knowledge to
                // manage the mint / burn ratio for any other chain.
                // Thus we should keep 100% of the foreign tokens in the treasury.
                // Handle the split imbalances
                // on_unbalanced is not implemented for other currencies than the native currency
                // https://github.com/paritytech/substrate/blob/85415fb3a452dba12ff564e6b093048eed4c5aad/frame/treasury/src/lib.rs#L618-L627
                // https://github.com/paritytech/substrate/blob/5ea6d95309aaccfa399c5f72e5a14a4b7c6c4ca1/frame/treasury/src/lib.rs#L490
                let res = <Tokens as Balanced<AccountId>>::resolve(
                    &TreasuryPalletId::get().into_account_truncating(),
                    fees_and_tips,
                );
                debug_assert!(res.is_ok());
            }
        }
    };
}

#[macro_export]
macro_rules! impl_foreign_fees {
    () => {
        use frame_support::{
            ensure,
            pallet_prelude::InvalidTransaction,
            traits::{
                fungibles::{Credit, Inspect},
                tokens::{
                    fungibles::Balanced, ConversionToAssetBalance, Fortitude, Precision,
                    Preservation, WithdrawConsequence, WithdrawReasons,
                },
                ExistenceRequirement,
            },
            unsigned::TransactionValidityError,
        };
        use orml_traits::{
            arithmetic::{One, Zero},
            asset_registry::Inspect as AssetRegistryInspect,
        };
        use pallet_asset_tx_payment::HandleCredit;
        use sp_runtime::{
            traits::{Convert, DispatchInfoOf, PostDispatchInfoOf},
            Perbill,
        };
        use zeitgeist_primitives::math::fixed::{FixedDiv, FixedMul};

        #[repr(u8)]
        pub enum CustomTxError {
            FeeConversionArith = 0,
            NoForeignAssetsOnStandaloneChain = 1,
            NoAssetMetadata = 2,
            NoFeeFactor = 3,
            NonForeignAssetPaid = 4,
            InvalidFeeAsset = 5,
            NonNativeFeeAssetOnStandaloneChain = 6,
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "parachain")] {
                /// The asset id specifically used for pallet_assets_tx_payment for
                /// paying transaction fees in different assets.
                /// Since the polkadot API extension assumes the same type as on the asset-hubs,
                /// we use it too.
                /// https://github.com/polkadot-fellows/runtimes/blob/20ac6ff4dc4c488ad08f507c14b899adc6cb4394/system-parachains/asset-hubs/asset-hub-polkadot/src/lib.rs#L767
                pub type TxPaymentAssetId = xcm::latest::Location;

                pub(crate) fn convert_asset_to_currency_id(
                    value: TxPaymentAssetId,
                ) -> Result<CurrencyId, TransactionValidityError> {
                    // value (TxPaymentAssetId) is a Location as defined above
                    let currency_id = AssetRegistry::location_to_asset_id(value).ok_or(
                        TransactionValidityError::Invalid(InvalidTransaction::Custom(
                            CustomTxError::InvalidFeeAsset as u8,
                        )),
                    )?;
                    Ok(currency_id)
                }

                pub(crate) fn get_fee_factor(
                    currency_id: CurrencyId,
                ) -> Result<Balance, TransactionValidityError> {
                    let metadata = <AssetRegistry as AssetRegistryInspect>::metadata(&currency_id).ok_or(
                        TransactionValidityError::Invalid(InvalidTransaction::Custom(
                            CustomTxError::NoAssetMetadata as u8,
                        )),
                    )?;
                    let fee_factor =
                        metadata.additional.xcm.fee_factor.ok_or(TransactionValidityError::Invalid(
                            InvalidTransaction::Custom(CustomTxError::NoFeeFactor as u8),
                        ))?;
                    Ok(fee_factor)
                }

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
                    let converted_fee = native_fee.bmul(fee_factor).map_err(|_| {
                        TransactionValidityError::Invalid(InvalidTransaction::Custom(
                            CustomTxError::FeeConversionArith as u8,
                        ))
                    })?;

                    Ok(converted_fee)
                }

                pub struct TTCBalanceToAssetBalance;
                impl ConversionToAssetBalance<Balance, CurrencyId, Balance> for TTCBalanceToAssetBalance {
                    type Error = TransactionValidityError;

                    fn to_asset_balance(
                        native_fee: Balance,
                        asset_id: CurrencyId,
                    ) -> Result<Balance, Self::Error> {
                        let fee_factor = get_fee_factor(asset_id)?;
                        let converted_fee = calculate_fee(native_fee, fee_factor)?;
                        Ok(converted_fee)
                    }
                }
            }
        }

        pub struct TTCHandleCredit;
        impl HandleCredit<AccountId, Tokens> for TTCHandleCredit {
            fn handle_credit(final_fee: Credit<AccountId, Tokens>) {
                // Handle the final fee and tip, e.g. by transferring to the treasury.
                DealWithForeignFees::on_unbalanced(final_fee);
            }
        }

        // It does calculate foreign fees by extending transactions to include an optional
        // `AssetId` that specifies the asset to be used for payment (defaulting to the native
        // token on `None`), such that for each transaction the asset id can be specified.
        // For real ZTG `None` is used and for DOT `Some(Asset::Foreign(0))` is used.
        pub struct TokensTxCharger;

        #[cfg(feature = "parachain")]
        impl pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime> for TokensTxCharger {
            type AssetId = TxPaymentAssetId;
            type Balance = Balance;
            type LiquidityInfo = Credit<AccountId, Tokens>;

            fn withdraw_fee(
                who: &AccountId,
                call: &RuntimeCall,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                asset_id: Self::AssetId,
                native_fee: Self::Balance,
                _tip: Self::Balance,
            ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
                let currency_id = convert_asset_to_currency_id(asset_id)?;
                // We don't know the precision of the underlying asset. Because the converted fee
                // could be less than one (e.g. 0.5) but gets rounded down by integer division we
                // introduce a minimum fee.
                let min_converted_fee =
                    if native_fee.is_zero() { Zero::zero() } else { One::one() };
                let converted_fee =
                    TTCBalanceToAssetBalance::to_asset_balance(native_fee, currency_id)?
                        .max(min_converted_fee);
                let can_withdraw =
                    <Tokens as Inspect<AccountId>>::can_withdraw(currency_id, who, converted_fee);
                if can_withdraw != WithdrawConsequence::Success {
                    return Err(InvalidTransaction::Payment.into());
                }
                let result = <Tokens as Balanced<AccountId>>::withdraw(
                    currency_id,
                    who,
                    converted_fee,
                    Precision::Exact,
                    Preservation::Protect,
                    Fortitude::Polite,
                );
                result.map_err(|_| TransactionValidityError::from(InvalidTransaction::Payment))
            }

            fn correct_and_deposit_fee(
                who: &AccountId,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                _post_info: &PostDispatchInfoOf<RuntimeCall>,
                corrected_native_fee: Self::Balance,
                tip: Self::Balance,
                paid: Self::LiquidityInfo,
            ) -> Result<(Self::Balance, Self::Balance), TransactionValidityError> {
                let min_converted_fee =
                    if corrected_native_fee.is_zero() { Zero::zero() } else { One::one() };
                // Convert the corrected fee and tip into the asset used for payment.
                let converted_fee =
                    TTCBalanceToAssetBalance::to_asset_balance(corrected_native_fee, paid.asset())?
                        .max(min_converted_fee);
                let converted_tip = TTCBalanceToAssetBalance::to_asset_balance(tip, paid.asset())?;

                // Calculate how much refund we should return.
                let (final_fee, refund) = paid.split(converted_fee);
                // Refund to the account that paid the fees. If this fails, the account might have
                // dropped below the existential balance. In that case we don't refund anything.
                let _ = <Tokens as Balanced<AccountId>>::resolve(who, refund);

                // Handle the final fee and tip, e.g. by transferring to the treasury.
                // Note: The `corrected_native_fee` already includes the `tip`.
                let final_fee_peek = final_fee.peek();
                TTCHandleCredit::handle_credit(final_fee);

                Ok((final_fee_peek, tip))
            }
        }

        #[cfg(not(feature = "parachain"))]
        impl pallet_asset_tx_payment::OnChargeAssetTransaction<Runtime> for TokensTxCharger {
            // used `u32` since we don't care about decoding in the polkadot API, because it would throw an error anyways
            // additionally, we don't want to add the `xcm` dependency to the standalone chain (without parachain feature)
            type AssetId = u32;
            type Balance = Balance;
            type LiquidityInfo = Credit<AccountId, Tokens>;

            fn withdraw_fee(
                _who: &AccountId,
                _call: &RuntimeCall,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                _asset_id: Self::AssetId,
                _native_fee: Self::Balance,
                _tip: Self::Balance,
            ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
                Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(
                    CustomTxError::NonNativeFeeAssetOnStandaloneChain as u8,
                )))
            }

            fn correct_and_deposit_fee(
                _who: &AccountId,
                _dispatch_info: &DispatchInfoOf<RuntimeCall>,
                _post_info: &PostDispatchInfoOf<RuntimeCall>,
                _corrected_native_fee: Self::Balance,
                _tip: Self::Balance,
                _paid: Self::LiquidityInfo,
            ) -> Result<(Self::Balance, Self::Balance), TransactionValidityError> {
                Err(TransactionValidityError::Invalid(InvalidTransaction::Custom(
                    CustomTxError::NonNativeFeeAssetOnStandaloneChain as u8,
                )))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_market_creator_fees {
    () => {
        pub struct MarketCreatorFee;

        /// Uses the `creator_fee` field defined by the specified market to deduct a fee for the
        /// market's creator. Calling `distribute` is noop if the market doesn't exist or the
        /// transfer fails for any reason.
        impl DistributeFees for MarketCreatorFee {
            type Asset = Asset<MarketId>;
            type AccountId = AccountId;
            type Balance = Balance;
            type MarketId = MarketId;

            fn distribute(
                market_id: Self::MarketId,
                asset: Self::Asset,
                account: &Self::AccountId,
                amount: Self::Balance,
            ) -> Self::Balance {
                Self::do_distribute(market_id, asset, account, amount)
                    .unwrap_or_else(|_| 0u8.saturated_into())
            }

            fn fee_percentage(market_id: Self::MarketId) -> Perbill {
                Self::fee_percentage(market_id).unwrap_or(Perbill::zero())
            }
        }

        impl MarketCreatorFee {
            fn do_distribute(
                market_id: MarketId,
                asset: Asset<MarketId>,
                account: &AccountId,
                amount: Balance,
            ) -> Result<Balance, DispatchError> {
                let market = MarketCommons::market(&market_id)?;
                let fee_amount = Self::fee_percentage(market_id)?.mul_floor(amount);
                // Might fail if the transaction is too small
                <AssetManager as MultiCurrency<_>>::transfer(
                    asset,
                    account,
                    &market.creator,
                    fee_amount,
                )?;
                Ok(fee_amount)
            }

            fn fee_percentage(market_id: MarketId) -> Result<Perbill, DispatchError> {
                let market = MarketCommons::market(&market_id)?;
                Ok(market.creator_fee)
            }
        }
    };
}

#[macro_export]
macro_rules! fee_tests {
   () => {
        use crate::*;
        use frame_support::{assert_noop, assert_ok, dispatch::DispatchClass, traits::fungible::Balanced as SubstrateBalanced, weights::Weight};
        use orml_traits::MultiCurrency;
        use pallet_asset_tx_payment::OnChargeAssetTransaction;
        use sp_core::H256;
        use sp_runtime::BuildStorage;
        use zeitgeist_primitives::constants::{BASE, MILLI};

        fn run_with_system_weight<F>(w: Weight, mut assertions: F)
        where
            F: FnMut(),
        {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap().into();
            t.execute_with(|| {
                System::set_block_consumed_resources(w, 0);
                assertions()
            });
        }

        #[test]
        fn treasury_receives_correct_amount_of_native_fees_and_tips() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap().into();
            t.execute_with(|| {
                let fee_balance = 3 * ExistentialDeposit::get();
                let fee_imbalance = <Balances as SubstrateBalanced<AccountId>>::issue(fee_balance);
                let tip_balance = 7 * ExistentialDeposit::get();
                let tip_imbalance = <Balances as SubstrateBalanced<AccountId>>::issue(tip_balance);
                assert_eq!(Balances::free_balance(Treasury::account_id()), 0);
                DealWithSubstrateFeesAndTip::<Runtime, FeesTreasuryProportion>::on_unbalanceds(
                    vec![fee_imbalance, tip_imbalance].into_iter(),
                );
                assert_eq!(
                    Balances::free_balance(Treasury::account_id()),
                    fee_balance + tip_balance,
                );
            });
        }

        #[test]
        fn treasury_receives_correct_amount_of_foreign_fees_and_tips() {
            let mut t: sp_io::TestExternalities =
                frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap().into();
            t.execute_with(|| {
                let fee_and_tip_balance = 10 * ExistentialDeposit::get();
                let fees_and_tips = <Tokens as Balanced<AccountId>>::issue(
                    Asset::ForeignAsset(0),
                    fee_and_tip_balance,
                );
                assert!(
                    Tokens::free_balance(Asset::ForeignAsset(0), &Treasury::account_id()).is_zero()
                );
                DealWithForeignFees::on_unbalanced(fees_and_tips);
                assert_eq!(
                    Tokens::free_balance(Asset::ForeignAsset(0), &Treasury::account_id()),
                    fee_and_tip_balance,
                );
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

        #[cfg(feature = "parachain")]
        mod parachain {
            use super::*;
            use orml_traits::asset_registry::AssetMetadata;

            #[test]
            fn correct_and_deposit_fee_dot_foreign_asset() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    {
                        let alice =  AccountId::from([0u8; 32]);
                        let fee_factor = 143_120_520;
                        let custom_metadata = CustomMetadata {
                            xcm: XcmMetadata { fee_factor: Some(fee_factor) },
                            ..Default::default()
                        };
                        let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
                            decimals: 10,
                            name: "Polkadot".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "DOT".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: ExistentialDeposit::get(),
                            location: Some(xcm::VersionedLocation::V4(xcm::latest::Location::parent())),
                            additional: custom_metadata,
                        };
                        let dot = Asset::ForeignAsset(0);

                        assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta.clone(), Some(dot)));


                        assert_ok!(<Tokens as MultiCurrency<AccountId>>::deposit(dot, &Treasury::account_id(), BASE));

                        let mock_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
                        let mock_dispatch_info = frame_support::dispatch::DispatchInfo {
                            weight: frame_support::weights::Weight::zero(),
                            class: DispatchClass::Normal,
                            pays_fee:  frame_support::dispatch::Pays::Yes,
                        };
                        let mock_post_info = frame_support::dispatch::PostDispatchInfo {
                            actual_weight: Some(frame_support::weights::Weight::zero()),
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
            fn get_fee_factor_works() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(143_120_520u128) },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> =
                        AssetMetadata {
                            decimals: 10,
                            name: "Polkadot".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "DOT".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: ExistentialDeposit::get(),
                            location: Some(xcm::VersionedLocation::V4(
                                xcm::latest::Location::parent(),
                            )),
                            additional: custom_metadata,
                        };
                    let dot_asset_id = 0u32;
                    let dot = Asset::ForeignAsset(dot_asset_id);

                    assert_ok!(AssetRegistry::register_asset(
                        RuntimeOrigin::root(),
                        meta,
                        Some(dot)
                    ));

                    assert_eq!(get_fee_factor(dot).unwrap(), 143_120_520u128);
                });
            }

            #[test]
            fn get_fee_factor_metadata_not_found() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    {
                        // no registering of dot
                        assert_noop!(
                            get_fee_factor(Asset::ForeignAsset(0)),
                            TransactionValidityError::Invalid(InvalidTransaction::Custom(2u8))
                        );
                    }
                });
            }

            #[test]
            fn get_fee_factor_fee_factor_not_found() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: None },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> =
                        AssetMetadata {
                            decimals: 10,
                            name: "Polkadot".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "DOT".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: ExistentialDeposit::get(),
                            location: Some(xcm::VersionedLocation::V4(
                                xcm::latest::Location::parent(),
                            )),
                            additional: custom_metadata,
                        };
                    let dot = Asset::ForeignAsset(0u32);

                    assert_ok!(AssetRegistry::register_asset(
                        RuntimeOrigin::root(),
                        meta,
                        Some(dot)
                    ));

                    assert_noop!(
                        get_fee_factor(dot),
                        TransactionValidityError::Invalid(InvalidTransaction::Custom(3u8))
                    );
                });
            }

            #[test]
            fn get_fee_factor_none_location() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(10_393) },
                        ..Default::default()
                    };
                    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> =
                        AssetMetadata {
                            decimals: 10,
                            name: "NoneLocationToken".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "NONE".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: ExistentialDeposit::get(),
                            location: None,
                            additional: custom_metadata,
                        };
                    let non_location_token = Asset::ForeignAsset(1);

                    assert_ok!(AssetRegistry::register_asset(
                        RuntimeOrigin::root(),
                        meta,
                        Some(non_location_token)
                    ));

                    assert_eq!(get_fee_factor(non_location_token).unwrap(), 10_393);
                });
            }

            #[test]
            fn withdraws_correct_dot_foreign_asset_fee() {
                let mut t: sp_io::TestExternalities =
                    frame_system::GenesisConfig::<Runtime>::default()
                        .build_storage()
                        .unwrap()
                        .into();
                t.execute_with(|| {
                    let fee_factor = 143_120_520;
                    let custom_metadata = CustomMetadata {
                        xcm: XcmMetadata { fee_factor: Some(fee_factor) },
                        allow_as_base_asset: true,
                    };
                    let dot_location = xcm::latest::Location::parent();
                    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> =
                        AssetMetadata {
                            decimals: 10,
                            name: "Polkadot".as_bytes().to_vec().try_into().unwrap(),
                            symbol: "DOT".as_bytes().to_vec().try_into().unwrap(),
                            existential_deposit: 5 * MILLI,
                            location: Some(xcm::VersionedLocation::V4(
                                dot_location.clone(),
                            )),
                            additional: custom_metadata,
                        };
                    let dot = Asset::ForeignAsset(0u32);

                    assert_ok!(AssetRegistry::register_asset(
                        RuntimeOrigin::root(),
                        meta,
                        Some(dot)
                    ));

                    let fees_and_tips = <Tokens as Balanced<AccountId>>::issue(dot, 0);
                    assert_ok!(<Tokens as MultiCurrency<AccountId>>::deposit(
                        dot,
                        &Treasury::account_id(),
                        BASE
                    ));

                    let mock_call =
                        RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
                    let mock_dispatch_info = frame_support::dispatch::DispatchInfo {
                        weight: frame_support::weights::Weight::zero(),
                        class: DispatchClass::Normal,
                        pays_fee: frame_support::dispatch::Pays::Yes,
                    };
                    assert_eq!(
                        <TokensTxCharger as OnChargeAssetTransaction<Runtime>>::withdraw_fee(
                            &Treasury::account_id(),
                            &mock_call,
                            &mock_dispatch_info,
                            dot_location,
                            BASE / 2,
                            0,
                        )
                        .unwrap()
                        .peek(),
                        71_560_260
                    );
                });
            }
        }
    }
}
