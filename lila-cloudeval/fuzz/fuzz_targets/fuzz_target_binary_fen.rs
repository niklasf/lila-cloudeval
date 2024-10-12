#![no_main]

use libfuzzer_sys::fuzz_target;
use shakmaty::fen::Fen;

fuzz_target!(|data: &[u8]| {
    if let Ok(fen) Fen::parse_ascii(data) {
        let original = VariantSetup {
            setup: fen.into_setup(),
            variant: Variant::Chess,
        };

        let mut buf = Vec::new();
        original.write(&mut buf);

        let roundtripped = VariantSetup::read(&mut buf[..]);
        assert_eq!(original, roundtripped);
    }
});
