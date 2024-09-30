/// Highest/lowest bit always refers to the big endian representation of each bit sequence.
mod tests;

use super::typedefs::Hash;
use core::num::ParseIntError;
use ethnum::U256;
use halo2curves::{
    bn256::{Fq, G1Affine},
    ff::PrimeField,
    CurveAffine,
};

pub(crate) fn get_collection_id(
    hash: Hash,
    parent_collection_id: Option<Hash>,
    force_max_work: bool,
) -> Option<Hash> {
    let mut u = decompress_hash(hash, force_max_work)?;

    if let Some(pci) = parent_collection_id {
        let v = decompress_collection_id(pci)?;
        let w = u + v; // Projective coordinates.
        u = w.into(); // Affine coordinates.
    }

    // Convert back to bytes _before_ flipping, as flipping will sometimes result in numbers larger
    // than the base field modulus.
    let mut bytes = u.x.to_bytes();
    bytes.reverse(); // Little-endian to big-endian.

    if u.y.is_odd().into() {
        flip_second_highest_bit(&mut bytes);
    }

    Some(bytes)
}

const DECOMPRESS_HASH_MAX_ITERS: usize = 600;

/// Decompresses a collection ID `hash` to a point of `alt_bn128`. The amount of work done can be
/// forced to be independent of the input by setting the `force_max_work` flag.
///
/// We don't have mathematical proof that the points of `alt_bn128` are distributed so that the
/// required number of iterations is below the specified limit of iterations, but there's good
/// evidence that input hash requires more than `log_2(P) = 507.19338271000436` iterations.
fn decompress_hash(hash: Hash, force_max_work: bool) -> Option<G1Affine> {
    // Calculate `odd` first, then get congruent point `x` in `Fq`. As `hash` might represent a
    // larger big endian number than `field_modulus()`, the MSB of `x` might be different from the
    // MSB of `x_u256`.
    let odd = is_msb_set(&hash);

    // `Fq` won't let us create an element of the Galois field if the number `x` represented by
    // `hash` does not satisfy `x < P`, so we need to use `U256` to calculate the remainder of `x`
    // when dividing by `P`. That's the whole reason we need ethnum.
    let x_u256 = U256::from_be_bytes(hash);
    let mut x = Fq::from_u256(x_u256.checked_rem(field_modulus())?)?;

    let mut y_opt = None;
    let mut dummy_x = Fq::zero(); // Used to prevent rustc from optimizing dummy work away.
    let mut dummy_y = None;
    for _ in 0..DECOMPRESS_HASH_MAX_ITERS {
        // If `y_opt.is_some()` and we're still in the loop, then `force_max_work` is set and we're
        // jus here to spin our wheels for the benchmarks.
        if y_opt.is_some() {
            // Perform the same calculations as below, but store them in the dummy variables to
            // avoid setting off rustc optimizations.
            let dummy_x = x + Fq::one();

            let matching_y = matching_y_coordinate(dummy_x);

            if matching_y.is_some() {
                dummy_y = matching_y;
            }
        } else {
            x = x + Fq::one();

            let matching_y = matching_y_coordinate(x);

            if matching_y.is_some() {
                y_opt = matching_y;

                if !force_max_work {
                    break;
                }
            }
        }
    }
    std::mem::forget(dummy_x); // Ensure that the dummies are considered "read" by rustc.
    std::mem::forget(dummy_y);
    let mut y = y_opt?; // This **should** be infallible.

    // We have two options for the y-coordinate of the corresponding point: `y` and `P - y`. If
    // `odd` is set but `y` isn't odd, we switch to the other option.
    if (odd && y.is_even().into()) || (!odd && y.is_odd().into()) {
        y = y.neg();
    }

    G1Affine::from_xy(x, y).into()
}

fn decompress_collection_id(mut collection_id: Hash) -> Option<G1Affine> {
    let odd = is_second_msb_set(&collection_id);
    chop_off_two_highest_bits(&mut collection_id);
    collection_id.reverse(); // Big-endian to little-endian.
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

fn field_modulus() -> U256 {
    U256::from_be_bytes([
        0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58,
        0x5d, 0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c,
        0xfd, 0x47,
    ])
}

/// Flips the second highests bit of big-endian `bytes`.
fn flip_second_highest_bit(bytes: &mut Hash) {
    bytes[0] ^= 0b01000000;
}

/// Checks if the most significant bit of the big-endian `bytes` is set.
fn is_msb_set(bytes: &Hash) -> bool {
    (bytes[0] & 0b10000000) != 0
}

/// Checks if the second most significant bit of the big-endian `bytes` is set.
fn is_second_msb_set(bytes: &Hash) -> bool {
    (bytes[0] & 0b01000000) != 0
}

/// Zeroes out the two most significant bits off the big-endian `bytes`.
fn chop_off_two_highest_bits(bytes: &mut Hash) {
    bytes[0] &= 0b00111111;
}

/// Returns a value `y` of `Fq` so that `(x, y)` is a point on `alt_bn128` or `None` if there is no
/// such value.
fn matching_y_coordinate(x: Fq) -> Option<Fq> {
    let xx = x * x;
    let xxx = x * xx;
    let yy = xxx + Fq::from_raw([3u64, 0u64, 0u64, 0u64]); // 3
    let y = pow_magic_number(yy);

    if y * y == yy { Some(y) } else { None }
}

/// Returns `x` to the power of `(P + 1) / 4` where `P` is the base field modulus of `alt_bn128`.
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
