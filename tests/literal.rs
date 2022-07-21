use litter::AnyLiteral;

static SOME_BYTES: &[u8] = &[
    0x00, 0x02, 0x04, 0x03, 0x06, 0x08, 0x07, 0x09, 0x0D, 0x0C, 0x0E, 0x10, 0x10, 0x12, 0x14, 0x13,
    0x17, 0x19, 0x18, 0x1A, 0x1D, 0x1C, 0x1E, 0x20, 0x21, 0x23, 0x25, 0x24, 0x27, 0x29, 0x28, 0x2A,
    0x2E, 0x2D, 0x2F, 0x31, 0x31, 0x33, 0x35, 0x34, 0x38, 0x3A, 0x39, 0x3B, 0x3E, 0x3D, 0x3F, 0x41,
    0x42, 0x44, 0x46, 0x45, 0x48, 0x4A, 0x49, 0x4B, 0x4F, 0x4E, 0x50, 0x52, 0x52, 0x54, 0x56, 0x55,
    0x59, 0x5B, 0x5A, 0x5C, 0x5F, 0x5E, 0x60, 0x62, 0x63, 0x65, 0x67, 0x66, 0x69, 0x6B, 0x6A, 0x6C,
    0x70, 0x6F, 0x71, 0x73, 0x73, 0x75, 0x77, 0x76, 0x7A, 0x7C, 0x7B, 0x7D, 0x80, 0x7F, 0x81, 0x83,
    0x84, 0x86, 0x88, 0x87, 0x8A, 0x8C, 0x8B, 0x8D, 0x91, 0x90, 0x92, 0x94, 0x94, 0x96, 0x98, 0x97,
    0x9B, 0x9D, 0x9C, 0x9E, 0xA1, 0xA0, 0xA2, 0xA4, 0xA5, 0xA7, 0xA9, 0xA8, 0xAB, 0xAD, 0xAC, 0xAE,
    0xB2, 0xB1, 0xB3, 0xB5, 0xB5, 0xB7, 0xB9, 0xB8, 0xBC, 0xBE, 0xBD, 0xBF, 0xC2, 0xC1, 0xC3, 0xC5,
    0xC6, 0xC8, 0xCA, 0xC9, 0xCC, 0xCE, 0xCD, 0xCF, 0xD3, 0xD2, 0xD4, 0xD6, 0xD6, 0xD8, 0xDA, 0xD9,
    0xDD, 0xDF, 0xDE, 0xE0, 0xE3, 0xE2, 0xE4, 0xE6, 0xE7, 0xE9, 0xEB, 0xEA, 0xED, 0xEF, 0xEE, 0xF0,
    0xF4, 0xF3, 0xF5, 0xF7, 0xF7, 0xF9, 0xFB, 0xFA, 0xFE, 0x00, 0xFF, 0x01, 0x04, 0x03, 0x05, 0x07,
    0x08, 0x0A, 0x0C, 0x0B, 0x0E, 0x10, 0x0F, 0x11, 0x15, 0x14, 0x16, 0x18, 0x18, 0x1A, 0x1C, 0x1B,
    0x1F, 0x21, 0x20, 0x22, 0x25, 0x24, 0x26, 0x28, 0x29, 0x2B, 0x2D, 0x2C, 0x2F, 0x31, 0x30, 0x32,
    0x36, 0x35, 0x37, 0x39, 0x39, 0x3B, 0x3D, 0x3C, 0x40, 0x42, 0x41, 0x43, 0x46, 0x45, 0x47, 0x49,
    0x4A, 0x4C, 0x4E, 0x4D, 0x50, 0x52, 0x51, 0x53, 0x57, 0x56, 0x58, 0x5A, 0x5A, 0x5C, 0x5E, 0x5D,
    0x61, 0x63, 0x62, 0x64, 0x67, 0x66, 0x68, 0x6A, 0x6B, 0x6D, 0x6F, 0x6E, 0x71, 0x73, 0x72, 0x74,
    0x78, 0x77, 0x79, 0x7B, 0x7B, 0x7D, 0x7F, 0x7E, 0x82, 0x84, 0x83, 0x85, 0x88, 0x87, 0x89, 0x8B,
    0x8C, 0x8E, 0x90, 0x8F, 0x92, 0x94, 0x93, 0x95, 0x99, 0x98, 0x9A, 0x9C, 0x9C, 0x9E, 0xA0, 0x9F,
    0xA3, 0xA5, 0xA4, 0xA6, 0xA9, 0xA8, 0xAA, 0xAC, 0xAD, 0xAF, 0xB1, 0xB0, 0xB3, 0xB5, 0xB4, 0xB6,
    0xBA, 0xB9, 0xBB, 0xBD, 0xBD, 0xBF, 0xC1, 0xC0, 0xC4, 0xC6, 0xC5, 0xC7, 0xCA, 0xC9, 0xCB, 0xCD,
    0xCE, 0xD0, 0xD2, 0xD1, 0xD4, 0xD6, 0xD5, 0xD7, 0xDB, 0xDA, 0xDC, 0xDE, 0xDE, 0xE0, 0xE2, 0xE1,
    0xE5, 0xE7, 0xE6, 0xE8, 0xEB, 0xEA, 0xEC, 0xEE, 0xEF, 0xF1, 0xF3, 0xF2, 0xF5, 0xF7, 0xF6, 0xF8,
    0xFC, 0xFB, 0xFD, 0xFF, 0xFF, 0x01, 0x03, 0x02, 0x06, 0x08, 0x07, 0x09, 0x0C, 0x0B, 0x0D, 0x0F,
    0x10, 0x12, 0x14, 0x13, 0x16, 0x18, 0x17, 0x19, 0x1D, 0x1C, 0x1E, 0x20, 0x20, 0x22, 0x24, 0x23,
    0x27, 0x29, 0x28, 0x2A, 0x2D, 0x2C, 0x2E, 0x30, 0x31, 0x33, 0x35, 0x34, 0x37, 0x39, 0x38, 0x3A,
    0x3E, 0x3D, 0x3F, 0x41, 0x41, 0x43, 0x45, 0x44, 0x48, 0x4A, 0x49, 0x4B, 0x4E, 0x4D, 0x4F, 0x51,
    0x52, 0x54, 0x56, 0x55, 0x58, 0x5A, 0x59, 0x5B, 0x5F, 0x5E, 0x60, 0x62, 0x62, 0x64, 0x66, 0x65,
    0x69, 0x6B, 0x6A, 0x6C, 0x6F, 0x6E, 0x70, 0x72, 0x73, 0x75, 0x77, 0x76, 0x79, 0x7B, 0x7A, 0x7C,
    0x80, 0x7F, 0x81, 0x83, 0x83, 0x85, 0x87, 0x86, 0x8A, 0x8C, 0x8B, 0x8D, 0x90, 0x8F, 0x91, 0x93,
    0x94, 0x96, 0x98, 0x97, 0x9A, 0x9C, 0x9B, 0x9D, 0xA1, 0xA0, 0xA2, 0xA4, 0xA4, 0xA6, 0xA8, 0xA7,
    0xAB, 0xAD, 0xAC, 0xAE, 0xB1, 0xB0, 0xB2, 0xB4, 0xB5, 0xB7, 0xB9, 0xB8, 0xBB, 0xBD, 0xBC, 0xBE,
];

