/// Highest/lowest bit always refers to the big endian representation of each bit sequence.
use core::num::ParseIntError;
use ethnum::U256;
use halo2curves::{
    bn256::{Fq, G1Affine},
    ff::PrimeField,
    CurveAffine,
};
use super::typedefs::Hash;

pub(crate) fn get_collection_id(hash: Hash, parent_collection_id: Option<Hash>) -> Option<Hash> {
    let mut u = decompress_hash(hash)?;

    if let Some(pci) = parent_collection_id {
        let v = decompress_collection_id(pci)?;
        let w = u + v; // Projective coordinates.
        u = w.into(); // Affine coordaintes.
    }

    let mut x = u.x;

    if u.y.is_odd().into() {
        x = flip_second_highest_bit(x)?;
    }

    let mut bytes = x.to_bytes();
    bytes.reverse(); // Little-endian to big-endian.

    Some(bytes)
}

// TODO Put everything below here into details!

// TODO Benchmarking info!
fn decompress_hash(hash: Hash) -> Option<G1Affine> {
    // Calculate `odd` first, then get congruent point `x` in `Fq`. As `hash` might represent a
    // larger big endian number than `field_modulus()`, the MSB of `x` might be different from the
    // MSB of `x_u256`.
    let odd = is_msb_set(&hash);

    let x_u256 = U256::from_be_bytes(hash);
    let mut x = Fq::from_u256(x_u256 % field_modulus())?;

    let mut iterations = 0;
    let mut y = loop {
        x = x + Fq::one();
        let y_opt = matching_y_coordinate(x);
        if let Some(y) = y_opt {
            break y;
        }

        iterations += 1;
        if iterations == DECOMPRESS_HASH_MAX_ITERATIONS {
            return None; // TODO Better error handling.
        }
    };

    // We have two options for the y-coordinate of the corresponding point: `y` and `P - y`. If
    // `odd` is set but `y` isn't odd, we switch to the other option.
    if odd && y.is_even().into() || !odd && y.is_odd().into() {
        y = y.neg();
    }

    G1Affine::from_xy(x, y).into()
}

fn decompress_collection_id(mut collection_id: Hash) -> Option<G1Affine> {
    let odd = is_second_msb_set(&collection_id);
    chop_off_two_highest_bits(&mut collection_id);
    collection_id.reverse(); // Big-endian to little-endian. TODO: Abstract this away since we're doing this at least twice.
    let x_opt: Option<_> = Fq::from_bytes(&collection_id).into();
    let x = x_opt?;
    let mut y = matching_y_coordinate(x)?; // TODO Raise clear error here: InvalidCollectionId.

    // We have two options for the y-coordinate of the corresponding point: `y` and `P - y`. If
    // `odd` is set but `y` isn't odd, we switch to the other option.
    if (odd && y.is_even().into()) || (!odd && y.is_odd().into()) {
        y = y.neg();
    }

    G1Affine::from_xy(x, y).into()
}

/// Flips the second highests bit of `x`. Always returns `Some`.
fn flip_second_highest_bit(x: Fq) -> Option<Fq> {
    let mut le_bytes = x.to_bytes();

    // Little endian representation, so highest bits are at the end of the sequence.
    le_bytes[31] ^= 0b01000000;

    Fq::from_bytes(&le_bytes).into()
}

// TODO Refactor: Make sure that on-chain, we're using BoundedVec<u8, 32> or Hash. The types in
// use here (U256, Fq, etc.) should all be implementation details.

const DECOMPRESS_HASH_MAX_ITERATIONS: usize = 500;

fn field_modulus() -> U256 {
    U256::from_be_bytes([
        0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58,
        0x5d, 0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c,
        0xfd, 0x47,
    ])
}

/// Checks if the most significant bit of the big-endian `bytes` is set.
fn is_msb_set(bytes: &Hash) -> bool {
    bytes[0] != 0u8
}

/// Checks if the second most significant bit of the big-endian `bytes` is set.
fn is_second_msb_set(bytes: &Hash) -> bool {
    bytes[1] != 0u8
}

/// Zeroes out the two most significant bits off the big-endian `bytes`.
fn chop_off_two_highest_bits(bytes: &mut Hash) {
    bytes[0] = 0u8;
    bytes[1] = 0u8;
}

/// Returns a value `y` of `Fq` so that `(x, y)` is a point on `alt_bn128` or `None` if there is no
/// such value.
fn matching_y_coordinate(x: Fq) -> Option<Fq> {
    let xx = x * x;
    let xxx = x * xx;
    let yy = xxx + Fq::from_str_prefixed("3").ok()?; // Infallible.
    let y = pow_magic_number(yy);

    if y * y == yy { Some(y) } else { None }
}

fn pow_magic_number(mut x: Fq) -> Fq {
    let x_1 = x;
    x = x * x;
    let x_2 = x;
    x = x * x;
    x = x * x;
    x = x * x_2;
    let x_10 = x;
    x = x * x_1;
    let x_11 = x;
    x = x * x_10;
    let x_21 = x;
    x = x * x;
    let x_42 = x;
    x = x * x;
    x = x * x_42;
    x = x * x;
    x = x * x;
    x = x * x_42;
    x = x * x_11;
    let x_557 = x;
    x = x * x;
    x = x * x;
    x = x * x_21;
    let x_2249 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_2249;
    x = x * x_557;
    let x_20798 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_20798;
    x = x * x_2249;
    let x_189431 = x;
    x = x * x_20798;
    let x_210229 = x;
    x = x * x;
    x = x * x;
    x = x * x_189431;
    let x_1030347 = x;
    x = x * x;
    let x_2060694 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_2060694;
    x = x * x_210229;
    let x_18756475 = x;
    x = x * x_1030347;
    let x_19786822 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_18756475;
    let x_177051051 = x;
    x = x * x;
    x = x * x;
    x = x * x_177051051;
    x = x * x;
    x = x * x;
    x = x * x_177051051;
    x = x * x_19786822;
    let x_3737858893 = x;
    x = x * x;
    let x_7475717786 = x;
    x = x * x;
    x = x * x;
    x = x * x_7475717786;
    x = x * x_3737858893;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_7475717786;
    x = x * x_177051051;
    let x_665515934005 = x;
    x = x * x;
    x = x * x_665515934005;
    x = x * x_3737858893;
    let x_2000285660908 = x;
    x = x * x;
    x = x * x_2000285660908;
    x = x * x;
    let x_12001713965448 = x;
    x = x * x;
    x = x * x_12001713965448;
    let x_36005141896344 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_36005141896344;
    x = x * x_12001713965448;
    x = x * x_665515934005;
    let x_1200836912478805 = x;
    x = x * x_2000285660908;
    let x_1202837198139713 = x;
    x = x * x;
    x = x * x_1200836912478805;
    let x_3606511308758231 = x;
    x = x * x_1202837198139713;
    let x_4809348506897944 = x;
    x = x * x_3606511308758231;
    let x_8415859815656175 = x;
    x = x * x_4809348506897944;
    let x_13225208322554119 = x;
    x = x * x_8415859815656175;
    let x_21641068138210294 = x;
    x = x * x;
    x = x * x_21641068138210294;
    x = x * x;
    x = x * x_13225208322554119;
    let x_143071617151815883 = x;
    x = x * x;
    x = x * x;
    x = x * x_21641068138210294;
    let x_593927536745473826 = x;
    x = x * x_143071617151815883;
    let x_736999153897289709 = x;
    x = x * x;
    x = x * x_736999153897289709;
    x = x * x_593927536745473826;
    let x_2804924998437342953 = x;
    x = x * x_736999153897289709;
    let x_3541924152334632662 = x;
    x = x * x_2804924998437342953;
    let x_6346849150771975615 = x;
    x = x * x_3541924152334632662;
    let x_9888773303106608277 = x;
    x = x * x;
    x = x * x;
    x = x * x_9888773303106608277;
    x = x * x_6346849150771975615;
    let x_55790715666305017000 = x;
    x = x * x;
    x = x * x_55790715666305017000;
    x = x * x_9888773303106608277;
    let x_177260920302021659277 = x;
    x = x * x_55790715666305017000;
    let x_233051635968326676277 = x;
    x = x * x_177260920302021659277;
    let x_410312556270348335554 = x;
    x = x * x_233051635968326676277;
    let x_643364192238675011831 = x;
    x = x * x_410312556270348335554;
    let x_1053676748509023347385 = x;
    x = x * x;
    x = x * x_1053676748509023347385;
    x = x * x;
    x = x * x_643364192238675011831;
    let x_6965424683292815096141 = x;
    x = x * x_1053676748509023347385;
    let x_8019101431801838443526 = x;
    x = x * x;
    x = x * x_8019101431801838443526;
    x = x * x;
    x = x * x_6965424683292815096141;
    let x_55080033274103845757297 = x;
    x = x * x;
    let x_110160066548207691514594 = x;
    x = x * x;
    x = x * x;
    x = x * x_110160066548207691514594;
    x = x * x_55080033274103845757297;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_110160066548207691514594;
    x = x * x_8019101431801838443526;
    let x_9812265024222286383242392 = x;
    x = x * x_55080033274103845757297;
    let x_9867345057496390228999689 = x;
    x = x * x_9812265024222286383242392;
    let x_19679610081718676612242081 = x;
    x = x * x_9867345057496390228999689;
    let x_29546955139215066841241770 = x;
    x = x * x;
    x = x * x_29546955139215066841241770;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_29546955139215066841241770;
    x = x * x_19679610081718676612242081;
    let x_758353488562095347643286331 = x;
    x = x * x;
    x = x * x_758353488562095347643286331;
    x = x * x;
    x = x * x_29546955139215066841241770;
    let x_4579667886511787152700959756 = x;
    x = x * x;
    x = x * x_4579667886511787152700959756;
    x = x * x_758353488562095347643286331;
    let x_14497357148097456805746165599 = x;
    x = x * x_4579667886511787152700959756;
    let x_19077025034609243958447125355 = x;
    x = x * x;
    x = x * x;
    x = x * x_14497357148097456805746165599;
    let x_90805457286534432639534667019 = x;
    x = x * x_19077025034609243958447125355;
    let x_109882482321143676597981792374 = x;
    x = x * x;
    x = x * x_90805457286534432639534667019;
    let x_310570421928821785835498251767 = x;
    x = x * x_109882482321143676597981792374;
    let x_420452904249965462433480044141 = x;
    x = x * x_310570421928821785835498251767;
    let x_731023326178787248268978295908 = x;
    x = x * x;
    x = x * x_731023326178787248268978295908;
    x = x * x_420452904249965462433480044141;
    let x_2613522882786327207240414931865 = x;
    x = x * x_731023326178787248268978295908;
    let x_3344546208965114455509393227773 = x;
    x = x * x;
    x = x * x_3344546208965114455509393227773;
    x = x * x;
    x = x * x;
    x = x * x_2613522882786327207240414931865;
    let x_42748077390367700673353133665141 = x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x_42748077390367700673353133665141;
    x = x * x_3344546208965114455509393227773;
    let x_388077242722274420515687596214042 = x;
    x = x * x_42748077390367700673353133665141;
    let x_430825320112642121189040729879183 = x;
    x = x * x;
    let x_861650640225284242378081459758366 = x;
    x = x * x_430825320112642121189040729879183;
    x = x * x;
    x = x * x;
    x = x * x_861650640225284242378081459758366;
    x = x * x_388077242722274420515687596214042;
    let x_6419631724299264117162257814522604 = x;
    x = x * x;
    x = x * x_430825320112642121189040729879183;
    let x_13270088768711170355513556358924391 = x;
    x = x * x_6419631724299264117162257814522604;
    let x_19689720493010434472675814173446995 = x;
    x = x * x_13270088768711170355513556358924391;
    let x_32959809261721604828189370532371386 = x;
    x = x * x_19689720493010434472675814173446995;
    let x_52649529754732039300865184705818381 = x;
    x = x * x_32959809261721604828189370532371386;
    let x_85609339016453644129054555238189767 = x;
    x = x * x_52649529754732039300865184705818381;
    let x_138258868771185683429919739944008148 = x;
    x = x * x;
    x = x * x_138258868771185683429919739944008148;
    let x_414776606313557050289759219832024444 = x;
    x = x * x_138258868771185683429919739944008148;
    x = x * x;
    x = x * x;
    x = x * x_414776606313557050289759219832024444;
    x = x * x_85609339016453644129054555238189767;
    let x_2712527845668981629297529614174344579 = x;
    x = x * x_138258868771185683429919739944008148;
    let x_2850786714440167312727449354118352727 = x;
    x = x * x_2712527845668981629297529614174344579;
    let x_5563314560109148942024978968292697306 = x;
    x = x * x_2850786714440167312727449354118352727;
    let x_8414101274549316254752428322411050033 = x;
    x = x * x_5563314560109148942024978968292697306;
    let x_13977415834658465196777407290703747339 = x;
    x = x * x;
    x = x * x_13977415834658465196777407290703747339;
    x = x * x_8414101274549316254752428322411050033;
    let x_50346348778524711845084650194522292050 = x;
    x = x * x_13977415834658465196777407290703747339;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x = x * x;
    x * x_50346348778524711845084650194522292050
}

