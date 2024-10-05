use super::*;
use test_case::test_case;

#[test_case(
    [0x16, 0x74, 0xab, 0x10, 0xed, 0xf8, 0xc4, 0xe2, 0x25, 0x72, 0x9e, 0x20, 0x9a, 0x58, 0x75, 0xa1, 0x9f, 0x14, 0x46, 0xba, 0xec, 0x3b, 0x30, 0xdf, 0x9b, 0xa8, 0x65, 0x75, 0xd5, 0x2d, 0xe3, 0xd3],
    (
        "0x1674ab10edf8c4e225729e209a5875a19f1446baec3b30df9ba86575d52de3d3",
        "0x1919edaf92ff08c3c5a2a5dafef1a0c01376dab9681be7fbe3895a18b96af98e",
    )
)]
#[test_case(
    [0x02, 0xfd, 0xc0, 0xbc, 0xde, 0x3b, 0x3d, 0xa1, 0xb4, 0xd6, 0x0d, 0x2f, 0x3f, 0x2c, 0xe7, 0x51, 0xd5, 0x20, 0xce, 0x53, 0xe0, 0x10, 0xb1, 0x16, 0x85, 0x9a, 0x8a, 0x9d, 0xe4, 0x6d, 0x45, 0x5e],
    (
        "0x2fdc0bcde3b3da1b4d60d2f3f2ce751d520ce53e010b116859a8a9de46d455e",
        "0x23a3ae9baa8ed04165a7ebfdb1ffb683d494fc3bf6e402c23b7de5b8ca3b41f6",
    )
)]
#[test_case(
    [0x18, 0x0a, 0x0e, 0x6e, 0x26, 0x13, 0xbc, 0x6e, 0x78, 0x1b, 0x9d, 0x8d, 0x9f, 0xc4, 0x16, 0xe0, 0x51, 0xbb, 0xa5, 0xe3, 0x83, 0x63, 0x93, 0xb2, 0x3e, 0x4f, 0x1d, 0x37, 0x7e, 0x2d, 0x52, 0x2d],
    (
        "0x180a0e6e2613bc6e781b9d8d9fc416e051bba5e3836393b23e4f1d377e2d522d",
        "0x1bc28847ccf82c7345687caba07a45b130e26e3f4489ceb47524e08a37d28d26",
    )
)]
#[test_case(
    [0x1d, 0x9b, 0x46, 0x27, 0xaa, 0x60, 0x6d, 0x7d, 0xda, 0xd6, 0xfe, 0xe6, 0x5d, 0xd9, 0x52, 0x5d, 0x75, 0xac, 0x9b, 0x00, 0x14, 0x42, 0xfa, 0xe6, 0x5a, 0x6e, 0xfd, 0x8f, 0xd4, 0x36, 0x9e, 0xb7],
    (
        "0x1d9b4627aa606d7ddad6fee65dd9525d75ac9b001442fae65a6efd8fd4369eb7",
        "0x20181bfe3b6cfce9bebb4b3870ddbd9f0cc1bdfff9f15f9bc85debc254ab4b9c",
    )
)]
#[test_case(
    [0x1d, 0xbf, 0x33, 0x21, 0x4f, 0x0a, 0xbe, 0xab, 0x8e, 0x39, 0x97, 0xf7, 0x6c, 0x79, 0x62, 0x90, 0x79, 0x5a, 0xc5, 0xc5, 0x1f, 0x51, 0xa7, 0xfb, 0x66, 0x25, 0x8c, 0x5a, 0x72, 0x07, 0x78, 0x9d],
    (
        "0x1dbf33214f0abeab8e3997f76c796290795ac5c51f51a7fb66258c5a7207789d",
        "0x241860400ae39bd169011fc88731985403051a72222cc82db0f443cbd2f9b886",
    )
)]
#[test_case(
    [0x22, 0x3c, 0x15, 0x71, 0x4d, 0x71, 0x7d, 0x70, 0x08, 0x31, 0x72, 0xa2, 0x60, 0xa9, 0x6e, 0xb9, 0xe0, 0x13, 0x40, 0x1b, 0xdd, 0x3e, 0xbe, 0x00, 0xd4, 0x71, 0x49, 0x8c, 0xac, 0x52, 0x45, 0x1a],
    (
        "0x223c15714d717d70083172a260a96eb9e013401bdd3ebe00d471498cac52451a",
        "0xe05de1bd5c32a55bbf45ebbb59bd406ea680b54f81bcba4a0ba2ec481d1d78e",
    )
)]
#[test_case(
    [0x23, 0xb5, 0x47, 0xd2, 0x96, 0xbe, 0x38, 0x65, 0x86, 0x60, 0x33, 0xfe, 0xbb, 0xe9, 0x4a, 0x12, 0x49, 0x2c, 0x81, 0x94, 0x38, 0xfa, 0x7b, 0x5a, 0xd6, 0xc2, 0x0c, 0xbf, 0xb2, 0x82, 0x43, 0x0d],
    (
        "0x23b547d296be3865866033febbe94a12492c819438fa7b5ad6c20cbfb282430d",
        "0x12cdd3c3f897dbc6237d317b1cc607916fafe3775d2971ee40c5248ee18e2b2a",
    )
)]
#[test_case(
    [0x0c, 0x05, 0x3e, 0xb3, 0x09, 0xe1, 0x48, 0xe1, 0xe9, 0xde, 0xb2, 0x46, 0xe4, 0xee, 0x89, 0x74, 0x90, 0xed, 0xd5, 0x4e, 0x26, 0xae, 0x27, 0x56, 0xf1, 0xc4, 0x94, 0x0f, 0x28, 0x84, 0x16, 0xe1],
    (
        "0xc053eb309e148e1e9deb246e4ee897490edd54e26ae2756f1c4940f288416e1",
        "0x23d95d4f591b78e62a38e3d9c4cd40642303f54ad08e94139be77316ea58a16a",
    )
)]
#[test_case(
    [0x0b, 0x59, 0x17, 0x6a, 0xba, 0xb9, 0x17, 0xe7, 0x72, 0xfe, 0x94, 0x50, 0xa8, 0x69, 0xcf, 0x62, 0xc8, 0x32, 0x88, 0x4a, 0x0f, 0xfd, 0xb6, 0x06, 0xbb, 0x6b, 0x3f, 0xa7, 0x0c, 0x1d, 0xb9, 0x1a],
    (
        "0xb59176abab917e772fe9450a869cf62c832884a0ffdb606bb6b3fa70c1db91a",
        "0x1420a6cb7d026a2c9cb2c05f7ff86afc73a11a6dfd90ce540cfe02d19372524e",
    )
)]
#[test_case(
    [0x1e, 0x08, 0x14, 0xc5, 0xe1, 0x64, 0x5b, 0x62, 0x5b, 0x9d, 0xfc, 0xff, 0xe9, 0x7e, 0x49, 0x77, 0x32, 0xf5, 0xfc, 0x65, 0x06, 0x1c, 0x75, 0xf0, 0x06, 0x06, 0x92, 0xb7, 0xa2, 0xea, 0x39, 0x83],
    (
        "0x1e0814c5e1645b625b9dfcffe97e497732f5fc65061c75f0060692b7a2ea3983",
        "0x1506fac00eed4e27b88c2d07811b0e1586a42494e9f5acf95dac46d25ab93b00",
    )
)]
#[test_case(
    [0x07, 0x50, 0x54, 0x78, 0x15, 0x5f, 0xcf, 0x43, 0x5d, 0x96, 0x77, 0xcc, 0x58, 0x7c, 0x85, 0x1e, 0x47, 0x02, 0xb7, 0x3d, 0xc2, 0xd8, 0xc6, 0xf5, 0x16, 0x7d, 0xbb, 0x84, 0x6c, 0x72, 0xbd, 0xb3],
    (
        "0x7505478155fcf435d9677cc587c851e4702b73dc2d8c6f5167dbb846c72bdb3",
        "0xce98d3afa473f8a47d734aa55dd74f3f6d18faf46689346e9930dfd2689877a",
    )
)]
#[test_case(
    [0x28, 0x0a, 0x7e, 0xbf, 0xaf, 0xce, 0x97, 0x43, 0x55, 0x0f, 0x42, 0x8f, 0xc2, 0xd4, 0xdd, 0x28, 0x8f, 0xa8, 0x13, 0x24, 0xcb, 0x6e, 0x10, 0xa6, 0x7b, 0x42, 0x34, 0x5f, 0x1b, 0x76, 0xc6, 0x5d],
    (
        "0x280a7ebfafce9743550f428fc2d4dd288fa81324cb6e10a67b42345f1b76c65d",
        "0x26ca96ef76f1c517684cfeb53b22157e340990317c020fcf8b631e7157e4ce92",
    )
)]
#[test_case(
    [0x08, 0x46, 0xe2, 0x53, 0x97, 0x46, 0xca, 0x06, 0xad, 0xa1, 0x8b, 0x22, 0xba, 0x2f, 0x66, 0xda, 0xcc, 0xaf, 0x0e, 0x9a, 0x99, 0x5c, 0x29, 0x35, 0xce, 0x8d, 0xbc, 0x55, 0x20, 0x8d, 0xcc, 0xbb],
    (
        "0x846e2539746ca06ada18b22ba2f66daccaf0e9a995c2935ce8dbc55208dccbb",
        "0x1336db44313546d7e740d3b1ed924eb8dbbd73dbb4399b7e37549562ad789930",
    )
)]
#[test_case(
    [0x1b, 0xe5, 0xd3, 0x49, 0x51, 0x06, 0x3a, 0x47, 0x6a, 0x3c, 0x78, 0xa7, 0xdb, 0x40, 0x85, 0x5c, 0x49, 0xf2, 0xc5, 0x70, 0xc5, 0x06, 0xb8, 0x5e, 0x3b, 0xef, 0x44, 0x05, 0x68, 0xab, 0x02, 0xab],
    (
        "0x1be5d34951063a476a3c78a7db40855c49f2c570c506b85e3bef440568ab02ab",
        "0x5d5ebaef6981b87cde5ed99565a36d4eafbc6fbacab1f0cd15e509fba71d8c2",
    )
)]
#[test_case(
    [0x01, 0x37, 0xe1, 0x21, 0xfc, 0x1b, 0xc8, 0x0c, 0x5b, 0x30, 0xf2, 0xad, 0x1a, 0x08, 0xe9, 0x26, 0x53, 0x30, 0xfb, 0x33, 0x07, 0x84, 0xa5, 0x63, 0x43, 0xc2, 0x9a, 0xc0, 0x46, 0x20, 0x1d, 0x1b],
    (
        "0x137e121fc1bc80c5b30f2ad1a08e9265330fb330784a56343c29ac046201d1b",
        "0x18d3facbbf735827083ff4e12a625ba2218edffef5c8815afdefea6528801fec",
    )
)]
#[test_case(
    [0x2d, 0xf7, 0x8d, 0xdd, 0x64, 0xbc, 0xb7, 0x53, 0x60, 0xa7, 0x88, 0x16, 0x35, 0x29, 0xe2, 0x84, 0x95, 0x04, 0x08, 0x4a, 0x4b, 0x79, 0x91, 0x17, 0x28, 0xee, 0x33, 0x03, 0x4c, 0x7f, 0x6c, 0xd3],
    (
        "0x2df78ddd64bcb75360a788163529e2849504084a4b79911728ee33034c7f6cd3",
        "0x1703b907563d006f1df3fe65c0ffff73d469b58b831827d2b68e12278bf9f0e",
    )
)]
#[test_case(
    [0x11, 0x05, 0x42, 0x04, 0x50, 0x3c, 0x67, 0x3f, 0x29, 0x4d, 0xf5, 0x82, 0xe7, 0x19, 0xe9, 0x5c, 0x40, 0x50, 0x2b, 0x65, 0xda, 0x36, 0xae, 0x0b, 0x05, 0x1c, 0xea, 0x5b, 0xa3, 0x80, 0x24, 0x7f],
    (
        "0x11054204503c673f294df582e719e95c40502b65da36ae0b051cea5ba380247f",
        "0xe9c934bcf52dfdefbcd8b9b1d669e41924ae89fb758d1bfacdc33c6ab98b6fc",
    )
)]
#[test_case(
    [0x09, 0x1d, 0x74, 0x16, 0xbb, 0xf0, 0x48, 0x03, 0x92, 0x0a, 0x1f, 0x84, 0xd1, 0xd5, 0x63, 0x28, 0x0d, 0xbb, 0x5a, 0x6e, 0x3b, 0x03, 0x3f, 0xce, 0x74, 0xaf, 0x4a, 0x6d, 0x63, 0xf9, 0x02, 0xe3],
    (
        "0x91d7416bbf04803920a1f84d1d563280dbb5a6e3b033fce74af4a6d63f902e3",
        "0x2976e2bde1f1b94ba5a82a3323351f48148a1e518ac2ed0aa0070fcbd12a9784",
    )
)]
#[test_case(
    [0x1b, 0x09, 0x08, 0xb9, 0xc1, 0xf2, 0x25, 0xa8, 0xfe, 0xf8, 0xf1, 0xff, 0x1d, 0x89, 0xf8, 0x65, 0x44, 0x06, 0x5a, 0xb2, 0xf5, 0x28, 0xed, 0x8a, 0x88, 0x47, 0x39, 0x04, 0x4b, 0x9b, 0x5d, 0x0d],
    (
        "0x1b0908b9c1f225a8fef8f1ff1d89f86544065ab2f528ed8a884739044b9b5d0d",
        "0x13bd194272a6615dfbfae1f888ace71eb2c035973b841c47cabbd5f617b55e5c",
    )
)]
#[test_case(
    [0x1e, 0xb7, 0xf8, 0x6e, 0x71, 0x58, 0x7f, 0x50, 0x91, 0x19, 0xec, 0xe1, 0xb1, 0x90, 0x01, 0xfa, 0xc7, 0xbd, 0x45, 0x79, 0xa1, 0xe1, 0xae, 0xdf, 0xca, 0x4e, 0x11, 0x9a, 0x77, 0x78, 0xc0, 0xe0],
    (
        "0x1eb7f86e71587f509119ece1b19001fac7bd4579a1e1aedfca4e119a7778c0e0",
        "0x14a7107c10f3b98153f4750d35b7dc5713defeab87d7ce4712ed84b7657efb84",
    )
)]
#[test_case(
    [0x18, 0x2f, 0x02, 0x1a, 0xcc, 0x92, 0x56, 0x45, 0x5e, 0x36, 0xf8, 0x2a, 0xca, 0xec, 0x16, 0xbd, 0xd4, 0x53, 0x6d, 0x1d, 0xca, 0xa1, 0xcd, 0x4b, 0xa0, 0x66, 0xb7, 0xba, 0xb4, 0x06, 0x7d, 0x72],
    (
        "0x182f021acc9256455e36f82acaec16bdd4536d1dcaa1cd4ba066b7bab4067d72",
        "0x1f326b433282b1e92c7012bf026405427d2490d9e28e6c01273f988255b9983c",
    )
)]
#[test_case(
    [0x13, 0xbc, 0xaa, 0xb8, 0x01, 0xe0, 0x94, 0x26, 0xf8, 0xda, 0xa9, 0x2b, 0xf2, 0xca, 0x83, 0x28, 0x2a, 0xe3, 0xed, 0x70, 0xcc, 0x8c, 0x27, 0x7a, 0xa3, 0x44, 0xb8, 0xfe, 0xb9, 0x72, 0x81, 0x8c],
    (
        "0x13bcaab801e09426f8daa92bf2ca83282ae3ed70cc8c277aa344b8feb972818c",
        "0xc1abae2e25a39dcc9054e7122a7403439b3a0f0ebbddfe3c1ac26c8023f2cc",
    )
)]
#[test_case(
    [0x1d, 0x87, 0x67, 0x17, 0x6e, 0xc6, 0xd9, 0x75, 0x96, 0xd0, 0x4e, 0x6b, 0xd7, 0x02, 0x4a, 0xa1, 0xcf, 0x32, 0x59, 0x50, 0x89, 0xb6, 0x45, 0x17, 0xa4, 0x3c, 0xd1, 0x0c, 0x1f, 0x99, 0x01, 0xbf],
    (
        "0x1d8767176ec6d97596d04e6bd7024aa1cf32595089b64517a43cd10c1f9901bf",
        "0x2302a48e833e3c702ed1eb82eebf8edb8b2cd48e03516061723ee87430a8d06a",
    )
)]
#[test_case(
    [0x06, 0xfd, 0x22, 0x53, 0x3c, 0xcd, 0x75, 0x7c, 0xb6, 0xdb, 0xfd, 0x1d, 0x32, 0xbe, 0xbb, 0x29, 0x30, 0x3d, 0xa7, 0x1b, 0xc3, 0x34, 0x6b, 0x96, 0xf8, 0x76, 0x6e, 0x7e, 0xbd, 0xbf, 0x04, 0x61],
    (
        "0x6fd22533ccd757cb6dbfd1d32bebb29303da71bc3346b96f8766e7ebdbf0461",
        "0x246972fcae26d8b93dc62ffc2462c50cdfe22af01ca802f4072349ea4823b34e",
    )
)]
#[test_case(
    [0x2d, 0x8f, 0x77, 0x8c, 0xfe, 0xbd, 0x03, 0x80, 0x95, 0xf3, 0x03, 0x8d, 0xdf, 0x86, 0x25, 0x44, 0xf8, 0x79, 0x8a, 0x64, 0xfe, 0x42, 0x33, 0x04, 0x7a, 0x2d, 0x29, 0x0e, 0xef, 0x4f, 0xf5, 0xd6],
    (
        "0x2d8f778cfebd038095f3038ddf862544f8798a64fe4233047a2d290eef4ff5d6",
        "0x27d0fab4ecf2e55a22af1f3f4d44e79ab133a2b9baecc4a59ae044d1c4ca4d52",
    )
)]
#[test_case(
    [0x01, 0x55, 0x8a, 0xd8, 0xdd, 0xf9, 0x24, 0xe5, 0x75, 0x8f, 0x4c, 0x29, 0x4b, 0x56, 0xfc, 0x27, 0x1f, 0xb2, 0xa8, 0x38, 0x2a, 0xaa, 0x86, 0x0d, 0x94, 0x3f, 0x52, 0x49, 0xa5, 0xe7, 0x63, 0x34],
    (
        "0x1558ad8ddf924e5758f4c294b56fc271fb2a8382aaa860d943f5249a5e76334",
        "0x155645307359f5a7a6c304a353715fd8023809355a255daf4854f026d92693dc",
    )
)]
#[test_case(
    [0x1e, 0x3c, 0xd8, 0xb4, 0xa8, 0x9e, 0x71, 0x63, 0x8a, 0xbf, 0xb0, 0x10, 0xe7, 0xfc, 0x77, 0x9c, 0xe9, 0x59, 0xe6, 0x39, 0x66, 0x73, 0x26, 0x37, 0x12, 0x83, 0x7c, 0xf0, 0xed, 0x76, 0x63, 0x91],
    (
        "0x1e3cd8b4a89e71638abfb010e7fc779ce959e6396673263712837cf0ed766391",
        "0x11d80dd5dd67cc67fcfe9bed95cfc2c4101526932088c041239885d723f0bc86",
    )
)]
#[test_case(
    [0x22, 0x60, 0x1d, 0x97, 0x77, 0x53, 0x1c, 0x8c, 0x68, 0x75, 0x89, 0x8e, 0x47, 0xff, 0xd8, 0x04, 0xb2, 0x21, 0x4e, 0xc2, 0x41, 0x1f, 0x40, 0x43, 0x25, 0x9c, 0x45, 0x76, 0xc7, 0x4e, 0x75, 0xfb],
    (
        "0x22601d9777531c8c6875898e47ffd804b2214ec2411f4043259c4576c74e75fb",
        "0xe61684d4e5ffd436cc8cc5cf7de02d8a3c373b396b72f9692e6694b998164c2",
    )
)]
#[test_case(
    [0x27, 0x01, 0xa9, 0xee, 0x7c, 0x63, 0x65, 0xce, 0x47, 0x75, 0x21, 0x31, 0xe9, 0xb9, 0xdd, 0x46, 0xb4, 0xc0, 0x48, 0xa9, 0x0d, 0x92, 0xf2, 0xe2, 0xf6, 0xae, 0xbb, 0x22, 0x6e, 0x58, 0xf0, 0x40],
    (
        "0x2701a9ee7c6365ce47752131e9b9dd46b4c048a90d92f2e2f6aebb226e58f040",
        "0x1875a82988dfe307569b5171ba09cc1707e1b03ecd3084f3251395a7894f0c22",
    )
)]
#[test_case(
    [0x08, 0xbb, 0x8c, 0x18, 0x75, 0x35, 0x03, 0x40, 0x36, 0x3e, 0xe4, 0x35, 0x02, 0xba, 0x73, 0xf6, 0x77, 0x73, 0x3f, 0x29, 0xb2, 0x25, 0x2d, 0xf1, 0x89, 0x72, 0xcc, 0x96, 0x4b, 0xd4, 0x62, 0xc5],
    (
        "0x8bb8c1875350340363ee43502ba73f677733f29b2252df18972cc964bd462c5",
        "0x299bdb13d2514ee0677704821b2d2748def74762a22fd2bf649ceb17b91f9996",
    )
)]
#[test_case(
    [0x01, 0xd7, 0x14, 0x30, 0x44, 0xba, 0x51, 0x3f, 0x92, 0x9f, 0xe7, 0x38, 0xd8, 0x0b, 0xd8, 0x4a, 0x45, 0x5e, 0x2b, 0xa9, 0x93, 0x59, 0x87, 0xaa, 0x7f, 0x8c, 0xf7, 0x7e, 0xc1, 0x8c, 0xf2, 0x0b],
    (
        "0x1d7143044ba513f929fe738d80bd84a455e2ba9935987aa7f8cf77ec18cf20b",
        "0x1a4e9dedf0f06f7c20e2c324c84e74f58b2c4845c3642ec20b3eb683e75b6edc",
    )
)]
#[test_case(
    [0x07, 0x8a, 0x2a, 0x4c, 0x2e, 0xea, 0x7a, 0x70, 0x4a, 0x12, 0x05, 0xd4, 0x96, 0xdb, 0x94, 0x62, 0xe1, 0xee, 0xb1, 0xcb, 0x30, 0xcd, 0xd8, 0xf9, 0xc3, 0xc6, 0xca, 0x42, 0x3a, 0xdc, 0x61, 0xca],
    (
        "0x78a2a4c2eea7a704a1205d496db9462e1eeb1cb30cdd8f9c3c6ca423adc61ca",
        "0x5e7dd7f679c96cdf9509fb3d4db66af20f28a8dcc2622f1c90419110b465828",
    )
)]
#[test_case(
    [0x1d, 0xbe, 0x17, 0xe6, 0x50, 0x79, 0x12, 0x6c, 0xaf, 0x00, 0x09, 0x7b, 0xdf, 0x54, 0x7c, 0x44, 0x57, 0xc6, 0x15, 0x1f, 0x4c, 0x6b, 0x90, 0xf4, 0xdc, 0x54, 0x24, 0xf5, 0x66, 0xdf, 0x0e, 0xe7],
    (
        "0x1dbe17e65079126caf00097bdf547c4457c6151f4c6b90f4dc5424f566df0ee7",
        "0x1f2fcbc14e2148084446caa954a2e0765dcf8af48e69dc186de83efa94fc806e",
    )
)]
#[test_case(
    [0x2d, 0xec, 0xa9, 0x23, 0x55, 0x5c, 0x5c, 0xfc, 0xa7, 0x97, 0x2d, 0xb2, 0xb8, 0x38, 0xb5, 0x68, 0xef, 0xef, 0x51, 0xfa, 0x44, 0x72, 0x4c, 0x66, 0x4c, 0xc0, 0x45, 0x2a, 0xb9, 0xff, 0x7d, 0x63],
    (
        "0x2deca923555c5cfca7972db2b838b568efef51fa44724c664cc0452ab9ff7d63",
        "0x2d47b4ffdebf3532efc6490bab958d393fd785dd02bbc2b03fc24a9678ee304",
    )
)]
#[test_case(
    [0x01, 0x98, 0x31, 0xeb, 0xf6, 0xa1, 0x58, 0x81, 0x45, 0x57, 0xfe, 0x02, 0x9a, 0x45, 0x37, 0xd5, 0xbf, 0x0f, 0xa3, 0xee, 0x84, 0xba, 0x43, 0x56, 0xe0, 0xe5, 0x98, 0x5f, 0x11, 0x29, 0x4a, 0xa4],
    (
        "0x19831ebf6a158814557fe029a4537d5bf0fa3ee84ba4356e0e5985f11294aa4",
        "0x2c8cad155eebb55bfd8492efade248ccee03202f5efb361904f74bbba5a17410",
    )
)]
#[test_case(
    [0x2c, 0x72, 0xbe, 0xfc, 0x77, 0xa2, 0x88, 0xdb, 0xc8, 0x9f, 0xd6, 0x11, 0xcf, 0x22, 0x73, 0x5c, 0x64, 0x4a, 0x34, 0xad, 0x2b, 0xb0, 0x49, 0x20, 0xd1, 0x62, 0x96, 0xc8, 0x77, 0x0e, 0x81, 0x7b],
    (
        "0x2c72befc77a288dbc89fd611cf22735c644a34ad2bb04920d16296c8770e817b",
        "0x68230ca4c35321ef952a1336e48b5b99f74baeb5b4ff2ad6481e652274eb984",
    )
)]
#[test_case(
    [0x28, 0x26, 0x3b, 0xc4, 0xaa, 0xed, 0xbc, 0xcc, 0xd2, 0xf6, 0x87, 0xa2, 0x05, 0x60, 0x48, 0x3c, 0x6a, 0x0a, 0xee, 0x2c, 0x89, 0x49, 0x74, 0x45, 0x75, 0xad, 0xc1, 0xf8, 0x1d, 0x4f, 0x72, 0xda],
    (
        "0x28263bc4aaedbcccd2f687a20560483c6a0aee2c8949744575adc1f81d4f72da",
        "0x1bfd7ddceed64b57a5b069ae92b2e7d56706c7a9dc56f4bbd82d986ee95603c6",
    )
)]
#[test_case(
    [0x26, 0x38, 0x46, 0x0e, 0x75, 0x6c, 0x86, 0x2b, 0x10, 0xbe, 0x8b, 0xda, 0x35, 0x13, 0x9a, 0xa7, 0xa6, 0x80, 0xcb, 0xf8, 0xab, 0x74, 0x0a, 0x1a, 0xdc, 0xa9, 0x6e, 0xd2, 0x2e, 0xf0, 0xb1, 0x6f],
    (
        "0x2638460e756c862b10be8bda35139aa7a680cbf8ab740a1adca96ed22ef0b16f",
        "0xde2e2c86e45de9a933cc052dd17707b8b5309e39914a0f9c79c134d76147b04",
    )
)]
#[test_case(
    [0x05, 0x52, 0x6b, 0xc9, 0xa0, 0xd7, 0x16, 0xe6, 0x66, 0x70, 0xd2, 0x31, 0x9e, 0x04, 0x1e, 0x46, 0xc9, 0x41, 0x64, 0xc4, 0x1c, 0x0c, 0xa7, 0x12, 0xe8, 0x11, 0x7b, 0x1a, 0xf6, 0x46, 0x76, 0xf8],
    (
        "0x5526bc9a0d716e66670d2319e041e46c94164c41c0ca712e8117b1af64676f8",
        "0x25e29f0a37ed937d2c226650fc78037790ea9c6d80baef29fa4539438bf853d8",
    )
)]
#[test_case(
    [0x2b, 0x70, 0xd4, 0xe9, 0x4d, 0x8e, 0x35, 0x49, 0xd6, 0x09, 0x53, 0xf5, 0x18, 0x65, 0x9b, 0xb8, 0x54, 0xf2, 0x22, 0x7c, 0x5a, 0x88, 0xde, 0x27, 0xdb, 0x77, 0x70, 0x51, 0x8e, 0xd3, 0xe9, 0x31],
    (
        "0x2b70d4e94d8e3549d60953f518659bb854f2227c5a88de27db7770518ed3e931",
        "0x1c844b23b907c8244677b9bd1b3c64edaa47ac04537c19762d7b38d43cd1f148",
    )
)]
#[test_case(
    [0x2e, 0x53, 0xd1, 0xcd, 0xd2, 0x8c, 0x7f, 0x6c, 0x2e, 0xa0, 0xc3, 0xc3, 0x2e, 0xea, 0x02, 0x28, 0x51, 0x10, 0xeb, 0xb9, 0xcc, 0x50, 0x7a, 0xa0, 0xc1, 0x2f, 0x1e, 0x33, 0xf7, 0x6e, 0xdf, 0x74],
    (
        "0x2e53d1cdd28c7f6c2ea0c3c32eea02285110ebb9cc507aa0c12f1e33f76edf74",
        "0x1a87019d19b0108728fa9e79c9083ad6e8688989f09920354380ac2721827282",
    )
)]
#[test_case(
    [0x15, 0x69, 0x8b, 0x00, 0x9f, 0x37, 0xf8, 0xa4, 0x64, 0x62, 0xdb, 0xc2, 0x68, 0x8f, 0xff, 0xee, 0xb6, 0x78, 0x71, 0x30, 0xd8, 0xc1, 0xd6, 0xb2, 0x6d, 0x5b, 0xf8, 0xa1, 0x8d, 0x56, 0x27, 0xbf],
    (
        "0x15698b009f37f8a46462dbc2688fffeeb6787130d8c1d6b26d5bf8a18d5627bf",
        "0x189c4535196083e9a50d66efef26e4d599879d0244ddc08978315df04b41533a",
    )
)]
#[test_case(
    [0x23, 0xa4, 0x90, 0x60, 0xff, 0xf2, 0xef, 0x43, 0x29, 0xbc, 0xe4, 0x7d, 0x12, 0x7a, 0x0c, 0x11, 0x3d, 0x66, 0xf4, 0xf8, 0xc1, 0x4f, 0x23, 0x94, 0xc7, 0x97, 0x6c, 0xff, 0x59, 0x5b, 0xd5, 0xaa],
    (
        "0x23a49060fff2ef4329bce47d127a0c113d66f4f8c14f2394c7976cff595bd5aa",
        "0x11cce022cb4450c53e53839a3940567784be6fb67d6f561966b9dee65f987a3e",
    )
)]
#[test_case(
    [0x23, 0x32, 0x9b, 0x82, 0x1e, 0x5f, 0xbb, 0xa3, 0x02, 0x01, 0x27, 0xe3, 0x39, 0x28, 0xe4, 0xf2, 0x50, 0x48, 0x06, 0x17, 0xf9, 0x17, 0x39, 0x96, 0x60, 0xbd, 0x7b, 0x08, 0xb9, 0x28, 0x8b, 0x48],
    (
        "0x23329b821e5fbba3020127e33928e4f250480617f917399660bd7b08b9288b48",
        "0x197578bd1194ee1d207e9b0d2e4f2e55e4512a1e818dcf3556176ff017192dc2",
    )
)]
#[test_case(
    [0x1f, 0x15, 0xd8, 0xcc, 0x8d, 0x7e, 0x53, 0x64, 0xd5, 0xac, 0x6e, 0xa0, 0xd3, 0x23, 0x12, 0x12, 0x76, 0xb1, 0x45, 0xbd, 0xdf, 0x05, 0x68, 0x1d, 0x2f, 0xcc, 0x3d, 0x2f, 0xe5, 0x77, 0x23, 0xf4],
    (
        "0x1f15d8cc8d7e5364d5ac6ea0d323121276b145bddf05681d2fcc3d2fe57723f4",
        "0x17ddaa99c0dbdfcc568d7b05a25a7f16a64f16eb2e86b6c8f7ec2b9998887792",
    )
)]
#[test_case(
    [0x07, 0x76, 0x7c, 0x7e, 0x57, 0x22, 0xba, 0x85, 0x87, 0x91, 0x20, 0x15, 0x4f, 0x58, 0xaa, 0x16, 0xe2, 0xdb, 0x21, 0x75, 0x79, 0xea, 0x1d, 0x3d, 0xf7, 0x66, 0xbc, 0x4c, 0xad, 0xea, 0xc5, 0x4c],
    (
        "0x7767c7e5722ba85879120154f58aa16e2db217579ea1d3df766bc4cadeac54c",
        "0x871640dd8539208c68524db2071d814214b0c5c6e7de73aaacd2afcddb6ffc8",
    )
)]
#[test_case(
    [0x29, 0x88, 0x22, 0x5b, 0x7c, 0x44, 0xf9, 0xe5, 0x06, 0xbb, 0xfe, 0x85, 0x39, 0xcd, 0x26, 0xc8, 0xb9, 0xb8, 0xec, 0xf3, 0xec, 0xab, 0x33, 0x1d, 0x86, 0x95, 0xad, 0xf3, 0x5e, 0x3f, 0xda, 0x07],
    (
        "0x2988225b7c44f9e506bbfe8539cd26c8b9b8ecf3ecab331d8695adf35e3fda07",
        "0x2f44627a25fb04aa6b44300e7421bf3ee3196d0538e5ae2b210546f783f2cd32",
    )
)]
#[test_case(
    [0x2b, 0x5e, 0x49, 0x45, 0xc0, 0x74, 0x9b, 0xf9, 0xe1, 0x7e, 0x4d, 0x1d, 0xea, 0xc5, 0xe4, 0x89, 0x21, 0xb4, 0xc2, 0x82, 0xee, 0x45, 0x08, 0x8a, 0x7b, 0xf9, 0x6a, 0x1b, 0xc5, 0x42, 0xb5, 0x71],
    (
        "0x2b5e4945c0749bf9e17e4d1deac5e48921b4c282ee45088a7bf96a1bc542b571",
        "0x28b3a31a05ada890146c5e0227de13e27baa96bc781d5e5bcd6109b713d5f758",
    )
)]
#[test_case(
    [0x23, 0x55, 0x02, 0x2c, 0x52, 0x57, 0x66, 0x1b, 0xfd, 0xce, 0xbe, 0xd1, 0xc7, 0xad, 0x0e, 0x22, 0x96, 0xa3, 0x3d, 0x4b, 0xe3, 0x05, 0x4d, 0x73, 0x85, 0x3c, 0xf6, 0x3d, 0x60, 0xec, 0x45, 0x70],
    (
        "0x2355022c5257661bfdcebed1c7ad0e2296a33d4be3054d73853cf63d60ec4570",
        "0xfcfaf1ef9839f6f5598a916ae7c28fc28bc0c987c5c3f9e170159e322c0ec58",
    )
)]
#[test_case(
    [0x26, 0x05, 0x25, 0x5c, 0x41, 0x1a, 0x24, 0x88, 0x26, 0x14, 0xa9, 0x47, 0x8e, 0xd2, 0x66, 0x76, 0x22, 0xed, 0xa5, 0xd8, 0xb1, 0xc8, 0x12, 0xc8, 0x2b, 0xd3, 0x8a, 0xac, 0xf9, 0x7b, 0x46, 0xff],
    (
        "0x2605255c411a24882614a9478ed2667622eda5d8b1c812c82bd38aacf97b46ff",
        "0x20df728f992c0dbd084c479155b1b8cc4b0ace3bc7b7d31d671ed49d568b2d1a",
    )
)]
#[test_case(
    [0x15, 0x84, 0x15, 0x4f, 0xf4, 0xec, 0xd3, 0x96, 0x7f, 0x84, 0x94, 0xfa, 0x33, 0xe9, 0x4f, 0x0e, 0x69, 0xb6, 0x9e, 0xb7, 0x50, 0xec, 0xe2, 0x14, 0x83, 0x78, 0xda, 0xbc, 0xba, 0xd3, 0x8c, 0x05],
    (
        "0x1584154ff4ecd3967f8494fa33e94f0e69b69eb750ece2148378dabcbad38c05",
        "0x51ad519b5257340edff33212434e68c5b43486ceed9211469506621abd5ece",
    )
)]
#[test_case(
    [0x1b, 0x73, 0x05, 0x71, 0x5a, 0xdb, 0xc0, 0x99, 0xeb, 0xeb, 0xf9, 0x2d, 0x7d, 0xa0, 0x9e, 0x04, 0xfd, 0x2f, 0x85, 0x9f, 0xd7, 0x61, 0x3d, 0xc2, 0x60, 0xb0, 0x8d, 0x76, 0x73, 0x7c, 0x65, 0x4d],
    (
        "0x1b7305715adbc099ebebf92d7da09e04fd2f859fd7613dc260b08d76737c654d",
        "0x14a4f76f9fbcc89c48a1f88f7603d551d1fb5e46c539d17cf11b18a73265a4ec",
    )
)]
#[test_case(
    [0x15, 0xa6, 0xf1, 0x77, 0xcb, 0xa6, 0x73, 0x5b, 0x75, 0x5b, 0xdd, 0x33, 0xd4, 0x93, 0xe8, 0xa2, 0xe1, 0xd7, 0xe4, 0x16, 0x05, 0x40, 0xb1, 0x57, 0xca, 0x70, 0x37, 0x82, 0xeb, 0x72, 0x16, 0xf5],
    (
        "0x15a6f177cba6735b755bdd33d493e8a2e1d7e4160540b157ca703782eb7216f5",
        "0x143b33d4f6e8b2ec4b4438e76b334e959a2094ba76693e10fb6e0bbbe1b56a52",
    )
)]
#[test_case(
    [0x1c, 0x79, 0x7f, 0xe1, 0x35, 0x4c, 0x09, 0x27, 0xcf, 0x6b, 0x44, 0xe7, 0xa0, 0x3e, 0xe9, 0x51, 0xb5, 0x89, 0xf7, 0x3d, 0x53, 0x7f, 0xa7, 0x93, 0xcb, 0xf6, 0xc4, 0xdd, 0x36, 0x6a, 0x1c, 0x9a],
    (
        "0x1c797fe1354c0927cf6b44e7a03ee951b589f73d537fa793cbf6c4dd366a1c9a",
        "0x2ac21244ff5ef8ce3981c171cad186d69533485c3e58e5dec28814a6a528491c",
    )
)]
#[test_case(
    [0x2c, 0x97, 0x85, 0xc9, 0x60, 0x78, 0xde, 0x3b, 0xdc, 0x3d, 0x95, 0x34, 0x75, 0x81, 0x32, 0x94, 0x45, 0x57, 0x0e, 0x46, 0xc9, 0x7f, 0xc5, 0x42, 0xad, 0x8b, 0xcd, 0xf1, 0xd7, 0x27, 0x42, 0xd8],
    (
        "0x2c9785c96078de3bdc3d95347581329445570e46c97fc542ad8bcdf1d72742d8",
        "0x14fc6cf9920b68bca02cd2223108a03fb4c8aa5fbb9335de3d7b74be069700e6",
    )
)]
#[test_case(
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
    ("0x1", "0x2")
)]
fn decompress_collection_id_works(collection_id: CombinatorialId, expected: (&str, &str)) {
    let x = Fq::from_hex_str(expected.0);
    let y = Fq::from_hex_str(expected.1);
    let expected = G1Affine::new(x, y);

    let actual = decompress_collection_id(collection_id).unwrap();
    assert_eq!(actual, expected);
}

