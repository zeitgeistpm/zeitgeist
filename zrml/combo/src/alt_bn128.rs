use core::num::ParseIntError;
use ethnum::U256;
use halo2curves::bn256::Fq;

fn quadratic_residue(x: Fq) -> Option<Fq> {
    let xx = x * x;
    let xxx = x * xx;
    let yy = xxx + Fq::from_str_prefixed("3").ok()?;
    let y = pseudo_sqrt(yy);

    if y * y == yy { Some(y) } else { None }
}

fn pseudo_sqrt(mut x: Fq) -> Fq {
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
        let bytes = x.to_le_bytes();
        let ct_opt = Fq::from_bytes(&bytes);

        // Map `CtOption` to `Option`. FIXME Breaking our rule of not having panickers (but this
        // clearly can't panic).
        if ct_opt.is_some().into() { Some(ct_opt.unwrap()) } else { None }
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
    fn test_pseudo_sqrt(x: &str, expected: &str) {
        let x = Fq::from_str_prefixed(x).unwrap();
        let expected = Fq::from_str_prefixed(expected).unwrap();

        let actual = pseudo_sqrt(x);
        assert_eq!(actual, expected);
    }

    #[test_case(Fq::from(0))]
    #[test_case(Fq::from(1))]
    fn test_pseudo_sqrt_is_sqrt(x: Fq) {
        let sqrt = pseudo_sqrt(x);
        assert_eq!(sqrt * sqrt, x);
    }
}
