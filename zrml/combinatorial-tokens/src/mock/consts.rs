#[cfg(feature = "parachain")]
use zeitgeist_primitives::types::{Asset, MarketId};

#[cfg(feature = "parachain")]
pub(crate) const FOREIGN_ASSET: Asset<MarketId> = Asset::ForeignAsset(1);