#[test_case(
    [0x64, 0xf3, 0x41, 0x72, 0x9b, 0xea, 0x43, 0x90, 0x10, 0x1d, 0x0b, 0x1a, 0xcc, 0x67, 0x47, 0xbc, 0x0d, 0x8d, 0x1a, 0xc5, 0x9f, 0xf0, 0xb3, 0x2f, 0xe9, 0x91, 0x94, 0x93, 0x8b, 0x70, 0xa4, 0xda]
)]
#[test_case(
    [0x71, 0xfb, 0x5a, 0x00, 0x74, 0xc4, 0xfd, 0xf8, 0xff, 0x2a, 0x59, 0x75, 0x0c, 0xb7, 0x25, 0x7c, 0x60, 0x6b, 0x5d, 0x09, 0x93, 0xf0, 0xe7, 0x9c, 0x33, 0x08, 0x84, 0x72, 0xbc, 0x98, 0xb0, 0xf2]
)]
#[test_case(
    [0xed, 0x84, 0xb8, 0xdd, 0xca, 0x0c, 0x1b, 0x21, 0x42, 0x48, 0x4f, 0x42, 0x0e, 0x05, 0xba, 0x7b, 0x19, 0xa7, 0x91, 0xc4, 0xd9, 0x17, 0x5f, 0x4f, 0x49, 0xf7, 0x83, 0x6f, 0xf1, 0xfa, 0xba, 0xae]
)]
#[test_case(
    [0x93, 0x0c, 0xf8, 0x88, 0x63, 0xc7, 0x0c, 0x66, 0x28, 0x5f, 0x4b, 0x96, 0x17, 0x10, 0x32, 0x65, 0x3d, 0x91, 0xc5, 0x0e, 0xcf, 0xda, 0x23, 0xf1, 0x82, 0x7e, 0x6a, 0x9b, 0x16, 0xb1, 0x50, 0x95]
)]
#[test_case(
    [0x72, 0x7a, 0xf5, 0x5e, 0x17, 0x46, 0xa7, 0x00, 0xd1, 0xde, 0x3e, 0x03, 0x99, 0x92, 0x91, 0x20, 0xdd, 0xf7, 0xae, 0xff, 0xb3, 0x2d, 0xd9, 0x53, 0x18, 0xdc, 0xf5, 0x4d, 0x39, 0x44, 0xa3, 0xd8]
)]
#[test_case(
    [0xd5, 0x7f, 0xa0, 0x9a, 0x3f, 0xa7, 0xaf, 0xb2, 0x1c, 0x94, 0xb0, 0x3b, 0x06, 0x65, 0xd3, 0x59, 0x5f, 0xa8, 0x48, 0x18, 0x7e, 0x68, 0xe2, 0xbc, 0x01, 0x0b, 0xfc, 0x16, 0xb1, 0x65, 0x55, 0x63]
)]
#[test_case(
    [0x1a, 0x04, 0x7d, 0x43, 0x00, 0xd3, 0x6f, 0xb3, 0xea, 0xef, 0x0b, 0x27, 0x71, 0xf4, 0x54, 0x02, 0xf4, 0x05, 0xd9, 0x90, 0x84, 0x08, 0x7a, 0xd3, 0xd9, 0x59, 0xfb, 0x0d, 0x3f, 0x4d, 0x7d, 0xf4]
)]
#[test_case(
    [0xbc, 0x15, 0xb3, 0x40, 0x0d, 0xe4, 0x0a, 0xd4, 0x96, 0x68, 0x98, 0x6a, 0xca, 0xb4, 0xf2, 0xa6, 0x2b, 0x5c, 0x8e, 0x18, 0x3e, 0x22, 0xd1, 0xa1, 0xe3, 0x52, 0xa8, 0x86, 0xc6, 0x56, 0xc2, 0xa9]
)]
#[test_case(
    [0x59, 0x87, 0x2c, 0xc4, 0x34, 0x24, 0x80, 0x20, 0x47, 0xf5, 0xc6, 0xda, 0x00, 0x9d, 0xad, 0xc6, 0x48, 0x74, 0x74, 0x10, 0xf0, 0xc7, 0x70, 0x92, 0x7b, 0xe3, 0x9a, 0x1e, 0x47, 0x29, 0x76, 0xe1]
)]
#[test_case(
    [0xa6, 0xf3, 0x83, 0x53, 0x08, 0x5f, 0x48, 0xaa, 0x67, 0x65, 0x24, 0xdc, 0x50, 0x50, 0x20, 0x76, 0x2c, 0x14, 0xc6, 0x11, 0x2e, 0xd2, 0x94, 0x87, 0xcf, 0x0e, 0x23, 0x3b, 0x32, 0xc5, 0xc2, 0x88]
)]
#[test_case(
    [0x66, 0x61, 0x64, 0x78, 0xd5, 0xa0, 0xad, 0xeb, 0x87, 0x0a, 0x9e, 0x88, 0xb9, 0x1e, 0xe4, 0x77, 0xb1, 0x76, 0x81, 0x63, 0xd8, 0xea, 0x8d, 0x4c, 0x7e, 0x54, 0x33, 0xd4, 0x07, 0xf8, 0x78, 0x50]
)]
#[test_case(
    [0x70, 0x8e, 0x06, 0xc5, 0xdf, 0xbf, 0x31, 0x86, 0xf1, 0x25, 0xa4, 0xb2, 0x78, 0x8a, 0x96, 0x61, 0x6f, 0x76, 0xa6, 0x1f, 0xa7, 0x92, 0x5b, 0xec, 0xd0, 0xab, 0xa7, 0xd1, 0xde, 0x77, 0xe0, 0xd7]
)]
#[test_case(
    [0x63, 0x76, 0x07, 0xf0, 0xe1, 0x22, 0xde, 0xca, 0x26, 0x3d, 0x6a, 0xba, 0x24, 0xd2, 0x5d, 0x72, 0xc0, 0x1c, 0x52, 0x1b, 0x52, 0x2c, 0x2b, 0xfb, 0x38, 0x9a, 0x7c, 0xac, 0xd6, 0x47, 0xcd, 0x30]
)]
#[test_case(
    [0xb3, 0xd6, 0x11, 0x5a, 0x46, 0xcd, 0x0b, 0x52, 0xd8, 0xac, 0xe6, 0xb4, 0x21, 0x3f, 0x1a, 0x1b, 0x52, 0x38, 0xfd, 0x51, 0x01, 0x0a, 0x11, 0x84, 0x5e, 0xb2, 0x03, 0xdb, 0xf4, 0xad, 0x2c, 0x47]
)]
#[test_case(
    [0xcd, 0x46, 0xba, 0x18, 0x6b, 0x98, 0xe0, 0x47, 0xff, 0x70, 0x8c, 0xf3, 0xf4, 0x3e, 0x55, 0x25, 0x63, 0x2b, 0x62, 0x47, 0x76, 0xd5, 0xdb, 0xe6, 0xf2, 0xa0, 0x01, 0x13, 0x15, 0x5e, 0x3b, 0xb3]
)]
#[test_case(
    [0x98, 0xa9, 0x27, 0x46, 0xc5, 0x6e, 0x65, 0x9b, 0x12, 0x0b, 0x0a, 0x23, 0xed, 0x39, 0x59, 0x33, 0x70, 0x8e, 0x12, 0xd5, 0x89, 0x8f, 0x10, 0x25, 0xb3, 0x8e, 0xb5, 0xfb, 0x03, 0xf2, 0x2d, 0x8e]
)]
#[test_case(
    [0x03, 0x9c, 0xe1, 0x34, 0x24, 0x43, 0x6f, 0xd6, 0xf1, 0xe5, 0xb8, 0x98, 0x2a, 0x8a, 0xea, 0xb0, 0x74, 0xf4, 0xeb, 0x5e, 0xfa, 0x05, 0x5f, 0x8c, 0x1f, 0x1b, 0xf9, 0xef, 0x20, 0xe9, 0x90, 0xbd]
)]
#[test_case(
    [0x64, 0xd3, 0x87, 0xc1, 0x6e, 0x52, 0x10, 0xe3, 0xe4, 0x8a, 0x7f, 0x07, 0x5d, 0x70, 0xd9, 0x2d, 0x19, 0xe9, 0xcc, 0x94, 0x66, 0x7a, 0x7f, 0x6a, 0x95, 0x36, 0xd0, 0xd9, 0x4c, 0x5b, 0xc4, 0xd7]
)]
#[test_case(
    [0x96, 0x74, 0x61, 0x31, 0xcf, 0xcc, 0x2d, 0xcc, 0x27, 0xd0, 0x46, 0xc2, 0x46, 0x2d, 0x10, 0xa7, 0xa4, 0xb8, 0x1f, 0x5b, 0xe6, 0xc9, 0xd5, 0xc7, 0x69, 0x1f, 0xad, 0x1f, 0x34, 0x89, 0x05, 0xee]
)]
#[test_case(
    [0x7f, 0x23, 0x8a, 0x24, 0x2f, 0xf8, 0xbe, 0x73, 0xfb, 0xd4, 0x68, 0x5e, 0x36, 0xe7, 0x64, 0xd4, 0xf0, 0x25, 0x7a, 0xb8, 0x47, 0x6e, 0x51, 0x13, 0x18, 0xa5, 0x07, 0xc9, 0x21, 0x2c, 0xb1, 0x73]
)]
#[test_case(
    [0x34, 0x67, 0x08, 0xf1, 0x00, 0x8c, 0xe1, 0x71, 0x7f, 0x00, 0x88, 0x08, 0xb8, 0xd3, 0xff, 0xb2, 0x15, 0x1d, 0xf3, 0xc2, 0xbb, 0x45, 0x2d, 0x63, 0x34, 0xda, 0x21, 0x90, 0xdd, 0xd6, 0xfb, 0x91]
)]
#[test_case(
    [0xe2, 0x70, 0xb9, 0x4f, 0xfc, 0xda, 0x54, 0x73, 0xd3, 0x9b, 0xf7, 0x23, 0xb1, 0xc3, 0x83, 0xf1, 0xe8, 0x01, 0xe8, 0xf7, 0x57, 0xb0, 0x9d, 0xf3, 0x27, 0xb5, 0x8b, 0xb6, 0x95, 0x3d, 0x78, 0xa8]
)]
#[test_case(
    [0x25, 0x79, 0x24, 0x89, 0x2e, 0x15, 0x34, 0x5c, 0xe7, 0xfa, 0x78, 0x15, 0x68, 0xf8, 0x23, 0x3d, 0x1d, 0x4e, 0xb8, 0x7c, 0xaf, 0xa8, 0x75, 0x04, 0x49, 0xaf, 0xd0, 0x39, 0x77, 0x7b, 0xbe, 0xac]
)]
#[test_case(
    [0x6b, 0x3b, 0x0a, 0x09, 0x43, 0x4d, 0x23, 0x0d, 0x4c, 0x6f, 0x93, 0xba, 0xe3, 0x02, 0xd7, 0x1b, 0xcc, 0xa5, 0x9e, 0xbb, 0x27, 0xb6, 0xa9, 0x66, 0xb3, 0x8f, 0x49, 0x06, 0x73, 0xbe, 0x79, 0xf1]
)]
#[test_case(
    [0x49, 0xaf, 0x83, 0x00, 0x60, 0x19, 0x13, 0x24, 0xea, 0x98, 0x1b, 0x1a, 0xf5, 0x84, 0x72, 0x02, 0xd3, 0x0f, 0x28, 0x80, 0xbd, 0xa0, 0x9d, 0x33, 0xc4, 0x49, 0xa2, 0xf5, 0x7b, 0xca, 0xe1, 0xfe]
)]
#[test_case(
    [0xd0, 0x38, 0x1e, 0xd6, 0xae, 0xd8, 0x85, 0xe2, 0x2d, 0x22, 0xdc, 0x10, 0x5e, 0x89, 0xc9, 0xc7, 0xc7, 0xba, 0x91, 0x7f, 0x98, 0xfe, 0x05, 0x59, 0xf0, 0xb6, 0x2e, 0xed, 0x24, 0xc7, 0xf5, 0x58]
)]
#[test_case(
    [0x6b, 0xe4, 0xe5, 0x7d, 0x54, 0xf0, 0x48, 0xe0, 0x3f, 0x7e, 0xe5, 0x16, 0x91, 0x5d, 0x1c, 0xa2, 0x04, 0x4c, 0x08, 0x85, 0xf3, 0xe2, 0x50, 0x02, 0x73, 0x85, 0x65, 0x79, 0xde, 0x86, 0x5c, 0x75]
)]
#[test_case(
    [0x20, 0x24, 0x92, 0x76, 0xe9, 0x41, 0x79, 0x08, 0x75, 0x82, 0xcd, 0xe9, 0x15, 0x76, 0xa0, 0xba, 0x2a, 0x8d, 0x69, 0x9f, 0xca, 0xa3, 0xc5, 0xa6, 0x8a, 0xf6, 0xcd, 0xdb, 0xbe, 0x90, 0x6b, 0x17]
)]
#[test_case(
    [0x98, 0x1f, 0x5b, 0x9d, 0x34, 0x7e, 0x79, 0xe6, 0x71, 0xe6, 0x25, 0xe8, 0xb1, 0xe2, 0xdc, 0x27, 0xa3, 0x90, 0x43, 0x14, 0xe3, 0xe5, 0x5e, 0x58, 0x7b, 0x8f, 0xab, 0x9f, 0x9c, 0x94, 0x03, 0x1f]
)]
#[test_case(
    [0x8e, 0x63, 0x3a, 0x31, 0xc5, 0x6d, 0x22, 0x8b, 0x4d, 0x55, 0xda, 0xbd, 0x4e, 0x2a, 0x9b, 0xae, 0xf6, 0x12, 0x4c, 0xf6, 0x56, 0x3d, 0xc8, 0x76, 0xb6, 0x33, 0x72, 0x48, 0x9a, 0x30, 0xfc, 0x3f]
)]
#[test_case(
    [0x41, 0xe0, 0x5f, 0x70, 0xa2, 0x15, 0x83, 0x7a, 0x69, 0x2a, 0x8e, 0x18, 0x5f, 0x7a, 0x99, 0xe5, 0x86, 0x21, 0x51, 0xbd, 0xe7, 0xe4, 0xf4, 0x72, 0xfa, 0x8b, 0xf8, 0x54, 0x5e, 0xf5, 0x85, 0xd7]
)]
#[test_case(
    [0x76, 0x67, 0x10, 0xb5, 0x92, 0xe8, 0x2f, 0xd1, 0xa8, 0x96, 0x8b, 0xb9, 0x13, 0x0f, 0x50, 0xe3, 0xda, 0xfa, 0xeb, 0x12, 0xce, 0xa4, 0x13, 0xe4, 0x5e, 0x31, 0xcd, 0x0c, 0x55, 0x08, 0xd4, 0x4e]
)]
#[test_case(
    [0x94, 0xb4, 0xbd, 0xbb, 0xcd, 0x3d, 0x9d, 0x7e, 0x3b, 0x90, 0x2c, 0x9d, 0x02, 0x73, 0xf8, 0x7a, 0x84, 0x51, 0x0e, 0x52, 0xa5, 0x8c, 0x75, 0xfe, 0xce, 0xf5, 0x00, 0x0d, 0xf5, 0x4c, 0x91, 0x85]
)]
#[test_case(
    [0x45, 0xe8, 0xf2, 0x5a, 0xfe, 0xf6, 0xfd, 0x7a, 0x2f, 0xf7, 0xcf, 0x6b, 0x05, 0x8b, 0x2d, 0xf9, 0x03, 0x5c, 0x76, 0x7a, 0x16, 0x1b, 0x55, 0x06, 0x39, 0x22, 0xdd, 0xc7, 0xa9, 0x55, 0xf7, 0x24]
)]
#[test_case(
    [0xa2, 0xc0, 0xdd, 0xe2, 0x1a, 0x63, 0xd8, 0xe7, 0x57, 0xa9, 0x98, 0x51, 0xd8, 0x79, 0xf6, 0xe2, 0xe5, 0x82, 0x60, 0x7b, 0xd2, 0x08, 0x80, 0xef, 0x64, 0xc8, 0x31, 0xc9, 0xa7, 0xce, 0x88, 0x00]
)]
#[test_case(
    [0xaa, 0x0a, 0x4e, 0xa0, 0xab, 0x4c, 0x0e, 0xbf, 0x66, 0x39, 0x9b, 0x36, 0xdc, 0xc1, 0x75, 0x9c, 0x0f, 0x00, 0x31, 0xb5, 0x45, 0xc5, 0x1d, 0xdc, 0x38, 0x45, 0x76, 0x53, 0x31, 0x07, 0x99, 0xa4]
)]
#[test_case(
    [0x4d, 0x0f, 0x2c, 0xf2, 0xd1, 0xfc, 0x52, 0x49, 0xc5, 0x21, 0xaa, 0xbf, 0xbf, 0x91, 0xb9, 0x13, 0xd1, 0xfb, 0x42, 0x19, 0x86, 0x0a, 0x35, 0x5e, 0x3a, 0x3a, 0xee, 0x76, 0xd9, 0x2d, 0x6d, 0xf9]
)]
#[test_case(
    [0xa2, 0xc3, 0xb5, 0xac, 0x07, 0xbd, 0x2f, 0x74, 0xf7, 0x98, 0x7e, 0x00, 0xe2, 0xaf, 0x52, 0x4f, 0x6a, 0x95, 0x07, 0xd4, 0x14, 0x93, 0x16, 0x87, 0xf6, 0xca, 0x42, 0x34, 0xe3, 0x7d, 0xf3, 0x2c]
)]
#[test_case(
    [0x99, 0x32, 0x74, 0x19, 0x77, 0x0f, 0x9b, 0x3d, 0x5d, 0x19, 0xce, 0xad, 0xcc, 0x06, 0xa5, 0x1d, 0x08, 0xe2, 0x86, 0x30, 0x4b, 0x61, 0xd5, 0x08, 0xcc, 0x36, 0xbc, 0x2e, 0x23, 0x5b, 0xf3, 0x05]
)]
#[test_case(
    [0x34, 0x7b, 0x86, 0xec, 0xe0, 0x00, 0x89, 0x2a, 0x2d, 0x84, 0x5b, 0x2b, 0x36, 0x82, 0x21, 0x63, 0x8a, 0x2a, 0x04, 0x82, 0x1d, 0x03, 0x2c, 0xe3, 0xef, 0xbc, 0xf7, 0xbe, 0x57, 0x44, 0x79, 0x59]
)]
#[test_case(
    [0x3a, 0xb4, 0x53, 0xb4, 0xf9, 0x53, 0xe1, 0x50, 0xaa, 0xb1, 0x57, 0xdd, 0x64, 0xd7, 0x85, 0x77, 0x9e, 0xeb, 0xe6, 0x00, 0xb8, 0x7f, 0xb6, 0xf8, 0xe4, 0x62, 0x1f, 0x41, 0x94, 0x41, 0x73, 0x57]
)]
#[test_case(
    [0xfc, 0x0c, 0xe9, 0xb2, 0xec, 0xb2, 0x50, 0xfb, 0xb9, 0x34, 0xbc, 0x5a, 0x74, 0x98, 0xe4, 0xc7, 0x62, 0x8d, 0x6f, 0x1f, 0x3a, 0x56, 0x69, 0x45, 0x47, 0x96, 0xbe, 0x15, 0xf8, 0x56, 0x31, 0x4e]
)]
#[test_case(
    [0xab, 0x5e, 0x49, 0x8f, 0xd0, 0xf7, 0x2c, 0xd8, 0x54, 0xed, 0xe6, 0x27, 0x20, 0x4c, 0x23, 0xd6, 0x39, 0xa1, 0x4a, 0x71, 0x4b, 0x35, 0xe7, 0xa8, 0x09, 0x1e, 0x0f, 0x04, 0xa4, 0xd7, 0x49, 0xb2]
)]
#[test_case(
    [0x62, 0x52, 0xa3, 0xde, 0xa6, 0x05, 0x54, 0x85, 0x65, 0xb6, 0x83, 0x8f, 0x85, 0x38, 0xee, 0xab, 0x9c, 0x8b, 0x66, 0x64, 0x90, 0x05, 0xc0, 0x17, 0x95, 0x9d, 0x0d, 0x2d, 0x20, 0xec, 0x2a, 0xa0]
)]
#[test_case(
    [0xbd, 0xd9, 0x27, 0xb4, 0x6b, 0x96, 0x7b, 0xc9, 0x3a, 0xc4, 0x61, 0x41, 0x8d, 0x5e, 0x66, 0xad, 0xf2, 0xde, 0x77, 0x58, 0x95, 0x42, 0x57, 0x45, 0x5b, 0x16, 0x5e, 0x40, 0x5f, 0x40, 0x25, 0xc9]
)]
#[test_case(
    [0x7f, 0x6f, 0x8b, 0xde, 0xf8, 0x24, 0x5a, 0x28, 0x10, 0x50, 0x4d, 0xa9, 0x8c, 0xe0, 0x59, 0x64, 0x5c, 0xa3, 0xa6, 0x27, 0x17, 0x79, 0x8e, 0x5c, 0x13, 0x5f, 0xbb, 0x5c, 0x13, 0x9a, 0x55, 0x59]
)]
#[test_case(
    [0x30, 0x65, 0x95, 0xba, 0xf3, 0xbc, 0x3a, 0x34, 0x1a, 0xb9, 0x42, 0xf6, 0x00, 0x94, 0xd9, 0x1e, 0xc5, 0x51, 0x4b, 0x1c, 0x53, 0x5a, 0x33, 0xca, 0x77, 0x03, 0x93, 0x12, 0x39, 0x99, 0x3c, 0x45]
)]
#[test_case(
    [0xef, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
)]
#[test_case(
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
)]
#[test_case(
    [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
)]
#[test_case(
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
)]
#[test_case(
    [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]
)]
fn decompress_collection_id_fails_on_invalid_collection_id(collection_id: CombinatorialId) {
    let actual = decompress_collection_id(collection_id);
    assert_eq!(actual, None);
}
