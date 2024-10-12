#![no_main]

use libfuzzer_sys::fuzz_target;
use shakmaty::fen::Fen;
use shakmaty::variant::Variant;
use lila_cloudeval::binary_fen::VariantSetup;

fuzz_target!(|data: &[u8]| {
    if let Ok(fen) = Fen::from_ascii(data) {
        let variant = Variant::Chess;
        let original = VariantSetup::new_normalized(fen.into_setup(), variant);

        let mut buf = Vec::new();
        original.write(&mut buf);

        let roundtripped = VariantSetup::read(&mut &buf[..]);
        assert_eq!(original, roundtripped);
    }
});
