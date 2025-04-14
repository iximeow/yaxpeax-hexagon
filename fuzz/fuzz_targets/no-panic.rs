#![no_main]
use libfuzzer_sys::fuzz_target;

use yaxpeax_arch::Decoder;

use std::fmt::Write;

fuzz_target!(|data: &[u8]| {
    let decoder = yaxpeax_hexagon::InstDecoder::default();

    let mut hexagon_inst = yaxpeax_hexagon::InstructionPacket::default();

    let mut words = yaxpeax_arch::U8Reader::new(data);

    // whatever the output, we should be able to format the instruction that was decoded into
    let _ = decoder.decode_into(&mut hexagon_inst, &mut words);
    write!(&mut String::new(), "{}", hexagon_inst).expect("formatting does not panic either");
});
