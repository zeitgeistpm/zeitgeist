// Copyright 2024-2025 Forecasting Technologies LTD.
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

use super::*;
use test_case::test_case;

// Empty string in the `expected` argument signals `None`.
#[test_case("0x00", "")]
#[test_case("0x01", "0x02")]
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
    let x = Fq::from_hex_str(x);
    let expected = if expected.is_empty() { None } else { Some(Fq::from_hex_str(expected)) };

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