static SOME_FORMATTED_BYTES: &str = r#"b"\x00\x02\x04\x03\x06\x08\x07\x09\x0D\x0C\x0E\x10\x10\x12\x14\x13\
  \x17\x19\x18\x1A\x1D\x1C\x1E\x20!#%$')(*.-/113548:9;>=?ABDFEH\
  JIKONPRRTVUY[Z\x5C_^`bcegfikjlpoqssuwvz|{}\x80\x7F\x81\x83\x84\
  \x86\x88\x87\x8A\x8C\x8B\x8D\x91\x90\x92\x94\x94\x96\x98\x97\x9B\
  \x9D\x9C\x9E\xA1\xA0\xA2\xA4\xA5\xA7\xA9\xA8\xAB\xAD\xAC\xAE\xB2\
  \xB1\xB3\xB5\xB5\xB7\xB9\xB8\xBC\xBE\xBD\xBF\xC2\xC1\xC3\xC5\xC6\
  \xC8\xCA\xC9\xCC\xCE\xCD\xCF\xD3\xD2\xD4\xD6\xD6\xD8\xDA\xD9\xDD\
  \xDF\xDE\xE0\xE3\xE2\xE4\xE6\xE7\xE9\xEB\xEA\xED\xEF\xEE\xF0\xF4\
  \xF3\xF5\xF7\xF7\xF9\xFB\xFA\xFE\x00\xFF\x01\x04\x03\x05\x07\x08\
  \x0A\x0C\x0B\x0E\x10\x0F\x11\x15\x14\x16\x18\x18\x1A\x1C\x1B\x1F\
  ! \x22%$&()+-,/10265799;=<@BACFEGIJLNMPRQSWVXZZ\x5C^]acbdgfhj\
  kmonqsrtxwy{{}\x7F~\x82\x84\x83\x85\x88\x87\x89\x8B\x8C\x8E\x90\
  \x8F\x92\x94\x93\x95\x99\x98\x9A\x9C\x9C\x9E\xA0\x9F\xA3\xA5\xA4\
  \xA6\xA9\xA8\xAA\xAC\xAD\xAF\xB1\xB0\xB3\xB5\xB4\xB6\xBA\xB9\xBB\
  \xBD\xBD\xBF\xC1\xC0\xC4\xC6\xC5\xC7\xCA\xC9\xCB\xCD\xCE\xD0\xD2\
  \xD1\xD4\xD6\xD5\xD7\xDB\xDA\xDC\xDE\xDE\xE0\xE2\xE1\xE5\xE7\xE6\
  \xE8\xEB\xEA\xEC\xEE\xEF\xF1\xF3\xF2\xF5\xF7\xF6\xF8\xFC\xFB\xFD\
  \xFF\xFF\x01\x03\x02\x06\x08\x07\x09\x0C\x0B\x0D\x0F\x10\x12\x14\
  \x13\x16\x18\x17\x19\x1D\x1C\x1E\x20 \x22$#')(*-,.01354798:>=\
  ?AACEDHJIKNMOQRTVUXZY[_^`bbdfeikjlonprsuwvy{z|\x80\x7F\x81\x83\
  \x83\x85\x87\x86\x8A\x8C\x8B\x8D\x90\x8F\x91\x93\x94\x96\x98\x97\
  \x9A\x9C\x9B\x9D\xA1\xA0\xA2\xA4\xA4\xA6\xA8\xA7\xAB\xAD\xAC\xAE\
  \xB1\xB0\xB2\xB4\xB5\xB7\xB9\xB8\xBB\xBD\xBC\xBE""#;

#[test]
fn byte_string_display() {
    assert_eq!(
        SOME_FORMATTED_BYTES,
        AnyLiteral::from(SOME_BYTES).to_string()
    );
}

static SOME_JSON_STRING: &str = r#"{
    "name": "GitHub",
    "short_name": "GitHub",
    "start_url": "/",
    "display": "standalone",
    "icons": [
        {
            "sizes": "114x114",
            "src": "https://github.githubassets.com/apple-touch-icon-114x114.png"
        },
    ]
}
"#;

static SOME_FORMATTED_JSON_STRING: &str = r##"r#"{
    "name": "GitHub",
    "short_name": "GitHub",
    "start_url": "/",
    "display": "standalone",
    "icons": [
        {
            "sizes": "114x114",
            "src": "https://github.githubassets.com/apple-touch-icon-114x114.png"
        },
    ]
}
"#"##;