trait FromU256
where
    Self: Sized,
{
    fn from_u256(x: U256) -> Option<Self>;
}

impl FromU256 for Fq {
    fn from_u256(x: U256) -> Option<Fq> {
        let le_bytes = x.to_le_bytes();
        let ct_opt = Fq::from_bytes(&le_bytes);

        ct_opt.into()
    }
}

#[derive(Debug)]
enum FromStrPrefixedError {
    /// Failed to convert bytes to scalar.
    FromBytesError,

    /// Failed to convert prefixed string to U256.
    ParseIntError(core::num::ParseIntError),
}

trait FromStrPrefixed
where
    Self: Sized,
{
    fn from_str_prefixed(x: &str) -> Result<Self, FromStrPrefixedError>;
}

impl FromStrPrefixed for Fq {
    fn from_str_prefixed(x: &str) -> Result<Fq, FromStrPrefixedError> {
        let x_u256 =
            U256::from_str_prefixed(x).map_err(|e| FromStrPrefixedError::ParseIntError(e))?;
        Fq::from_u256(x_u256).ok_or(FromStrPrefixedError::FromBytesError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("0x0", "0x0")]
    #[test_case("0x1", "0x1")]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000002",
        "0x279d7bc4e184e3a57f5fa684690c6df6b484a7f1daa1de608d266a2a4be6593f"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000003",
        "0x0000000000000000b3c4d79d41a91759a9e4c7e359b6b89eaec68e62effffffd"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000004",
        "0x0000000000000000000000000000000000000000000000000000000000000002"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000005",
        "0x2bbffb7b85b84d517b91517bcc7429cfc7fb18a7d88cfd7641df308c6d1a3517"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000006",
        "0x357d8998da08d51735597f5035c7a6c3cc58cca2a1e008b29c700e675c609ab"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000007",
        "0x1dada9100531c64cbe18cee1c3fabfe5082f0ce8505483dc5b9d6fd2be57cae1"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000008",
        "0x1ed6a916e1d82721466f07525097838fd187e5524cd1f233de2c483dbf4fb537"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000000000000000009",
        "0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd44"
    )]
    #[test_case(
        "0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
        "0x1c078130f1d71a2b85a61f5a04d5b4ff2ee5f874e9bad4d7e5bb79b1be5e892e"
    )]
    #[test_case(
        "0x0000000000000000000000000000000100000000000000000000000000000000",
        "0x0000000000000000000000000000000000000000000000010000000000000000"
    )]
    #[test_case(
        "0x00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "0x05f01ae4b9556bff71e988a7268d8faeddb82ac1f0f472e74f5e05f3c524cbb2"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000000002bacf38d0ee9d",
        "0x2bb83c2e6a71464d3072fb62139c9e13a419cf13d5b31b17f3438cdcc2ab5a79"
    )]
    #[test_case(
        "0x0000000000000000000000000000000000000000000000269fa8c16b8066ea69",
        "0x0c250f940031ee8f6e14788aaec7bb1bb3cf23fe6ea76db7ee8dea724681c57c"
    )]
    #[test_case(
        "0x000000000000000000000000000000000000000801241bb1f6295e704fba336b",
        "0x0228ae1957bfe58548d834b28463d1d98fe69e2de54b873fcf78cd3e9d0fa195"
    )]
    #[test_case(
        "0x00000000000000000000000000000000000000e132f566fa6d16bf5486f5bf6a",
        "0x2c0dbf7f0f4afe4421700aa8ed8788757b0b12b1197931b7be00281772c0dc27"
    )]
    #[test_case(
        "0x123456789abcde00000000000000000fffffffffffffffffffffffffffffffff",
        "0x21b8fe191fb7d5a2ebf018c8c52f4317a41dddcb1a1ffc4ec9141bbbc97bcc62"
    )]
    #[test_case(
        "0x00fedcbafedcbafedcbafedcbaffffffffffffffffffffffffffffffffffffff",
        "0x00b2577cf5861468ac05eb9334b380f22bb78c575cb69061fc10f2358e539a31"
    )]
    // Generated using Gnosis implementation:
    #[test_case(
        "0x2354c122221ffc8680c57566ebd1f3970afe06d53facd4117a43aa1def82b557",
        "0x1de6fa86b13267005611590911a66f775c303ae545a85e6f66356063836709bb"
    )]
    #[test_case(
        "0x24fedbc02622ac674c98380f0d80ce10c43abe50b2264d16dcd12a0af6c2609d",
        "0x1ab93efca782cd854bb5372470201592c1cd2b5f25a471abb2c7175c07792baa"
    )]
    #[test_case(
        "0x75a840f2ef2c84472c42df4f53e095d26e5b4be57078cb000f3eb90667b05a9",
        "0x29665bde39f038131074909fce09c0b794b74b1ff9ab189e8c5a4592f23be3d6"
    )]
    #[test_case(
        "0xfacd5657c56f55eed66b72b74980fcc198a2cb23043e8a725001aa0995e5188",
        "0x1d5e6d113cecf9a9c0a1896289f5c3483e1f0b5b547cbe7486b2c6b843274da5"
    )]
    #[test_case(
        "0x5fb0c7c63f31999e68e2a2bd01bc7a95d05d558acedf49ffc054d25f78c0cf8",
        "0x2bcc1855af3ab7e82dc70e97284736aecba06de8613e9e328a79ae12934ead01"
    )]
    #[test_case(
        "0x1788074e4f196b2f08fd0d2259cc15e85c378a7468002991d018e8962d23af56",
        "0x2c531e481bf64ed74c551f2a421c600d49894af8d408f136cc90418d2cc21277"
    )]
    #[test_case(
        "0x9193bffd27dc61c61c79180e65dc7774bfb8282333fe039e864042f175e8afe",
        "0x12b2834cf20f27203b810f287db287caa03c9259538a8ba65874866122373e06"
    )]
    #[test_case(
        "0x112bd6c30c4438ae381f0a2a9fad2492b87270c63ff9fb8920937611d3b55cb4",
        "0x2b0f81356115396a5a677d12ad18f13b62c626221963c4a1d8b1a93dafe478dc"
    )]
    #[test_case(
        "0x13a26ba3b2d697fc55bbecd3510200468bc0ba0e7637ce13fbf54167121f5f13",
        "0x1b409f6679274b016de55448f8ba69b77636cf75581e3cbce2841e7318e178e1"
    )]
    #[test_case(
        "0x2bce28974dcfe586fb6eea5dde6d9efb91ee7ac94d3bab482d0e73136976b7a3",
        "0x25539ea969433a9a46f52805de3243572d53a19a6d082e6908c393aa0429ac37"
    )]
    #[test_case(
        "0x21b1257812346ebb0dfcf0a3c212e4619b789713557d76ee5310240be590cf27",
        "0x18192eea11af08727c73131ea95b6885e118e687c8aeceb2f8d2b803227c5aee"
    )]
    #[test_case(
        "0x1b0ce6c54d89b9e577c5ac893d12d3cdaf71609a4bae66d06bd95353d7722306",
        "0x1b056f18f53791a773f2eec525fc09a3ad3b87963c9b2738809140f8c627f1d4"
    )]
    #[test_case(
        "0x2e2f05a7a5952517e92834acc01cccea2d45e932d9e62433e841bc5574d9cb52",
        "0x27c6cbc51000cc53e42e1200873ff5f3079eef6108afb351deb1207844e6a0f8"
    )]
    #[test_case(
        "0x2b483511c5d2445dc876806f56d96c99ac7858013059c7ccea0821bc587b4945",
        "0x13bfd6ec70991017d8d7e2ac37a08d505f0b51314509904817c2d7e7b2615938"
    )]
    #[test_case(
        "0xb09f7171c618f1f3debe2cceccb452c5f1785173fa021b1eca6b9598519636a",
        "0x9bbefdddae55c42cd4740a8103d8388dafa93cfcc68c1b1a8afdd4d22e5f411"
    )]
    #[test_case(
        "0x684aba84d1f6436011195074e326ca68feae4ef4f4b505211ca13425d3c1b7",
        "0x180ee0f41d5cb3e37a1e0d123cb605120c23a98da00739245dd29da73b7bf715"
    )]
    #[test_case(
        "0x2a0dcbb881a771aa81d622ccd5c4cc3e6e2b18b5611e460f48add75df7ffbda2",
        "0x6c9d6721042e1ccc0b3840b0c9a16a6373973aeab6ac474c0ef31896291b466"
    )]
    #[test_case(
        "0x22a235b1ffd6a6f26fbbdab352088ad04ac3a90a06a309429b90f619fe47ad63",
        "0x2cda39f4076bf3bbe07be17e4d41a2a430d0528ec17e1cb9b5d39930ea477261"
    )]
    #[test_case(
        "0x141dbc7b4620096b85b492fac2df889353e1f0ae021b26b9d3b7c19171e3983c",
        "0xe9e2f62e2ee75ee90d3ba561dc6f931b0a836f6a7ecfba9015cc41f91185a6f"
    )]
    #[test_case(
        "0x17e0b43fcba8929d9c4f275c449c2c92195e1f6a364a51a6b5ce80fb3539d6d7",
        "0x14f6886d669d73d6cc4ac31b787f55889e64eca1acfa89e3e4cde3c13f620c7f"
    )]
    #[test_case(
        "0x4753eee5c232a1b78698e289da515d5540150f9fc9d5fda0dcc5b5113cffa69",
        "0x1e1445559bffe23b8dbec67292850ebfa9135ac3027ea5de29929456083520a3"
    )]
    #[test_case(
        "0x195b5985dc2ae928e1a0d098bb04e0d0750bd17ba249a6fad642b29ef0299ed9",
        "0x1afc14672b9df4c141a9b20643434c197fe610bcd95855c8d7b9f193615ce807"
    )]
    #[test_case(
        "0x15510b2b2d41c4d41b9bf7de8ce7453ba6d101ab5b298c83f6b8d21a3ab5fe5d",
        "0x30216b11e53f919936fa4d5fca5bec88891f8c59407e7e043f156e7cfaf98ad"
    )]
    #[test_case(
        "0x12358a54316e0412177026f53af2025e7ae28a10f574a1fb2834e1e449f79447",
        "0x19f6b4c86f02e3fe5ac1c3f7356ca32e1f45c620a4b649501413018dffcf3cb1"
    )]
    #[test_case(
        "0xcd2b6fbe475e734bbf0e7be0e7b64954b9dd33a571d5d03a4d9f6906afacef1",
        "0x2cb10e299812d74d02c8c5f010fd787280f4b131f1233cba91bf9c25bf55a73f"
    )]
    #[test_case(
        "0xd77c37d1367bbefc9a5955344a0e8c38ad03808f327b0ade3fdab5438199578",
        "0x1c7f29a1a2d8fa2deeed36b97735e8483c3451da9dc7ffcb1004cad8b49f1861"
    )]
    #[test_case(
        "0x23f0a43fba6dca923fa3081b857e4dd4c4055fc9a9a0c6a876049e2af998159f",
        "0x4b4763b6047cf4930311921461ced618942c083cb81d198d5d3c6169d7e22fd"
    )]
    #[test_case(
        "0x21b2b431ff068e1e30a05f9a2570a89469f334e4d5dc09bd41fe156c4f40df6c",
        "0x1b61353cc37c1db2ff1cf5c0079e4648060ca8e5077bfb3123d2d7418624b046"
    )]
    #[test_case(
        "0x6de94f99eb1da34ead03e09a0b72bb6e2881bd51dc96dc92b0891bc41b4b109",
        "0xe294983fadc22f4e10612fc9d2d8c9b9b22ffef2986fc008764ba44c4d2ad18"
    )]
    #[test_case(
        "0x2f17bf634f48c738caba4d42e4cc2ec3334ae98b8a2c82524f2dfd17a70620c8",
        "0x299c9930426810ef78894f2ee8a47440463f3db583a745bc9a4e941b9207dd31"
    )]
    #[test_case(
        "0x241b59c2e4d782f3afa89b28b336bfe2b0848d570d300b1c41fd0ef4d3054214",
        "0x97eb9b64afe8299382fc790ed2d253ceb19463474bd01ca5b50095fcd33eab7"
    )]
    #[test_case(
        "0x2ba18996e25bc6066756c7f9a388c1a746193b12fd56e39231395e5d96d06816",
        "0xc7aee951b9b0a5c887aef547ffaec5a4e272ba627f44781c7173b1f89ef1ed7"
    )]
    #[test_case(
        "0x2e2f2bcb14958b66d51d0ad8e30d83245f08972dbe72f8d759645113150a5d43",
        "0x12136e3b37eade27d694e8bb0e31b6a563f074844375b273d70e823d990c4e63"
    )]
    #[test_case(
        "0x1e51edbf2f604da89d2ba200d6a3872f73cc828dc8b642a61e0397990f086fc9",
        "0x2bd93682d1122c1fcedbfe4403dd6f0b3488fd746b3231359f86e897c55a58d1"
    )]
    #[test_case(
        "0x1608a8e8353015ca9afd7b4f4671db355ab64b56dbee919968d4d99c8da8dc59",
        "0x112127e6f781c2abd9ff0e1b62712b16566d9b90506a5e5c68df494c552a599c"
    )]
    #[test_case(
        "0x20fe1cb977b943d0541fdc5a5bf136f3db7011c7bbf36fe6e2f2c68d1dc50e67",
        "0x23103fcadbcde4feeab37808c6c9c548e4a4320d25d7c3e14129e2d03b9a8fc9"
    )]
    #[test_case(
        "0x1055689b55c0874a419dd4118b6adada3c93ab1609c7aa902e437840e798743e",
        "0x1e52df64c4d282d9c7bbd7e6822bb3beb62776d58b317ad30137eec0a3febbb9"
    )]
    #[test_case(
        "0x233e59a9b7b15e97edd03cb365fbd860df1bf9c2204f79b75298c43022605a37",
        "0x14c976744a666aef10ef39e1dd873910789fe651f42a8398104a513dd37e3269"
    )]
    #[test_case(
        "0x1cfa85789a2c8e93bd34d5f387e0ecaf3108c8a121cc986b361155ce2ac11cb0",
        "0x277374b1c47704a063aa891969b41ac40241aac52ee9c4cf6b41770a7e2c77c9"
    )]
    #[test_case(
        "0x196851b066fdbb555b5b81175f712805dede8449be697726f325a6fad8d5fddd",
        "0x2b131a525f9630b24c2f9884b98ba541fbfecaec2499c9c57987b543543f0c21"
    )]
    #[test_case(
        "0x2b07f1d0b3be425b6e54085c2a36170cd4692a68d30f071f32cc7a73234f0545",
        "0x255085e9274e407b70d0c7fa163f2ab38b6a4cd55ec56ceb7286ea63c8991de3"
    )]
    #[test_case(
        "0x2ab94649a62976a445688157861373a3774d144203362decc26feb65286541fa",
        "0xc7cd7f76e45f7126649d448d70c7bd1db92d64dd431db6c333e89abdec4ae13"
    )]
    #[test_case(
        "0x239136e344ae444cee3dac47937e8c275ef0d97d2659d17f4fbeb86c89ea4b75",
        "0x158fd2a72745452819cf3b28ae52b3e17b8a4a24ed6d148fd04ae8ce424a0ea1"
    )]
    #[test_case(
        "0x241e0cd8d249155b10c485b5dc0b07ac6c4b96af18ff8b88c005088d8f8f764c",
        "0x23df5bcf5e7414159bf6397b93d5567ec2ae6d8245681e110527d531340f4322"
    )]
    #[test_case(
        "0x19f25f19ce5e03e807cb86dc5166e9bb6ad3cde094402a2e03f18bb15793b421",
        "0x1b1c24ee4dbdb2eca6cc36eb7aa8abd92ad4abd5ebb5d09ce543c492b435af04"
    )]
    #[test_case(
        "0x139628f946e8e6ae364c76444ece6c5903c2a79d7b647a1e4563cad636f5b793",
        "0x2b452dc3f48526d93f79364ed1fac59bb6908ddd149a51b40aff6aae2b55849"
    )]
    #[test_case(
        "0x2b4405636d4ba139c8fb6f2ea9625282ed0731d7cb7de9a6019216251d96dd78",
        "0x153224e84078d2805ac8acc14147a0670913c6bf9cfa5033bce10270f89feea8"
    )]
    #[test_case(
        "0x209f53a57dbe9c5e7771737a6bea2f390df00ee9fdc60730b419139f9ffa3d04",
        "0x15ed703c5e68539f26c73c28d98e14b87539f3ca5196b98cde2ab5fde6640639"
    )]
    #[test_case(
        "0xd491ca9d59671992cbb519471940cc5d3bbeed8a2110a611649882589c4c774",
        "0x1085ea5b92511a510a20c03a7974c08d822e9c42282dc5b024c12c0e8dd77d5f"
    )]
    #[test_case(
        "0x15f0d0433d24a54402fb8cb84a23afb09501c829c59ce3a5660a57f175bdec34",
        "0x1bf1b7bf37e25c8e1608bf5a51788a8883ada2d13d856507126b820ba7faf7a1"
    )]
    #[test_case(
        "0x1616cc74f6732135f096bb3152c31f1fd255ebb2cec28aa5dce33a3089d6fbc8",
        "0xc78bb843de8b7f3bfb5d2371c5b7a209392f9122406b2c4ca843062e5b9a840"
    )]
    #[test_case(
        "0x185d82402cc3600275d4606f9aa0bc9b35845ade09f104c5c39382550b58eb24",
        "0xc423b24d395eced3de2148bafe2f7898f3647ec17ff677ba0cef7aa6520a050"
    )]
    #[test_case(
        "0x387ef556dccfbca8df28cf43e3b59bfe2b9e2935ee8b406f2cc075dd631eee6",
        "0x152af2790bc93c155ac2598a57c8813bc28a05a29e5aee1247d7acdba73c862c"
    )]
    #[test_case(
        "0x2bd06d520de42ec02fba49e235dca1778f512dfca5922de360a88673b21493b6",
        "0x1ae0c86b7b3702566e356ac4d689bf8bb0e1afa9970961a0d6b2ef63eebb7574"
    )]
    #[test_case(
        "0x176966499fd973f0d3d0366467e6e488730930fb43890aa8b177702f3a0b05f2",
        "0x2cae575e00b58a74f3cb81e4223b175e09a7a66dfebae6d1be2a145d5eca4d21"
    )]
    #[test_case(
        "0xe0a1b86632a7a010880754e792a1b655badadfded7971485e359c1157ee3565",
        "0x11fe1c510d7a6144419b9c81b4d7039df681b97fcacb1cf4eaf4bd2e1e03198f"
    )]
    #[test_case(
        "0x18a751dbd45c179b53f48bec1f3680ec4edc4c8fefd9b1934129724c25ef00f3",
        "0x4cccf662fe8f4a260f178cfa8119870b93279ba323ee2f2dbf004376f3113a0"
    )]
    #[test_case(
        "0x294cc553274f4c1147b277763e14ded5ccf337c6438fde9e69c2c767a51b10e7",
        "0x27c095c3b1fca003c634c8933718911c0440d41565ded28ad6d38c6a8daa268"
    )]
    #[test_case(
        "0x6ef20946ebada23d09c7167b95edd86ed8eee029927ce0f581dabd9b7645859",
        "0x61b0c49ddfd7f064f4729131a4dbdfd4568283bc4fef8acde2721991c4e3f3f"
    )]
    #[test_case(
        "0x9b205a3a97f2c638aa2d2ec9b0db6626a795d091e129a37c196fbce1ca7ad1f",
        "0x9b7815c258d4b8035772e030b841ac131f32012c934488a5ea8d906152d4c50"
    )]
    #[test_case(
        "0x502f9865bf63391ae24b8e9f3916c907a6b602d53be9c7ffbcd5d6fac35bfe3",
        "0xd39cf062200d8b13abce798151a48e90b97267cf9d82f0a215160d2f2f960f0"
    )]
    #[test_case(
        "0xccf4f0a11f6fe5ed3ffa674a2ccbd2caa733ed990b6dea5a6cf9aedcfe6215b",
        "0x29d076ac9f5e6602c1ec3211b2beb0c31afb903b42e505ccba90ea255ec13a86"
    )]
    #[test_case(
        "0x147f26d61f84fd5b9788fb08e6ab806c14405687a10539651f4901cf5043b69d",
        "0x2e0f2541f0450db6f69006c3b44287941338800eaf087e4ce89ade713fbb8707"
    )]
    #[test_case(
        "0x2483d51b769934debb3a943d85e547359727728ae0347cbf787e7af2448fb5aa",
        "0x2541be86692f6efd2ebbf5a958959aaa0cb71f854405ca74d84b1fd8d4b439cf"
    )]
    #[test_case(
        "0x1e33f31e367e4e7c4dc712985e0a54c4b7de742dd4e3cd316a09c71023f9d597",
        "0x257b621ee15812e62bd9b8b0b85e3aa62b377e3871840312718fe1f3c90dcaf"
    )]
    #[test_case(
        "0x1e359f812824489acb1cba316dbfcef3d2b7751710adf91179b086b9cea631fe",
        "0x2245438b1cb85c8e0bd9612aacd2e80145333885819ac342ef5b938e75394bf5"
    )]
    #[test_case(
        "0x13f408afe13c0224c8b147363cd8536d65cf475f7f6c437061e71418543e8cc3",
        "0x68c85f58bd05d929e83fb9046fdfdbeaf87b507400df5fae0a569fb78c2c90d"
    )]
    #[test_case(
        "0x1fa5471cba8562ba144f3d5be81aa7382c45c1de82c1ef93d1b4a8f557b2810d",
        "0x2ddce2470c756ab1633f3a676e2a5145d5e262b075396c48be3c354243a10544"
    )]
    #[test_case(
        "0x2bacd1057e935b7536d34b9f068dbd77266192f8854f92575fbf2f5a394d741c",
        "0x1d4b3a1aebfcbd89f70acf083e1e267226038076614bb5bfa0641a8fb8a7ff50"
    )]
    #[test_case(
        "0x53c791f801f297d5c03099659a5e4716a3dda9157d230c82a65eab1322c8992",
        "0x15400ea5393513ac718c0a6c0fa615a4b862bc601aab1a79f17c25dd7628f072"
    )]
    #[test_case(
        "0x13c9d53ffb115abb64a72f82f56127f7700eb52cdc7a5b83e9f5e5b848a56059",
        "0x2fbe129483b47b0d60109830ba641b7c62841f898bf1b6f1ca2a8e64fc61c041"
    )]
    #[test_case(
        "0x227e0f3e939bce0986024c953faebcaf144e5269d7bf9217351a429b5f688b1e",
        "0x2939f37529d6282e8ffdb72874645618d13c2b61962570a4a59748a33eb52544"
    )]
    #[test_case(
        "0x2d15888ecfa89595f82f096ceb01bd7196022f0e8c5d00b60f630300836fef37",
        "0x2c2b04c58c6bf23752caebf5cf023c9ff2fb56e185ffeb005b48f9aba74b42f2"
    )]
    #[test_case(
        "0x21441a377ae74dfbe448db5a3daf0f0a8d0f6897646b1c07869083fa4e6671c7",
        "0x1ce46c42247a833720505d4138b198cafda1e624bd34a105445373451884c6f6"
    )]
    #[test_case(
        "0x67965676a05d25adc2956f33859e87b1e244f7e3204125125f1ffb8efbd03c0",
        "0x3f3fa9ba0f6a2bbb83ed096683279c6fc70182c6470bb1e5f5645ebeded5cfd"
    )]
    #[test_case(
        "0x83f16198f036f472d6750ef862b5ac34ac4473bd056415e3568da9d730114e4",
        "0x11e53e97c5ff41c6c6f981e5f59a7335799f83d7a71a5fb746ac0d4b579c7f5b"
    )]
    #[test_case(
        "0x1cc9fdde2b75006a9abf8457a189c5f79afdab1c6ce63137575e1b74a0508995",
        "0x3bc70435799feecdc5e7b5c2d1dd46ae076e626955b95802f69454e029429d7"
    )]
    #[test_case(
        "0x2d246f374692631611eb9a02ee4907070d28887837845257e170a030c716bd13",
        "0x193d617dfcd6392625da7bae5f313a5910a8b577e1abb9338c18614ef938931a"
    )]
    #[test_case(
        "0x262b94eb71256dd7ec1e4b2b97ee562d4b977a7a7e5ebfe7cb9c41a3947efcb8",
        "0x2f97463b4f05c3ab7f93cc98a2135530ad3a55beb5d6ee8e2846c44ebb182812"
    )]
    #[test_case(
        "0x1491001ad8736a24a82bb64075c042023cb7ba9d2d709a84147dc4ef0bef7a20",
        "0xb26973f748d8372ae5185a7a76ac9ea488dbb1366892aa1bd716260258540a9"
    )]
    #[test_case(
        "0x2665f121fa2b054d5a5f4c0929cbffd4f8c008f88910d41a6e418166e7ed882f",
        "0x25f3c1bae6c0def1650c8e8647cd768486100a513aab42b5ec90895aa660d1d"
    )]
    #[test_case(
        "0x266f2a8e026378f15e398d030b4f556e00c0479b2b13a056ccdaa068111aa718",
        "0x2339aeaf432f0a2e82848f69db7800014e68864ca9f13ad6ab1c26a634885304"
    )]
    #[test_case(
        "0x14b78c618d1fd090d251b7abd7f7acfdecd668c48ef5e64d9c59d74ebedee960",
        "0x18dc3dc2626c7bbeb733ef7a87149aa1143afd9cbc5ecbcda6ea4914beb9586d"
    )]
    #[test_case(
        "0x233776dbd1b0ae61b72abb3cdf438c041dca957eb6dc25f7ce568f4f5acac14d",
        "0x1cea4fd035651a82a4c3353d538b4310452949fbfbbf1057cdea4aa74661e4a9"
    )]
    #[test_case(
        "0x277c5b62ead9931e1fa5f2cfeaaa77281451f89912df125d239ada32aa11f63e",
        "0x16feb53a2ae546c9a663cff07e2b5ccd5c69a164df39b9620f10f4353279cd5c"
    )]
    #[test_case(
        "0x16e9a9ec7ace195477fb7a08376a4f586811dd5ed1415f2926e4f3c78ebbe731",
        "0x2669afac1014727ca473e0d88a0e1b1ea6ccb357f4f3949048ba7f22c81c14a4"
    )]
    #[test_case(
        "0x2519f5da57a3f30043801ebe62115213cb6cdcef84d747ebb93628f2698d50fb",
        "0x2e8ad65f5845ce324803a66e4f2110a60b0dfc014272a5097b61ca58ccf519ea"
    )]
    #[test_case(
        "0x1aa1c72105370ecb28eebc32eca71a19104765eeb6f711cf5235eb87692de0d4",
        "0x191acf349969702acdb8e963beb15653d7371ec34a02590f65d4c635487540"
    )]
    #[test_case(
        "0x2a2bc990b5bf71485a4a8dee0683d123f5f71e77623ca07a30ce1e2c2676b167",
        "0xf4f32804774aef11a3749dac93e5f5141531bb30b9d8b8d3912cb05b09dbbe9"
    )]
    #[test_case(
        "0xe3c9c824a566e3432c02c284fb2f8363db4676cefae2e6355b24996908d2b8d",
        "0x1b3fe8d1eab6c222c54d8ff4b958ba008de1eea7e319c3249fe55ca068228270"
    )]
    #[test_case(
        "0x20c2d4a3019f45085e549daebad566d9ee4a4a429d1a928a244aebfa4233d5da",
        "0x132582146ed242a554750ee95788acbc2c3172594f3e834b033b8e3c17540c05"
    )]
    #[test_case(
        "0x2fdbf1efee35c9c7d94962251ccb9604b7ce53d89b144f0c5729ce618642ca94",
        "0x10ba762fe7f91fa7e1476a22e56771e22c9320c5b21ed1591db78a630604e9f4"
    )]
    #[test_case(
        "0x189352877c36efc33ae7c8f2ea974cd280a5cab22d5b14fc976d4115babcc09a",
        "0x7af3cdfd3925232d2298f65706cb4e362d8ca5ea8b7f860b6bcd04d9373d938"
    )]
    #[test_case(
        "0x206ba61317fb7ecc200706e5c5ef40d5422258e13728c0405ebe59137359ab3a",
        "0x1338285e79deddc57e25bb8b69f0c71a993e81690fc25505a2a4957fbb1c62d5"
    )]
    #[test_case(
        "0xbf70e0026bd8c03411aff5786b34da4079bddc3da0e957a32f27422ef3bed53",
        "0x140d99cb25901d245ab6182665981db23f87140b578af38a4805ee43571ab347"
    )]
    #[test_case(
        "0x15b3a542d0cc7cef48ed98178f2bb9540b663afaa3863e1258e3b9c3900bdbef",
        "0x1a3e34051be5649640f1d6658c78f2ad219d5b82155b185da0d71663858e4f59"
    )]
    #[test_case(
        "0x84b12f410c77939d561102b2b25decac4bd6b4c1caa0d93a212ffb7b26ca4d1",
        "0x28d08e65568558d8d0edc3750ff4d8bebed2f022a286251ed87902744eb1455e"
    )]
    #[test_case(
        "0x186384bd9f5ba026ac6fae1bf15ee4bd88268751941f9dfeefab8890080da767",
        "0x101f9e607da2d440390faab2e58ecaa35497c61c410d44a6b3cd9292f912a7c5"
    )]
    #[test_case(
        "0x362c451cb3b9d33292eb061e801c5a98d50613a67eb5b3b273742d81dad9273",
        "0x1f115bd90d72204281efaac2afc5bfd4c5efe572267b13adc7e495640dc61c20"
    )]
    #[test_case(
        "0x1349e6cc590e4954fc551e76d93ec5d6f342b5d7a9195db47ee63069dbf74b2d",
        "0xdb453e6389b7e43027d0bf4c45b114bab8615d74860f97bb05271f5d0b5a2f7"
    )]
    #[test_case(
        "0x10e2c61d7e369db368a05f02a2cda93c9c5e9a330f084a628cb1fb53c6d255db",
        "0xfb1b229496d73280c1c156ccad9c10ebfeaf1d62253b01d58fd60bb2f68caad"
    )]
    #[test_case(
        "0x1500111ac021fd81246012e138b9eda8aa616939892df222a7e2cfa9826ccd9a",
        "0x1778361bc74efeb1525c906ab0fe8a1e3389b9334a38364ac88fbe8ce2df14f2"
    )]
    #[test_case(
        "0x2501fcc7cf66f7855b80a99f61a1aaf4da79f15e733b5c162aae779cd899f7fe",
        "0x1ee83d1fd40c18ca194aad652d0ccc646c8fdba07f96a8a3dc8811c60f5856d"
    )]
    #[test_case(
        "0x22f86b046a3061ec416739cd769d1ee88528134d7efff80b380c3c75d965cd7b",
        "0x2ede9051c70b409f9b161331c78de40f976a76e60b7ff762b6f0aa874af71979"
    )]
    #[test_case(
        "0x257b9a3b85164b570a3acf910ee5ed3a42723605c7a3a20440f6ede770e19633",
        "0x1d9b106156c3279a618931fbddc913e391e8caf2d818c4aa914b35bd88bd19aa"
    )]
    #[test_case(
        "0x300bf0e4501d17fbc102d3c3e70fed3234d048f7648616e49b4b23ef3cb4d51c",
        "0xd498e81ca2577112e6f6828ea8cdc3ca354e282224827349b76f1dd1bbefe64"
    )]
    #[test_case(
        "0x14cfaf7e94abf742481f345124713140d4d1342ae8a962baa3b7f65547ec6924",
        "0x2889f002b35b78413c8441379aecefc0c3ff55905bc2d719cb3518a079bbe4ff"
    )]
    #[test_case(
        "0x16a851fa9cb2a27bb2b5bf453ca0becce53162bbfc263677f8376e8c2518b225",
        "0x14d2a53d0871577ae830d08d38b66209b72891becef5e1ed23b016916eca5ad6"
    )]
    #[test_case(
        "0x21de9280f09d946498268ae02b8d8d81bee8903b591b92cc3def80aedf90db09",
        "0x132c4d2fa3356956754f4f1247c856bfe76857329a317f663217565a7bd90334"
    )]
    #[test_case(
        "0x16a09bad60267721122fe1a9d729d9baa7706c9f61fbef5a4d6f5ee36ab4ef13",
        "0xcfe2414cef63b161d8139fa85f26a86cb2a00db95415bdfcab29a0211b5356e"
    )]
    #[test_case(
        "0x7b4de2295293811d707f2190c63f72391200355d62dbd59d426b627842d8cfb",
        "0x3f0380d3101465ff3b5143a9d3b93481a74751e737a0a79e1275fa2f6695591"
    )]
    #[test_case(
        "0x13bf8783d8204af58937db2e397c5ddb61e6caa7b84420fffeb7e759e4467323",
        "0x7057404a9219fbd71ef1b308731c0269e601eae01652305c0f3e7aa9efecc2c"
    )]
    #[test_case(
        "0x993e08d75da08185411f995c20e16745ea379dfd4185f73caf434cf650f5727",
        "0x1f7559dd91a5c9b45a4dc73240815e2b4ee6e6fd824dfd67b6d743c81fe1e27d"
    )]
    #[test_case(
        "0x2535ee3ca4609c3f34c03e65976dde76028ebda7d2a1c9f3c7bf7703e03ec958",
        "0xf78b186166bc33c2d1123138474ad59dc6ff68d3af40c06eddd3a0664a983df"
    )]
    #[test_case(
        "0x136554371983ef155879777e38b94fa8d1760dd98cfd7107fd2c85edcb2cd405",
        "0x72cce4cb9def52526b490d338d80c0603778409b5bd95513b2341e2744af1da"
    )]
    #[test_case(
        "0x4ddb9c2c28b29dca5d89fd38d9fde22e416a8bb9f6b3815cbdd62b226ea6f3f",
        "0x2893e6637a84cd1355d8ed026877d582a1734b2798eb90c2b2245ca7c4919996"
    )]
    #[test_case(
        "0x228f58ab1cab846c87211fffb92053ef2c7efb0406f8e0078ca56794c8c7bbd7",
        "0x162da3d4f6f9bf7a7de6d4100bacb6d9cf26e581a542d289e5048e153a478842"
    )]
    #[test_case(
        "0x298a47d0e457aef872496d0a6058a20c0d0de976f4abcfeaf77c7ba62a6c85f0",
        "0x97ed3849361a48adeeda2b1b965f766888bd5ea86e2bd2c1557d9d6112cc9ef"
    )]
    #[test_case(
        "0x23461d9d06998da33f825a43a9d9404683276247982cd246804a95541e4b43ec",
        "0x1ae2b0c60cbad700eb49676a1c168426a100fc3f513c56014abcf2e3738cc3c6"
    )]
    #[test_case(
        "0xcc026e32762deb9eb877f8a685f8e98436374bb63f890bb68726dc0fd55c7c5",
        "0x159fb466e83311f0beebde81342ee8053ded5b305e2ed7b52137dbcdecd6ba9c"
    )]
    #[test_case(
        "0x26fc751063de000a640f48ed75faa36f28ea42602d10b8bf929cbb2b31637e28",
        "0x110d7550473d227718817190b68e5782f7cecf94884bddedff5bfa3ba73c33fc"
    )]
    #[test_case(
        "0x25d5773bc558d40a75f23b7a5599b7ac6fd00baa41e6e17a6d860621d0aac597",
        "0x29421c8b0b99e5fecb154e2c297aba0b622e775912c4ac23e008c7afe42db0a1"
    )]
    #[test_case(
        "0xccdbd8a83a7e3345d6dd502b4977afc50195b586a2eab7e5c8acd96b77af1b8",
        "0xc862bfc4d21cd661a2cceff3d9f07a375d264b8586abb89e2e174fbbccf022b"
    )]
    #[test_case(
        "0x8e132b417a8197928d55e603e7d43103aae1dab03e386a816ef813caa44f58d",
        "0x9bc7fae5a35ef284c1aa104fe83f895521a47f26046fb0217629afd42d8c5b1"
    )]
    #[test_case(
        "0xaa22fabd917e5a0de47b98167f65117a9e445f2dd15b2f6207a832d3aeac1d9",
        "0x2180f9dbc447a10449a847221daaaa3078bbacb3157d2068d9acfe478c205e15"
    )]
    #[test_case(
        "0x1bcf716ba5418321de4bc2729833b9a6eca98cac0f66a66748f8dc95ac847425",
        "0x252498df4065a1ecab39b386a73f5ae77b0099408c9e5178cd45f0a88f140f46"
    )]
    #[test_case(
        "0x24bc31040e89ac8640f77510f7aabed0f62c33f7e174cf23a8e5c125b09e5ce7",
        "0x26ff5d9eec53a8147791d60dec7334f0e15c0c1562bd39a5507b25a55fdc1117"
    )]
    #[test_case(
        "0x47bd7a63bc81d5630fb6947a7a997b8f71810b21b76246553fb8abae24ceda0",
        "0x86b905261721192ae90fcb738ede465b021e6f21e131648213b9c18e5f27a4d"
    )]
    #[test_case(
        "0x216a2d2f7c18050abaf32c9a8d0c6e6ebd37d30591dbeabba69f19558be37fd9",
        "0x229782d104cd55b2255bc599d7903eea5ee45f2a6361d8721487eb8345e0e495"
    )]
    #[test_case(
        "0x82604fa7c06b4e38e62c5e7eeb23728c4abe4ffdedb56d2a1c0072cca87f5c8",
        "0xbaa0693f0ade943fc7ba01deb817c6b7388abc36fcea8996f0da5e737b4c17a"
    )]
    #[test_case(
        "0x2a4bb9a545b055e62e92b6d00169dc4561eefc34c6cdc287bb6ed139fe180b89",
        "0x8116d95da4da06976f4ed7acd0a84b19f45e9be22f13a9cb334c3b4fa5e26e"
    )]
    #[test_case(
        "0x22160fe0f41fc35a7bcd52616c3e618d2b3070ea58805938250cadd9d02b2c8f",
        "0x7983df4fcc4b04301be325ceefad2ef40ccf1620f551881e1cbe3d512376fba"
    )]
    #[test_case(
        "0x2c133130b0190187a11ba3e5fa520aaadf5ebb470aab4d4e386b007520ab50f",
        "0x20260047fa8d1fad6f5e75d575e4c7d9861fd38d2fa9f3da7d3b1cdf73fffedf"
    )]
    #[test_case(
        "0x22315f658108887f29c99f4876b62b0d40a6fa3014274db2d04831ca36179f15",
        "0x202a1b12731bb44996d4223e623a0507e0c2f91140bc0655454c2e87f594b1fa"
    )]
    #[test_case(
        "0x2c4d0bacc44d53bebc8f0429f39708adfd43bf145155705721be0de3460f94bf",
        "0x57a992098a1e36d340b17d33fb89577108adb827d571f9c6d9824a6923217d7"
    )]
    #[test_case(
        "0x11849bca64f56566527fe827288dc6d3540e7e929efc5e4d7eafd509f095305d",
        "0x13813f285b8214856bc99fbeaa441c6838be9f4e5fb6971edf0b22364395b7a"
    )]
    #[test_case(
        "0x18ca311fd3804a43a56ce7fa94fc7052e1d56437314dcdb03b3151c7c86331e3",
        "0x1d7a1d018f905cd4212c8d6456eba53f7b4631eae6ac5f55051e16e11789afc0"
    )]
    #[test_case(
        "0x15748ac4dbe0a797ab1eac7959e1cfeb804d04ed3ed8bc9accacb643254db99d",
        "0x5c863ddf01e2e60916fa54489a3f7f90fd8dac5a39fcb07cdaf293756cfa8f"
    )]
    #[test_case(
        "0x27a611ee7aac4b7fdd20edeb5878948d4c609315b9e33daecbd89356db8d0a27",
        "0x403f5943ef58f9eb2de53a52173b4a91d24c90c3fb9cd55a554264d0d1ee874"
    )]
    #[test_case(
        "0x2d30e8e7214f50b604fb598c1efaa457d9db73de8d4cf5c7857265dcbfa581da",
        "0x1576cfeabe6034354bcb1cb32dc405599a803571a1a66bb5e15516c2c78e6606"
    )]
    #[test_case(
        "0x20711e31b7b5bc43d5ca145b9cfeb0be30c41edd881b1d833f2e54ee0d4be0c1",
        "0x1e8e340529e0d63c1b0a997e59b82bad4a94ff335b5a5f8a5a3b213014e924b2"
    )]
    #[test_case(
        "0x18c287dc14fe88b0adbef8d1ef2d58d422d0fab8de961340ceb7299ad7fe636a",
        "0x1af4d9c17247b56835cf26e29827ad91d1938f1605c5ceb3da3e6caf12ea30d9"
    )]
    #[test_case(
        "0xc2cc6f790940d65de04675d511459d0bfdad3a43d9a4c4677e381315c1452ec",
        "0x11288e1e3b0221d1aab000997c79f01eff73965e6fa18386b4b0bc265b2b7d7b"
    )]
    #[test_case(
        "0x19b3db61f6f3630bae4dc6b48015c8c5f467ea3419711bf0b5aad428f0a16802",
        "0x1fcc156cd904fc210f7b78a942e12e5fc3119dcd2a3dd82b498cbb1cd73089e"
    )]
    #[test_case(
        "0x21c92e918870e59b6987a256d184f010dd02adcbe2142b6ed4243019b7b9ab30",
        "0x1316f508bfcb400cd688f70dc30fce11e54f22bed5c31ea40c91800cc44ebde8"
    )]
    #[test_case(
        "0x624af0698ca0f02b1a2ed852b238e8b979ee621b223afca4f945aa569657167",
        "0x1ca5e2f0e4295e0149e5ba772121e9ef3e2fe0ac83f24b4aa8b471b40a9019fb"
    )]
    #[test_case(
        "0x22a5e74c6f717869bec28f8eac0e2f0c1f2fc5a1d67e797e2d89e2568710b94c",
        "0x2283ee08f3361ac672bcb978193c158f2af5169532cb3a612fc3e16ba070c077"
    )]
    #[test_case(
        "0x22cbd852b1135aed92cc6ccffa5aa346a0cebad916cf6fc93407f799c7f0b51",
        "0x1e48d46617abbe1221b75ebf92d02715bc7a5933239be0e1c154ffd0a2d6ad2f"
    )]
    #[test_case(
        "0x1b4ce54781392eeaed00d156eb301df99aed84044022acda6ef3880971f785f6",
        "0x22116f6d5a5056ac8d6a94b0944dd5b6be597c3def921d89069d21b5bd567e63"
    )]
    #[test_case(
        "0x195dad8e80a40291befcc391d6d90d96ac5da23517bf785bd43f9a694d0ace34",
        "0x10fa4dfb4cbf444f0cc15afbfe546e3da23ba9b31ed189f5bb788b2d2406abc8"
    )]
    #[test_case(
        "0x1b079c1ec1929783ed16ffbe1b3ce3cf799317cf50b9ccf4c7341c50ec003461",
        "0x1011ada74bb36a88828d810855f95725400d5c1f96bd16c234dc993a80b55f8b"
    )]
    #[test_case(
        "0xa79cdb7dadb55b1b2c86dabe430652ce2322793aa1ad3b8fbddc455b7b5bd48",
        "0x277c82ed5154c26d4b5005c0167a8824f4c4f78bd3ad73daf818f14551e5e235"
    )]
    #[test_case(
        "0x22b1ce3c072298900163dc4c8ef37c8abf6045bd8eac512ee84973723a889413",
        "0x18b349fa56cd37819656b1ca9870f3c5395c60dd2ff30812b02e865b88e1101d"
    )]
    #[test_case(
        "0x2d0c639d5a35a1391dc74e6f33152c898e7a963eaced5ebe7bc1ba283b6220a6",
        "0x1441b4908bea213aee6d1a8d798f6bf7392ee0c854faa12d71c918ae2c68c917"
    )]
    #[test_case(
        "0x2c55a5feedc228b663424fa4779406f58f90e5e0c378b14a01e0a5f99ecc3768",
        "0x229aaa49d261a890a9408edf4e53f8a807d107c82720d5af9c447adbd3e93df8"
    )]
    #[test_case(
        "0x2afcf0984f35aa2c805cdd1dbfa56c9fa869a32325115e633d855a2c4dc7bc7a",
        "0x22afc3e5832384badabfe1379d664ef0ce42504f1eebf4652562c12ff5fcd257"
    )]
    #[test_case(
        "0x2bd3c2c2b03cbb1bb9ea88c701e5331850345cdac3943ea4539fe18019cbcb5a",
        "0x1bbd5402cc56aa14b613632c83b44e17f32ddc90cb491c4fe36fc3db693870e3"
    )]
    #[test_case(
        "0x22b98fd8367db019f6e709e9761546868e1bd1b939cb37207ad7d2f4e61af6f3",
        "0x158196138d686cb065500e993011921b5a5e6b205aba5a1a917b61c8e6b61cfd"
    )]
    #[test_case(
        "0x2a00ef95a6225d666a5d0589b6ae18d3da939728ac6db46927f710a5902f46c1",
        "0x1b9d6226897f1ea4699b28dda521da10c0d1f928ba596ad25760dc701c2c4bf4"
    )]
    #[test_case(
        "0xaba72bc0da80375377698c6ab06913f5a1142eff7567e05f8de904f05399e2a",
        "0x25dc3060f58c0ce8180ccabf0853a82fdd59435859ca08a9bf8f442931b7c053"
    )]
    #[test_case(
        "0x17214d772a126a4bbe0939b3745d0741c333a1492004301411bae7a8e2ca5ef7",
        "0x208f48cf1b85082d8af1244483cf65bc3dd4a59f0090870be8892606b3dfad82"
    )]
    #[test_case(
        "0x650adf2d70e0d7b6c9da45b62125447e93f5af95674ac637b766f3a2d7e3b61",
        "0x11bf807322cbcf29edde405d0d5674b15e17c0abd160049d9b32b8bc3ff8aff7"
    )]
    #[test_case(
        "0x13782e4510c3427a26c6b1275e087148048bf7c630fa008de9aa579bac954dcc",
        "0x82821d57bf038dce7610450ddb72115b0371abd588dc00e85c49f09026e2553"
    )]
    #[test_case(
        "0x268b3c2f9b1c3f9fb23dfb7e3e8562170ecfd7b8b22cf37711cf82a5a3a1256b",
        "0x38495a00c1ea10a49f6347a1649c1f44e8000be59146c2d01da0fd6a699d48c"
    )]
    #[test_case(
        "0x1ff54deb17fed0d2aabd02bf5f96bd1c43c8a149e68ccc4f58c09fb1c45a39a5",
        "0xe7ba15a7898cb0b1aa319c2aca08a17e3cf3a2975298bb9e18a51e3e25192be"
    )]
    #[test_case(
        "0x1c030143aa7bedb029228a35e42e35e750b97601d674bd9f40595d8652cac93d",
        "0x1dda16d607c3571724edf8d8ac519fdd27c8e2fda0fa3f1e975b1d965589011f"
    )]
    #[test_case(
        "0x2239d7a1669c6bb9371b76d4eb5da03cf57b0916f6aa88e0646fecdc30deedd4",
        "0x15f84eb11e496cab754210bde053404f5b9117ca8c4046924d24969c29693519"
    )]
    #[test_case(
        "0xd6d12c3ae70dabba3da219d3e917b1cda174ee015cff2cb1a89991dcee456a6",
        "0x18dea13e72aed477445ee1649c506306aea918b1f05c572248827fed23db6c7b"
    )]
    #[test_case(
        "0x38de326bedaac59a687789272a28fff3d110f20c037306f7232791c463b90ad",
        "0x1dbeff41063016afd8b6e612eef54b6f02aef08e4aa527efe5bf4c908de1416"
    )]
    #[test_case(
        "0x1c1fe12683d8d267c934bc0a3254bcc8056e9cb9e969ad170e3dfaa8e5586",
        "0x183fd1dd6df336293c7ad0df4c4591c582103710ad9fdd20af29d2f00f2d8829"
    )]
    #[test_case(
        "0x243f3876808e9351c8bec2a03382141c21e7264ddab391f92859c8acbcd2d06b",
        "0x251e6a8eb4a0cb951331a5d4d311af0cfcf3e5e8ec38d8ba5da2616f3123f6ce"
    )]
    #[test_case(
        "0x2a33a6cb3f56d63941e7feffa23ee3c4d442849f0fae2a35126c6c8e983ab4b8",
        "0x25348748df332a5914ad1c5b3898683fa6d4413eeda26d9f37ec427dba5979b8"
    )]
    #[test_case(
        "0x1a4c2fc79b31fd5eebf158a263d367de37ff049ffecd6dac2f83f68f2789ad67",
        "0x1c7a391ae6060b2325d04c1afac3b2d9791f405e8149077309b6c61c33d1d59f"
    )]
    #[test_case(
        "0x1ba4663350cee7dc26b4b77acc236d62b1c220cd5cb021524fce492632062322",
        "0x237fb10067864ae6faa02aa24f13a92b8609d9617404d971176390e6e22057c9"
    )]
    #[test_case(
        "0x29c9cd26d1b2711cc25abc1631fd25b2cb3566954e818db4b243aa7c3f55269f",
        "0x15aa87b2b641244ea11090e09c38f46fcff0790c4b7d4881d60d1963ea7a7384"
    )]
    #[test_case(
        "0x14ab2b813699abc3ffcfdef130f3a7d1de942db1060e8b942de7a5b7455f5579",
        "0x12ea09450cabdc795b0eaa71106f49eabf27f617311ec4934f88381e778ca36"
    )]
    #[test_case(
        "0x261d723c8f3446e0fa887e3994c192d36a9b7ae89bad278d641495a31707bdaf",
        "0x177798c723be2859188df10cd6bfd6d22900fea5bcb17a8269c47c4ec57ecbc4"
    )]
    #[test_case(
        "0x2415743f394b16da0ad801e5c0458816fe6879e2b104e4b932d25bbae1e4f495",
        "0x21bbeef0c708be4177e609a980574acc4ee35694f7192bae93267480523dbfa6"
    )]
    #[test_case(
        "0x1e019a2b15a59774495646a2c6e90af5f9513855e8790fda0410d821aa85c77a",
        "0x2499a400283616bb886d1a50fae5aea968f2a9ba569e6ca66b3834bf24b5fe90"
    )]
    #[test_case(
        "0x18e594f40bebfd4378559de7cfb8134792f54201e6f97f13eeec495db4b72043",
        "0x2a05674c906911dd4174eff658b723d9cd196cfdb37899d5d5dd64033af8e049"
    )]
    #[test_case(
        "0xb026f975ba448d1862b063999cf926a0926b149d2af3cf4174e56e060b1e40",
        "0x18afdc9a34dfa214e17f298a56ac2cbf8e6da180fa3e83ab60f3310b5574f223"
    )]
    #[test_case(
        "0x162b6cb9b09cfbd3eddd0665bdeab6f7696e63e57c883d74160b360e55bda75a",
        "0x1dbcbbd18641d2b477e19ef0186c608f0e2775a3e80c6d12c30b2df6920b869"
    )]
    #[test_case(
        "0x3053a306366ae73004adb183c86cc3a3e6b0e2d317d08da708c9e5731927d427",
        "0x12a78b1fb6535610e34c60342df9aa6b7deba119bb371523b2554ff2490cc63d"
    )]
    #[test_case(
        "0xa257789e2277e79f78bcda818dbe60264be4ba74a7f0e5512b77ebde009ab2",
        "0x1ea57d595383619dacb68111240ba48a0fc88aac272c7c99b6d6bb52a28fa650"
    )]
    #[test_case(
        "0x13c2c6e174d814c5d7eae609741101a78a451faf30d8c4ada0ddb50c2c32bb0c",
        "0x37e2eb482c3ccd3db5eca84aadc20da4a7f62709892006753382d9c7d7efdd"
    )]
    #[test_case(
        "0x29a1b6724661494c7d92de23d383dfee36bcd9a50948985fa15afcdc9fb4eefb",
        "0x261bc9b7013c833ec7a28ea5ef230053165f00baec582163d1773fad0344e793"
    )]
    #[test_case(
        "0x271d404a15841bfa3649f393f97e93740334697e842cdbaf6a506672cbb33ba1",
        "0x1caa0bcbb5781345da0a936ce4fe20bbe70292aa5d6875ebba25cb901b1a8756"
    )]
    #[test_case(
        "0x168dc2d2fc6ef1b60935ce41605391fee9d8293bae910b398093c645e352ceac",
        "0x2e2cf3475af52022033343ba610d4648ee9d8e929f1156ee58422871973c0f02"
    )]
    #[test_case(
        "0x697a67cafbabb021ac4e263bab1a3135b8eb0510287e63afed023d10e68faad",
        "0x124fef2af38736220515457ef35a016ef1e93d7a07301a7fcab4492fbcb540d6"
    )]
    #[test_case(
        "0x2d1a15910aea25812f7a713b58f69c67d1c1dd53afefcb469ce5ea550c63a0e1",
        "0x23437cd1744acee9b189947dd7df550d86c9177928c1da7d29b42737b9f753aa"
    )]
    #[test_case(
        "0x1923b06c682b6ab159e4179d06f5211606291fc5dac808d96ee9d92e06e8c19d",
        "0x19ac34336363b25e6a6f3689e311f7ee402685073aba9751c677fc4fe5d37a67"
    )]
    #[test_case(
        "0x23a31f65ff514776538165d29e89d78300cca5372e9846c8b753d3f9f8044be4",
        "0x92804e5af8af3bd1f48ac2385ead58df2d844d2f734054704cea69463b1347f"
    )]
    #[test_case(
        "0xf289591fdaf5296221ad7242f5df12d8a76043909630dbacfbac0a855fb0adf",
        "0x374b18432af1867a8151a59b22a1825a361f46f17d5b1eec617d2436f41f925"
    )]
    #[test_case(
        "0x16996e0caa62e15e8a84366e2cf51fb63c08b99be3bfc0fe611c15b1ffdb92d5",
        "0x35eea3531ad25b1b30965c1c28405def501afc7f05e77cd4420f3b5f97fcab8"
    )]
    #[test_case(
        "0x43eb07ab2d420f827b29288f5fc7d35c419690cf49555d779061809d652c0d5",
        "0x289a0cf358c7694fa5ededf12f69ad9d1fe4135fdd66c8b24b98d4149ea22f9e"
    )]
    #[test_case(
        "0x2c9d3863863bc9575b5674d0d89685e6c003afabde723d0d68de5a3337f34dfd",
        "0x1a71ab341620d6ffe5b16000cbd4fc618bd1970a824239c6d45f2c25bb99eeab"
    )]
    #[test_case(
        "0x279dfab7a5b2953cc11002895072278fcdac890e8ada0d5b7e51fb2471b92cca",
        "0xbcee4169b31286d0e2fc40fe9da800ba7b4949807d1ccb4538e99ef0334628"
    )]
    #[test_case(
        "0x14205cd4096bbca14a1155dfc76ceaed08d864aa680f2ff47cdf03aad0aad0ff",
        "0x11501a4f908f5cd18c3920b6d0604b77391e2490fc92c626eeaf5f88201c4f57"
    )]
    #[test_case(
        "0x1e0c56db882cb1718ed784d36b33fafd67bd7a836fe7b902e4d140bfe0b5f603",
        "0x2f6b1576269ab9c5d9d6958c068f3c1e23b120ab08a1c9158d069e283597dc06"
    )]
    #[test_case(
        "0x4f83ad44d04d568c28ae648ee6d61ceeb439c38e4bac0668655e623aa2519b3",
        "0x1438f4f055d8add1f00cef9ec83e1cdf4ea3c19b31ecd5939670ea6c005b0e60"
    )]
    #[test_case(
        "0xd136345c46ec42cd307d41cec11479e13f9f63a5b417c451d508b430eb70c59",
        "0xb41a0fdbd96f5e6641dc32030cfb727479824c38cd155b8a2a5b3985de8ca5b"
    )]
    #[test_case(
        "0x21d0e540338f197962ca54281b078cb0f2f1377e2ba8af4f4c317650cbc8474d",
        "0x251712ec63d4586c2f626fa7c326211c1008624a7137acd58c89c749f3c45910"
    )]
    #[test_case(
        "0x1a48e43fc78439d7830c56e6ad10939f412e49dd5bb0f91a0e6c691207c84fd1",
        "0x28d487cf2e89c9ebcdbde1d36c2880fc3cec52de2fe81e24be84d7c29edba734"
    )]
    #[test_case(
        "0x13dac95faecede510707379cc3d8aad5667c55ab22d88c541d92d5029311ec6b",
        "0x8da8cb35003282b91955580d729d83bb24da6c3e2329a9b3f59946dc1101fd5"
    )]
    #[test_case(
        "0x1005c9ba92045b0d2c091823efceb3e94c1309384093248fef60a7e4d214984a",
        "0x2a29050ef5c587f114c022906469727cc992065ab652589bbb602769cb624a1d"
    )]
    #[test_case(
        "0x26f00b244a6e09e33f340d3d9d9c70fe8bc8605ceba4ff60c4981ab964291cc8",
        "0x61b7ef308917ff7b6546b4ac579d00a2d23c4c6e7148ab0a76f0cb58553f992"
    )]
    #[test_case(
        "0x5ec65154b527ccee4fca4352dd70ee92af0c2acefc59efa0c3a01cf76666eb1",
        "0x71c4282827915fe5aa46f7cd29bff6439af37b8c36626ef163c1632bbcd627"
    )]
    #[test_case(
        "0x141d86e8db4e7dd156f37ef1b3e098397b91a16a7a958466ac3c1a4b88e93d59",
        "0x676f6e126cd32af7b2c04f5344b3e61c988d78ab477825566831d9c65003312"
    )]
    #[test_case(
        "0xba149f9370ab0600a294405707cf09266cebcc502e7a45ac3981ce267f91e97",
        "0x110c91fd089137d7a14d413be56177a5a16624965c4ca2751b16357fc70f69f2"
    )]
    #[test_case(
        "0x2e6f97e69d5118df6067cfcefaaf2911a823574180e7d33e81b614ac3993a31d",
        "0x53e6cc4a2355704346053c7fc1e4e019975a4c1917fee4d2099ecf9d9bee025"
    )]
    #[test_case(
        "0x1bc8fe8ca3b70b89b16c92588d9806b56d9ec5f800a6d8c4c9976de96c7276f9",
        "0x1df64626b2aae1f91194590703bfc4ed4f9d3fbe8b0937f2c7ea5aa5c5376d0c"
    )]
    #[test_case(
        "0x30428f6f569ab3465ef5ded3a0fe6d85622a4dfd4939749c5c3fb9d309a08a69",
        "0xfe83078ead3d36826a82ba1f0ed693b81cc56586094770fe3b3a9b18db47e60"
    )]
    #[test_case(
        "0x16c1f522802c3245334e31fb458f494b6dd60039e788e7cb24881abfc9c2c58",
        "0x1daf898f07c8391c96339014c8235d424bc60f0df2da2a0286520c6e01c2e899"
    )]
    #[test_case(
        "0x13ac03c94d96332158c4b7819cd271b032e9b88877284a2aca9620ee38e22ed9",
        "0x14ed68b189d663a58373656cd3944a5f6d0555fd4bec19b5605ef575b1424851"
    )]
    #[test_case(
        "0x542dbd7469f5c8501461055f3194ea70bbd3bdc5eeec2c5ded7899252799046",
        "0x236f13e247951025c8f2324d469dc4ef4c2a9dc39c8fbaa0be30290379cb49f3"
    )]
    #[test_case(
        "0x236078d5c1900bd459906b8393995385bc92ee0d53ba0464110f535cfd3acd60",
        "0x2f90c6c26b7665eb35d95212f19f24c493596cc81be5e7bf27abb6cdb86774a4"
    )]
    #[test_case(
        "0x1633354c0f110a08c8f587902ed44de52e02b0d819272f930ee4ecb4c31eeee",
        "0x28c72daa079017ac2e59f90eb088dc9f2900d1d7a49427166173891827827541"
    )]
    #[test_case(
        "0x16a8790a8c6cd1015cda32e8040e9b0e071f03e3c6c8e5fca421193c397cea9d",
        "0x165e900e7abfcca2cea1a088f89779d194e903f39659f00666483afd2410416e"
    )]
    #[test_case(
        "0x18e26d616cfc39c834d7e59ea96e6c289ad02426cbb195e3f6df65b1dc23055d",
        "0x1a8cad9931fe0e7bef40d3b6c1770de5e472bdfca5e2665c83620548fd577b90"
    )]
    #[test_case(
        "0xc263cf26d8fc42a248d39703c3712ad3b6e667c68287dfaa84651ad2f674ef5",
        "0x1aa6e9e20da131b2d0522e57913c27e51dc9d5797ab1c3e9813e3502c1e7c0ad"
    )]
    #[test_case(
        "0x23ad686412dd73cc77ffc7670572db2485ee7394837025ae48c289015c7e978d",
        "0xdbf9264771c45378a72e5b5c3335adccab42552a9064cbc1c622675090f53a"
    )]
    #[test_case(
        "0x220bfae06a1dceee83c753ea8e508cc774132e075a02f2096e3e7d81bd72172d",
        "0x1b4e2f80a83b905494000aed02b250f4d79d62551e024c5aaed162b8d79c3e6d"
    )]
    #[test_case(
        "0x24849513d1e063d79d8fa521a18ff3fbb7f3effb0d94f00877885d904d0314e6",
        "0x1f3b52e053cea7725920af07142a03110811f73822d1193d7a7f71f71dd14c12"
    )]
    #[test_case(
        "0x67d5f9c418e30438b48ec83ebeb2262a7a1ff60577bf38074d8abf732f2f264",
        "0x10000009fd7e1620b7b607f2740183853d6c37a77bed816294f2c3dad6282105"
    )]
    #[test_case(
        "0x5e88d3f870951bb54e1a186f064b988957d0e18acc6e9177576e15db5d9ef29",
        "0x1087dff3386b75e87737469875476093efc5726d79fc6ef6d605b65694fd9c4c"
    )]
    #[test_case(
        "0xc7ae5cabcc87938fbbd6ca8ce43032e04e77b5a31382dd3f155c32eaafba312",
        "0x1d88f46f8b490622b5dc8ecb21496010ea7dcfedbdb03b0379b7a82b13e8d345"
    )]
    #[test_case(
        "0x1f17d00517a321ddf7b37580b49ab1019704fb58c398cccc661574f7ad3bce84",
        "0x10480c2fca990545bf4c75ebb9f3e0d2c1d1202064bec0a2538c361ed5383b40"
    )]
    #[test_case(
        "0x186440545c491379b155591f000ee1a2966e0b06edf7f04821b6c5aa32560ae5",
        "0x5fbc982c0451c06ad62cc26690b79eaa41a328e0c2dd0a2efc30142d800066c"
    )]
    #[test_case(
        "0x1920006d20470b1e1c8915483ea8fc39d9c8d6d28ee99067fb3eacf811264b36",
        "0x18cbd767b3a0abfbe14074c1ca66674582d21811b6db4d1646f0e3974a2bdd80"
    )]
    #[test_case(
        "0x283a1eba0ffb8d78c6d6a83192fe0a3e8c8a85bf676ea15638aaed2298b5e4c2",
        "0xc2f16dbd66f12ddbdca4d2598e1bfb0b4609c6ece9846c9ad206f9a84360c13"
    )]
    #[test_case(
        "0x2a6b2dffe1bbbb905cc8ae33f94979d221254f2ee312f447931e78ae760f384",
        "0x1223e3acf24d68f700fde34dda5b99d3d14a2e19e521d6bae5eb3208ec85da93"
    )]
    #[test_case(
        "0x16ec2b2cfde5cdcc0ad46b2819883b7648cafc79c0ffa6b37495ce963fd9e73a",
        "0x13efd7c9427a9230e2dd4f9912b2f89f2b1b7b8a5381b1c3545eea5329e3760b"
    )]
    #[test_case(
        "0x1af1e3ac1ca9c2c6e50de497a5110f4fab04765a35c2e69f282a4622230f12c9",
        "0x230a32aa27ab6ba91b54885db3f7743eddfce719c86d2f1dc53a66f933bb1d9c"
    )]
    #[test_case(
        "0xa4ce73df237a1bc5586b30c1a90abf2635d85b3d05c19a2559f576c4aca68b2",
        "0x2a8964830d7bb4137e6ccf55bd2d1769a3eb21883f4210a11898f047c990c54c"
    )]
    #[test_case(
        "0x2013f902abddeaf7a59420acfca012978c86fb145710a778a1eb9e3a8cd3e0e",
        "0xfa674fc2825d5cdf1c0b4354f82b81349f21423ac35e21c2941e07309094e99"
    )]
    #[test_case(
        "0x1c8ca38b0b4b152d76bd669baff3df0c2fb5821f3a3409b02a27243920fc092d",
        "0x245c4a67f70014be0c35c3249d26700bb93c8ce5944476fd8e080ccdcedc4f3c"
    )]
    #[test_case(
        "0x28d993d130104cf376503225bb871cb6c7c04f6884d1966b856f0cb22cf12896",
        "0x153ea86b6aced470fedb21490d39a49107706264da949981127ab06016df4413"
    )]
    #[test_case(
        "0x255a0d5a19ab5fb26aadc41155a468d721044645b99a7e5d37f3832723703029",
        "0x2cb62312a43afd94a89aa39701a030a88ce6f0f6fffe7cad0d2464b5b43a8b0c"
    )]
    #[test_case(
        "0x8794e5bfc7d0af0a931f44e0be72f96dce52de694163df1dde09d1ed8f5acad",
        "0x1271b41eaa2a672bfc42d7da1d001de8d2d92fd315f90335c13ea496249dd207"
    )]
    #[test_case(
        "0x1ccc02c48f9c79d7185058e3e987a32f82df5cb5cb393ce878f9d72b87f4cc84",
        "0x144c8f26f6199f4e7c847b18542ed6300491857b307a74e0b43341731752d43b"
    )]
    #[test_case(
        "0x1d9d714ff4edea908f1cba490bbfe172e4678f302e60e43dd8d12a4db829e2cb",
        "0x19493334bf21e33536df25e60eaf793b68ece2f347fd4f18774204e3366f3d2a"
    )]
    #[test_case(
        "0x2c724d15b6f2a12d3c480012fe0b5ca437c82cf489e2d3a2dbdbdc2df75ebf8b",
        "0x107b89affc90a92d066d2be0a0fab2c3c13cfeb3669a149ae70eef3e57b72a5c"
    )]
    #[test_case(
        "0x14b878fab29234cf324d2ca450a17ea47dc6c8c163ee8e599e3bfd88929edda8",
        "0x5db74c45062dcf13c08ac1ad090c4a94d205616846e2db65c6514df4dc4511c"
    )]
    #[test_case(
        "0x19af4d9da562ab34189ead73da36968f1cfe5341764a07076227ac6ddc40331f",
        "0x184ff15b8e41cc46504cea5ed36b2514b81fd02b0a0c0b8cb810ed763b9b5c68"
    )]
    #[test_case(
        "0x8aa7975a01a9e3b97f38f81886f50f01d5f98c1dd342d00fbe0d058dce2ffbf",
        "0x1e88df023cf4833fed24c591a93b757b7c81e102e370b26bd9669ded5b32a912"
    )]
    #[test_case(
        "0x25134030c65aa48bc9f52e9f9de7d1700a0b2bc10a86d7fc8d935716959c398c",
        "0x1349a04b5d6495b421f8bee2e6c5445b381a24612fe8fc43db658a72a538677c"
    )]
    #[test_case(
        "0x21b068deff9b75e9607f7678139e876d3e0479dd9287a60ecbefb588f5fbdf4b",
        "0x1fc5aa3aec42426da9585fc66cf4281f21d7c745a750d0dc6b0bb073e4b66eb1"
    )]
    #[test_case(
        "0x81594083a63da6b2c349931c9236d758d57afa3f81eddb3f4a5fe2d53cb588c",
        "0xfb30f6c0fdf0a8e4fb0a207baae49af20b4eb11bf95497e2b255eea6a7dc461"
    )]
    #[test_case(
        "0x17492b875f2dda6c9b2c49c17b9095ca65de415170d3e9a4d834e5b5ef579182",
        "0x31374fb9c8e46406fa83f5db0274d8d799a480a24b396db3e4df165691d02a6"
    )]
    #[test_case(
        "0x1e6f690b9b2c5b4d3440c1907b38a8cbe2b3267e988c760d2bb426d19014eef9",
        "0xb13a6b0f347da61aff1a5273fc125b6a050cc903022f0ebf366a5d2ed3debc3"
    )]
    fn pow_magic_number_works(x: &str, expected: &str) {
        let x = Fq::from_str_prefixed(x).unwrap();
        let expected = Fq::from_str_prefixed(expected).unwrap();

        let actual = pow_magic_number(x);
        assert_eq!(actual, expected);
    }

    // Empty string in the `expected` argument signals `None`.
    #[test_case("0", "")]
    #[test_case("1", "2")]
    // Python-generated:
    #[test_case(
        "0x20e949416f9b53d227472744dcc6e807311aa8cf1f3de6e23d9f146759d5afe2",
        "0x14142aae4ac3081fc594fde12028d9b329e610472146bdfe7ec3ed4492894b90"
    )]
    #[test_case(
        "0x26df0c83801d57dab45b0e36bbf322a7fecf072e2542b77de0b8eb450165bd1b",
        "0x2596d10178999c2ed646acafa8a43cdd4926f9fb8a2ab3abfb75fcc010291440"
    )]
    #[test_case("0x21d8695d6abc0fcb4f070a45ebab7ce86ca8f82d948222b5ba0d572819c967f6", "")]
    #[test_case("0x11c65bcf21136b93e15a23ce980383e30670e0d7106aff3c38b0558237421ce3", "")]
    #[test_case("0x21c35530c4705937da4166a4f4e0c29b3a5ae1803610cd658638b33637261728", "")]
    #[test_case(
        "0x9a16530e10c85acbab040e9486fe1741e0a7c0d2249421a731d5c6403b4bcf8",
        "0xd7117c72ace2d2cec54fa56ecbd5aa8760896d31308f57b9e56d79d7f7f1b34"
    )]
    #[test_case(
        "0x42b9cfd6e74a002ee2ac03e230b74228359d49c0917784e0d2471867b593354",
        "0x1e3a6b435798e6c845b77845991d6645771d7f6387fdca2f70b733749b432659"
    )]
    #[test_case("0x20ca4d81498b973879ae13e6e84532a80f770d11b2358d195ea2829ba3767afa", "")]
    #[test_case("0x56eca00b1f4332c4ccc29256c0cde908b12f27f52767f223e47d276df7edaa2", "")]
    #[test_case(
        "0x2346bbe95aedd97d4e656c34e26428338e35c7557c401146378e26e325d9ccb1",
        "0xe6529e3bbfe35be3a2f50fdfa139dd7b90d56817b091534e7b1328542a84c9b"
    )]
    #[test_case(
        "0x2c18c2f96d049417a46388f4b33e9c027cda40949fc5ace55c0434ab14043389",
        "0x1bdf570be306f3affe252fa1506ce317e034c073953a5459445ac5a9b9cde10e"
    )]
    #[test_case(
        "0x1cb864f4de5a8449ffe9ffc9d24aae3893226c7fff60a9291729db5cbb6deae3",
        "0x22b4d4c27ff59f4a87cd116fa68a2d4d708c06923d954e5b33a4a777fed7fc65"
    )]
    #[test_case("0x13920c77970b63884e6e9d7129740e8203d5bd427b784a020f204aebe10b9c8b", "")]
    #[test_case("0x9098168314bd49d87562f2f67fe0bfc0d85f1dff08b42280804c25062bcfb56", "")]
    #[test_case("0xd46cf808af0ec6a8986bae5904e937a3c40e542785b57b7705f4c2063ba636f", "")]
    #[test_case(
        "0x304f7642f50320bfd0fecd785aab9e2cb9aa4d78eea9d3db8c5e533ab1753f2c",
        "0xd3a5d7fbc69cb057f7a36ed30c6b8bc1dee54bbea45a50a8459663f09aefc52"
    )]
    #[test_case("0xf37a8f1c56394ff52f1255a26d53f3fe24c7fb0765a3af1e58fc5d3fed84367", "")]
    #[test_case("0x22edcd91537cba89f0a550b266ab6368fba606e7ded18042d0b33a11fa6d3362", "")]
    #[test_case("0x2e1cf89cd4ee4b35ec15d0f9d475c52105f53ff6315629f7df36a9f02ed35646", "")]
    #[test_case("0x1209ec936ef1e7e27bc51787f8be69646a87bffd72a7b1e52e7fcaf9932f74bf", "")]
    #[test_case("0x162d48238d431d770a86ad6a013201cb0436d77ef7ddd2adcd39d6faa92b26d8", "")]
    #[test_case("0xba0fa54d12fb286c4e0b9718c3e1410ea825dd4c2b5929001e064ee7f6361b6", "")]
    #[test_case("0x1ebb509c2ecad8e01a0d86f0035ec68629aed3d5878300e044982292126783df", "")]
    #[test_case("0x139c6113eee4057f6304b4e6472300b73c8c8a6d570b913d4d23528cfa661428", "")]
    #[test_case(
        "0xb177c6f82daed853261bc68080e17a4db51bc68d2c7e052dd483ddecca3d539",
        "0x131902ccbeb6b5d5fcfd2902ff324b5ca03d26284f09932bbfb11c0cf25cbe04"
    )]
    #[test_case(
        "0x11c2e51e2564719627780fa81f817ecdce8d34d3c4bce40a4164aa68a331753c",
        "0x8ff562cbcbb36c6efc63b7d681820039bc4e8dff8ba94f26078a60a89312789"
    )]
    #[test_case(
        "0x101d78729b63303abee4f033c2bc62de1a6aa85abd38c8b1ce5f61ff312e0b7",
        "0xd2c903227d0b4da36a01788f84ca0abd481f75cd255907e99cfc90290af84c"
    )]
    #[test_case("0xb4aaee9e08f72a75fd4163a04ef9c2cef8467f388906a9dba01cc0a2707b31d", "")]
    #[test_case("0xbb0249aa04e94c409b4edafe3b0b5473b85fc3a73db9e7105dc18ff567b665b", "")]
    #[test_case("0x303dfe6b7f04759adb9c16d9a5f5e5e97ea4e059d7e4a3bc3d2bd466359c9d4b", "")]
    #[test_case("0x1cf614533dfc15bcc167f4ffc7853ae15ce0364b51eb9b44df0ce3a081b55fc6", "")]
    #[test_case(
        "0x10954baebe08defd59e4a9c21c2506a16b41c7529d23674471c9a05371070abf",
        "0x1732f7503ba205a5c39430040b2fd3f7f590eee539e01b1d645ca4de7f7ea5af"
    )]
    #[test_case("0x10f58303b4a4e4484ba1ce4c4a9e7e2df534ed15af0cfb0f283f9175af9fb6e5", "")]
    #[test_case("0x7025c6378827384463d968e6bacfec443ec08a194104fd28bf6b247930c9bdd", "")]
    #[test_case("0x1057d4959c007fa74d0e30dd6d15fbdaadd9c90279ca639c96d4f10d3af8a231", "")]
    #[test_case(
        "0xc2b2dcb2d9fe4efacc93808d9eb008d73712dacc9d4e5c2bc7a4a9fd1ff1ac0",
        "0xa3ea41649a906bcb181557dc75e7409961e372b32b22901257763b83f96e409"
    )]
    #[test_case(
        "0x28b8345b48442d41b77865f9fadb2e6e738f2e7d7d751d6b28d9e993fe361fd6",
        "0x203e0b4abb8bda20274a279082157b465e7ac24a307c31d5470d6423a3d66d59"
    )]
    #[test_case(
        "0x1f21a85e740f062dc35e7b38b7971c9224f17148f61035964d0741e8fa7ef795",
        "0xcc4ba641136ca82b8318f2a223be701a30de95619badb5866c37d6349e8f3ab"
    )]
    #[test_case("0x851d8e29b71af39041cca092d1186d6aaf0e29571707b706f7bea1f6536dea5", "")]
    #[test_case("0x22583cc7c94461fe8c221c39d3e3f9c0cf845c46b941f0685eb9d92b948d57b4", "")]
    #[test_case(
        "0x3677deec6a292db9413192996bef7e54cf765765e76f0442b48d03d78c1b4ca",
        "0x14a8e165c8d161b9da152a082e891aac336deb6ef0a107e1372bcfc14a25bf1e"
    )]
    #[test_case(
        "0x2b35222e0028f08e2cbe1b5b57880a5269d10f98d92e4c7ff173cbca5acc3c25",
        "0x1254ccfdd7ed6582a08f6f14e202b8426b8401154f201b49c6554439f475d053"
    )]
    #[test_case(
        "0x1a0559b8639f6ca09fbe7de8e573c75b52fcf271f59bb8d42de27e5b1dac5a51",
        "0x2b5fe06267f5728b6b07066a6371f717b7a881e06ac81f94e671e13c116f5ba2"
    )]
    #[test_case("0xbb47c6119cf19d7adde393c20995152d0a4602634d2438998be00e7f8948b06", "")]
    #[test_case(
        "0x2bdd9b5112703affe898f3dc4467c54029a5e765a2932eae1911cfe9cd8a01fd",
        "0x101458bcc903d98b9da338deebb97fa332a3b18eebeeeffca42fc360e8de37a7"
    )]
    #[test_case(
        "0x2fed5842e08f871e084de333470ad5ff7f863cad37927a8e8efd0428a9a532c0",
        "0x9ebd892a4e342dd6f4ebb4dd3a1f170192472e9a1f6f8de949f358fc36390ac"
    )]
    #[test_case(
        "0x28af3fa7a519c2b8fc659dae65d3b2b81218da021dad2a1234b3a26b058ef32b",
        "0x28a7177a3426b76cca4c9330911dd0c6d03fdc9458bc6f33b0b705aa642bc7a7"
    )]
    #[test_case("0x26e60dc63ec623b6b26972e6027d684925e7f482cd181eb73aabbf040f3b7ff6", "")]
    #[test_case("0x20d6c202cbe710e1086135b2aeae79c613f87fab0ad907224f37e65512401e51", "")]
    #[test_case("0x1b8a83401cce664812a7fd33096367451bc4d7a63178c84bc0fcfe7f5e7a866f", "")]
    #[test_case(
        "0x2420da207121c4043e1d956963a5bf85162ea695b29bfdd3fc6c62c5edf74ec0",
        "0x15643f4438f6a5ebd13ee69527a3c2d74e2f6e17f37a107d74787e11c28e04b"
    )]
    #[test_case("0x7fc07f648a39c0ec90b3d67229e4476cf67f7b9e53e765bd4d42d0cfbb7457e", "")]
    #[test_case(
        "0x42c25def65eb895a1b0180670625743ac57607f1c405ad9d574474382f35d62",
        "0x1d9d4fccdc146e3a621bfb55dbf8abcc07f98b2033e63a92416fdf705c12f7b7"
    )]
    #[test_case(
        "0x101d3fbe5e5b208434fe6cd8b059fb313207f50871135ee6d218b0e2474828fb",
        "0x113cf809c4e19b12bccb04341f807d6bd7ac373ef10f42d19788f6cef2a6685a"
    )]
    #[test_case("0x7947c8aa8cbdabafc8a2850e235cea4975b5a5579998ac2296c9d1e89bbb9de", "")]
    #[test_case("0x995594be605fbf8da0939f1141cc5a34200e5e3081b0f0b240684c95e36b4ce", "")]
    #[test_case(
        "0x285c71f8a0fdf24ffd992a7d052270d1a091d5af3e7adc468b3f5eb2e888f2d4",
        "0x276b94e9be42dac2f1311a5983897f69eaca89823559f3017dddff990f67a793"
    )]
    #[test_case(
        "0x21dffeca30717d80192b3aa7a4e783a4eca4505f82d035d784dac171e277a3bf",
        "0x2cb0757b9380194e04cd5963fff757de50d03831e4c283c46f7b0aa3d3e65641"
    )]
    #[test_case(
        "0x2b8e383d30dba6b63255e01cea467a612b61fde33259dd090d222d28c0c1d77d",
        "0x280980898b96035f9bf42af16aff52612b5e6016109e1aeb1ac525ca2477fce2"
    )]
    #[test_case(
        "0x150625fe25855faaacd55f741003c4a372eaf7abbb3c01552bfd24339a365ea4",
        "0x1826aead148f59af79633af3b978acbae66a3f1d4a49c1f1644a7507036d9a77"
    )]
    #[test_case("0xe98804dee68836ff4b4e42958efe51f4f1f669a9b8a1563f77f4e5805c42f51", "")]
    #[test_case("0x84ed2d57e8a1c8ca2bd33b931ac29379667203438af460553519fec8175541a", "")]
    #[test_case(
        "0x191ec6a5b4fa90143c450666337adae84179dd007da85721b9c9dec27d89e46f",
        "0x21b6ca804747058e4494e37f8013d244a343bb7b87f7f98f5577ab72c14b61c3"
    )]
    #[test_case("0x1ebf595517ff0e4c47793858604b0246b9c1a7677f372e1b7e46fa68dcf02a4d", "")]
    #[test_case(
        "0x17354a528bd9b24d83e741600566c8ff636dcfd04a8cb14b23eecc6c94857102",
        "0x189ca63421d9c8c324d36169775e37d5ead71eb04fce35a1e5cea3bf581ae621"
    )]
    #[test_case("0x2a0bb756e81294701bc9d286914aeb9dcb5c803f73b04f91245e46be5120ae92", "")]
    #[test_case("0x24dc380cd16479bb29e7651c506f23563f8a913b153d7c3a893981e8687bfb43", "")]
    #[test_case(
        "0x216db64e966475e96833d188725301fa0c04bb6b1d522fa93d161301b719c0bf",
        "0x61b0bf3ab67041a1b606340242661a6e33f04f471ec172cf4b573d4129242f5"
    )]
    #[test_case(
        "0x9f3a0b8ae4c51bd58da91095af52e6bef308ae8549b3a31d0358cdac7964976",
        "0x1425a6c7ff8a1c54c63724d804a38b5084ab88fc6f01e162b8c78e8f32c5ead2"
    )]
    #[test_case(
        "0x496a014cbfea4384fec7451a84b976ab0c28f76dd3ab3dc210be9b3bc4a7a94",
        "0x1922462c88634ed2177c19df71fb80b6e5a5fca6ada05647ee00829a0a9cc482"
    )]
    #[test_case(
        "0x2c751346419501eb3163f0e980a692ac5ff7ae880c23631737aba070d96a5068",
        "0x10ec84d1aec1064a6168858d16d61742edb9b343c62af7749c853a13e319cb56"
    )]
    #[test_case(
        "0x5aa5e9173ba1b12d79893accb863af98034dc10bee04a72745627805a96f432",
        "0xa44fbe2c2f13907d2d26568156442f252f1499c67d5e89224ec9516efbeec9a"
    )]
    #[test_case("0xf11b967d4fd453f72ea17c192057163a4b86faeba58ae230cb93bdef0243a2d", "")]
    #[test_case("0x17650d85a167378d8ecaed35d6c33f90011b5c1763b3a67edf99374e048e539", "")]
    #[test_case("0x4ce9c9935a2727683a3b19bc28c2b1a9e3f629899539e3a3bdf1e438e06af27", "")]
    #[test_case(
        "0x29f83eef57bb73a548854860b466f8960265aef86ce96b03d3a694c8d1bd432a",
        "0x2b6bf3ca49338fba98c6d0568093e7b350dfcf8eec2296d4dd514834a1f325d6"
    )]
    #[test_case("0x10f39b3b6e56245bcea1c0eaac5dde6f3dd4c245dc723dda85f28f7e8f059af1", "")]
    #[test_case("0x802c3e67cb4052e844722a8cdffbc32929468ee86a395e7a381706b47be3ac9", "")]
    #[test_case(
        "0x1867b07712a6519c3c32c8f6571812ba523d3bce9b20eced5b926ce452b359b6",
        "0x2c06e3a418433f6e8a72841772eb8968fba3eff8b6346b4d2d31730f6bda6254"
    )]
    #[test_case("0x127305af097d76b7ba80afb3b4d648ea1ef18137a6ed87193f79433a135dfcf1", "")]
    #[test_case(
        "0xf21ccbb974fd751baf632cc4d283d41ef3bf3692831d63c5f127c75b8568373",
        "0x13c3f7f4f167ac914f3f1ef9cfe0b0825939eda1612fbdc1e72c30fd16c2cbb0"
    )]
    #[test_case(
        "0x29c06ca72b9dd0e167319520dc58b330b0589fec9940e46b36e7e6d89f2d5f51",
        "0x2b863a476d7134bae804560ce9deb21507009fcb38ed7e891da5330e6cebf843"
    )]
    #[test_case(
        "0x24593c874871b77055cd668994aed955b3fd217707fc1e1aa548c46cce8b097",
        "0x1334a18f5cee20d4424129d25910fcf6f6df0a110ed190fca6790ca75f0f2a34"
    )]
    #[test_case("0x1d5f05febf7f601d031491c42106903a5092f00c5768b95d0fea08a5fda111d0", "")]
    #[test_case("0x28670112f911193a4fc626adce6ce40c8a4edf239379f69955b38d7e591a12ae", "")]
    #[test_case(
        "0x24e8ecef3e8905546882f8f34aaaa6936470006ed6d344ea9d9026ab3df5c224",
        "0x1f19c628f201c1132c459133f21a2084e11d5f102e0fcf0662093193bffcea86"
    )]
    #[test_case(
        "0x5d1d993e9dee8cd8065df381781ae2a452b54443dc4bd7b25aedf7e3c93b903",
        "0x1af0c09e4e5fa57be5d4a00a3f3a1a4ca7a4cc98f02e1d75d7353a4822071b58"
    )]
    #[test_case("0x195235e8cf1639843590e2c504b32d98a1e6f6c238088a9602f2e01080303c90", "")]
    #[test_case("0x19ed8c7dbc98179279c78eb4934e284f65bd8ad2840225e9c7cabe116620430b", "")]
    #[test_case(
        "0x17f87b57ea3ea75b10a58a5f4f14e77b7a3dc37980f750a1acbdc9f3eb7161a4",
        "0x18d3efc9587da0fad2549b9435b3a8dea192a296f00951fa204cae6177e70b99"
    )]
    #[test_case("0x235bee2c7fb649ce30cd804d98133ab5dfcc793528f7a776f1768a28ce24371", "")]
    #[test_case("0x2ad3dfe16d847f078d0ee7a573a41ff13fcd57a30f0c15ab22da5698c572facd", "")]
    #[test_case("0x2efa31393facd1c3916012ef31a5d854e6c1d4a9ad2960fc962c804b90d98e59", "")]
    #[test_case("0xede9f77d62bb62fbe81b267435fcf72fd0251b2814d10df320f6ea215eed56", "")]
    #[test_case("0x1db1fd22a8fc9dbc46a5d4cefe21ec711cfadae76414c4f6ab3cb6cd95738764", "")]
    #[test_case(
        "0xcf6c703c780e6f72140faa60768188edcaaf7bfc75b68757083888e477a3f28",
        "0x2bdea6633e0f2780a80366d9bc91d10a75904c9cd4555d462ef6166747fb629e"
    )]
    #[test_case(
        "0x23f6e9a2d6f32a5255f33846b7611cc5043a7d965a17996fa4c1f3ad0ed56ebc",
        "0x2f869f25efedaa551614153f9c706b3da92eeaab22137386e591865d25482306"
    )]
    #[test_case(
        "0x29949070e2f5f3055c3afdeb38955a6d407b30b260b256e151df6208d90c12eb",
        "0x1ec27fcc641f0d36c77d4ee49c905a562a5c25fb72f7cd1174b09b3537c0542e"
    )]
    #[test_case("0x1b8e001f30a427f88bebdb44d5ad768e417f79b6a087f135b26a1c872437fc99", "")]
    #[test_case(
        "0x16828439e722906f70b29c4837cdff27680ba81e01c7b1f210e558318090eb4d",
        "0x2f3a7f5710ff2bc13fd4b2cd3a84a61f6c7bc7ea3a2e125ae89fcfabfd9e737d"
    )]
    fn matching_y_coordinate_works(x: &str, expected: &str) {
        let x = Fq::from_str_prefixed(x).unwrap();
        let expected =
            if expected.is_empty() { None } else { Some(Fq::from_str_prefixed(expected).unwrap()) };

        let result = matching_y_coordinate(x);
        assert_eq!(result, expected);

        // Ensure that the result is actually a point on `alt_bn128`.
        if let Some(y) = result {
            let xx = x * x;
            let xxx = x * xx;
            let xxx_plus_3 = xxx + Fq::from(3);
            let yy = y * y;
            assert_eq!(yy, xxx_plus_3);
        }
    }

    fn field_modulus_works() {
        let expected = U256::from_str_prefixed(
            "0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47",
        )
        .unwrap();
        assert_eq!(field_modulus(), expected);
    }
}
