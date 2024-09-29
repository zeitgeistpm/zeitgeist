use super::typedefs::Hash;
use frame_support::{Blake2_256, StorageHasher};
use parity_scale_codec::Encode;
use zeitgeist_primitives::types::Asset;

pub(crate) fn hash_tuple<T1, T2>(tuple: (T1, T2)) -> Hash
where
    T1: ToBytes,
    T2: ToBytes,
{
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&tuple.0.to_bytes());
    bytes.extend_from_slice(&tuple.1.to_bytes());

    Blake2_256::hash(&bytes)
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

// TODO Use macros for this
impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for u128 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for bool {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl ToBytes for Hash {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl<T> ToBytes for Vec<T>
where
    T: ToBytes,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for b in self.iter() {
            result.extend_from_slice(&b.to_bytes());
        }

        result
    }
}

/// Beware! All changes to this implementation need to be backwards compatible. Failure to follow this
/// restriction will result in assets changing hashes between versions, causing unreachable funds.
///
/// Of course, this is true of any modification of the collection ID manager, but this is the place
/// where it's most likely to happen.
impl<MarketId> ToBytes for Asset<MarketId>
where
    MarketId: Encode,
{
    fn to_bytes(&self) -> Vec<u8> {
        self.encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    type MarketId = u128;

    // Beware! If you have to modify these tests, that means that you broke encoding of assets in a
    // way that's not backwards compatible.
    #[test_case(Asset::Ztg, vec![4])]
    #[test_case(Asset::ForeignAsset(0), vec![5, 0, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(1), vec![5, 1, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(2), vec![5, 2, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(3), vec![5, 3, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(4), vec![5, 4, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(5), vec![5, 5, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(6), vec![5, 6, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(7), vec![5, 7, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(8), vec![5, 8, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(9), vec![5, 9, 0, 0, 0])]
    #[test_case(Asset::ForeignAsset(u32::MAX - 1), vec![5, 254, 255, 255, 255])]
    #[test_case(Asset::ForeignAsset(u32::MAX), vec![5, 255, 255, 255, 255])]
    fn asset_to_bytes_works(asset: Asset<MarketId>, expected: Vec<u8>) {
        let actual = asset.to_bytes();
        assert_eq!(actual, expected);
    }
}