static SOME_MESSY_STRING: &str =
"abcbdedehghiijkjmnmnpopqrstsuvuvyxyzz{|{~\u{7f}~\u{7f}\u{81}\u{80}\u{81}\u{82}\u{83}\u{84}\u{85}\u{84}\u{86}\u{87}\u{86}\u{87}\u{8a}\u{89}\u{8a}\u{8b}\u{8b}\u{8c}\u{8d}\u{8c}\u{8f}\u{90}\u{8f}\u{90}\u{92}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{95}\u{97}\u{98}\u{97}\u{98}\u{9b}\u{9a}\u{9b}\u{9c}\u{9c}\u{9d}\u{9e}\u{9d}\u{a0}¡\u{a0}¡£¢£¤¥¦§¦¨©¨©¬«¬\u{ad}\u{ad}®¯®±²±²´³´µ¶·¸·¹º¹º½¼½¾¾¿À¿ÂÃÂÃÅÄÅÆÇÈÉÈÊËÊËÎÍÎÏÏÐÑÐÓÔÓÔÖÕÖ×ØÙÚÙÛÜÛÜßÞßààáâáäåäåçæçèéêëêìíìíðïðññòóòõöõöø÷øùúûüûýþýþāĀāĂĂăĄăĆćĆćĉĈĉĊċČčČĎďĎďĒđĒēēĔĕĔėĘėĘĚęĚěĜĝĞĝğĠğĠģĢģĤĤĥĦĥĨĩĨĩīĪīĬĭĮįĮİıİıĴĳĴĵĵĶķĶĹĺĹĺļĻļĽľĿŀĿŁłŁłŅńŅņņŇňŇŊŋŊŋōŌōŎŏŐőŐŒœŒœŖŕŖŗŗŘřŘśŜśŜŞŝŞşŠšŢšţŤţŤŧŦŧŨŨũŪũŬŭŬŭůŮůŰűŲųŲŴŵŴŵŸŷŸŹŹźŻźŽžŽžƀſƀƁƂƃƄƃƅƆƅƆƉƈƉƊƊƋƌƋƎƏƎƏƑƐƑƒƓƔƕƔƖƗƖƗƚƙƚƛƛƜƝƜƟƠƟƠƢơƢƣƤƥƦƥƧƨƧƨƫƪƫƬƬƭƮƭưƱưƱƳƲƳƴƵƶƷƶƸƹƸƹƼƻƼƽƽƾƿƾǁǂǁǂǄǃǄǅǆǇǈǇǉǊǉǊ";

