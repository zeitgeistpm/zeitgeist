use core::fmt::Debug;
use frame_support::weights::constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND};
use parity_scale_codec::MaxEncodedLen;
use zeitgeist_primitives::types::{Balance, Asset, TokenInfo};

// The fee cost per second for transferring the ztg token
pub fn ztg_per_second<MI: MaxEncodedLen + Debug>() -> Balance {
    base_tx_per_second(<Asset<MI>>::ZTG)
}

// The fee cost per second for transferring the KSM token
// We assume that KSM price is 50x ZTG price
// TODO: Use either ksm_per_second or roc_per_second depending on current runtime
/*
pub fn ksm_per_second<MI: MaxEncodedLen>() -> Balance {
    base_tx_per_second(<Asset<MI>>::KSM) / 50
}
*/

// The fee cost per second for transferring the ROC token
// We assume that ROC "price" is 10x ZBS price
pub fn roc_per_second<MI: MaxEncodedLen + Debug>() -> Balance {
    base_tx_per_second(<Asset<MI>>::ROC) / 10
}


// The fee cost per second for transferring the aUSD token
// We assume that 1 aUSD = 1 USD = 1 ZTG
pub fn ausd_per_second<MI: MaxEncodedLen + Debug>() -> Balance {
    base_tx_per_second(<Asset<MI>>::AUSD)
}


// The fee cost per second for any Asset
fn base_tx_per_second<MI: MaxEncodedLen + Debug>(currency: Asset<MI>) -> Balance {
    let base_weight = Balance::from(ExtrinsicBaseWeight::get());
    let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
    base_tx_per_second * base_tx(currency)
}

// Base transaction fee that reflects 0.1 cent for any Asset
fn base_tx<MI: MaxEncodedLen + Debug>(currency: Asset<MI>) -> Balance {
    cent(currency) / 10
}

// 1 Asset in fixed point decimal representation
pub fn dollar<MI: MaxEncodedLen + Debug>(currency_id: Asset<MI>) -> Balance {
    // We assume every asset is registered properly. For any non-currency
    // asset we will use the native precision of 10 decimal places
    match currency_id.decimals() {
        Some(decimals) => decimals.into(),
        None => {
            log::warn!("dollar() was called for a non-currency asset: {:?}", currency_id);
            10
        }
    }
}

// 0.01 Asset in fixed point decimal presentation
pub fn cent<MI: MaxEncodedLen + Debug>(currency_id: Asset<MI>) -> Balance {
    dollar(currency_id) / 100
}


/*
parameter_types! {
    // One XCM operation is 200_000_000 weight, cross-chain transfer ~= 2x of transfer.
    pub const UnitWeightCost: Weight = 200_000_000;
    pub KsmPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), ksm_per_second());
    pub KusdPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(KUSD.encode())),
        ).into(),
        // kUSD:KSM = 400:1
        ksm_per_second() * 400
    );
    pub KarPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(KAR.encode())),
        ).into(),
        kar_per_second()
    );
    pub LksmPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(GeneralKey(LKSM.encode())),
        ).into(),
        // LKSM:KSM = 10:1
        ksm_per_second() * 10
    );
    pub PHAPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X1(Parachain(parachains::phala::ID)),
        ).into(),
        // PHA:KSM = 400:1
        ksm_per_second() * 400
    );
    pub BncPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::bifrost::ID), GeneralKey(parachains::bifrost::BNC_KEY.to_vec())),
        ).into(),
        // BNC:KSM = 80:1
        ksm_per_second() * 80
    );
    pub VsksmPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::bifrost::ID), GeneralKey(parachains::bifrost::VSKSM_KEY.to_vec())),
        ).into(),
        // VSKSM:KSM = 1:1
        ksm_per_second()
    );
    pub KbtcPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kintsugi::ID), GeneralKey(parachains::kintsugi::KBTC_KEY.to_vec())),
        ).into(),
        // KBTC:KSM = 1:150 & Satoshi:Planck = 1:10_000
        ksm_per_second() / 1_500_000
    );
    pub KintPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kintsugi::ID), GeneralKey(parachains::kintsugi::KINT_KEY.to_vec())),
        ).into(),
        // KINT:KSM = 4:3
        (ksm_per_second() * 4) / 3
    );

    pub BaseRate: u128 = kar_per_second();
}

pub type Trader = (
    FixedRateOfAsset<BaseRate, ToTreasury, BuyWeightRateOfTransactionFeePool<Runtime, CurrencyIdConvert>>,
    FixedRateOfFungible<KsmPerSecond, ToTreasury>,
    FixedRateOfFungible<KusdPerSecond, ToTreasury>,
    FixedRateOfFungible<KarPerSecond, ToTreasury>,
    FixedRateOfFungible<LksmPerSecond, ToTreasury>,
    FixedRateOfFungible<BncPerSecond, ToTreasury>,
    FixedRateOfFungible<VsksmPerSecond, ToTreasury>,
    FixedRateOfFungible<PHAPerSecond, ToTreasury>,
    FixedRateOfFungible<KbtcPerSecond, ToTreasury>,
    FixedRateOfFungible<KintPerSecond, ToTreasury>,
    FixedRateOfAsset<BaseRate, ToTreasury, BuyWeightRateOfForeignAsset<Runtime>>,
    FixedRateOfAsset<BaseRate, ToTreasury, BuyWeightRateOfErc20<Runtime>>,
);*/