static SOME_FORMATTED_MESSY_STRING: &str = "r\"abcbdedehghiijkjmnmnpopqrstsuvuvyxyzz{|{~\u{7f}~\u{7f}\u{81}\u{80}\u{81}\u{82}\u{83}\u{84}\u{85}\u{84}\u{86}\u{87}\u{86}\u{87}\u{8a}\u{89}\u{8a}\u{8b}\u{8b}\u{8c}\u{8d}\u{8c}\u{8f}\u{90}\u{8f}\u{90}\u{92}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{95}\u{97}\u{98}\u{97}\u{98}\u{9b}\u{9a}\u{9b}\u{9c}\u{9c}\u{9d}\u{9e}\u{9d}\u{a0}¡\u{a0}¡£¢£¤¥¦§¦¨©¨©¬«¬\u{ad}\u{ad}®¯®±²±²´³´µ¶·¸·¹º¹º½¼½¾¾¿À¿ÂÃÂÃÅÄÅÆÇÈÉÈÊËÊËÎÍÎÏÏÐÑÐÓÔÓÔÖÕÖ×ØÙÚÙÛÜÛÜßÞßààáâáäåäåçæçèéêëêìíìíðïðññòóòõöõöø÷øùúûüûýþýþāĀāĂĂăĄăĆćĆćĉĈĉĊċČčČĎďĎďĒđĒēēĔĕĔėĘėĘĚęĚěĜĝĞĝğĠğĠģĢģĤĤĥĦĥĨĩĨĩīĪīĬĭĮįĮİıİıĴĳĴĵĵĶķĶĹĺĹĺļĻļĽľĿŀĿŁłŁłŅńŅņņŇňŇŊŋŊŋōŌōŎŏŐőŐŒœŒœŖŕŖŗŗŘřŘśŜśŜŞŝŞşŠšŢšţŤţŤŧŦŧŨŨũŪũŬŭŬŭůŮůŰűŲųŲŴŵŴŵŸŷŸŹŹźŻźŽžŽžƀſƀƁƂƃƄƃƅƆƅƆƉƈƉƊƊƋƌƋƎƏƎƏƑƐƑƒƓƔƕƔƖƗƖƗƚƙƚƛƛƜƝƜƟƠƟƠƢơƢƣƤƥƦƥƧƨƧƨƫƪƫƬƬƭƮƭưƱưƱƳƲƳƴƵƶƷƶƸƹƸƹƼƻƼƽƽƾƿƾǁǂǁǂǄǃǄǅǆǇǈǇǉǊǉǊ\"";

#[test]
fn string_display() {
    assert_eq!(
        SOME_FORMATTED_JSON_STRING,
        AnyLiteral::from(SOME_JSON_STRING).to_string()
    );

    assert_eq!(
        SOME_FORMATTED_MESSY_STRING,
        AnyLiteral::from(SOME_MESSY_STRING).to_string()
    );
}