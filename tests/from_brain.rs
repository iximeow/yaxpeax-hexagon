use yaxpeax_arch::Decoder;
use yaxpeax_arch::StandardDecodeError as DecodeError;

// these instructions are just what i think the manual says

#[track_caller]
fn test_display(bytes: &[u8], text: &str) {
    let decoder = yaxpeax_hexagon::InstDecoder::default();
    let inst = decoder.decode(&mut yaxpeax_arch::U8Reader::new(bytes)).expect("decode succeeds");
    let rendered = format!("{}", inst);
    assert_eq!(rendered, text);
}

fn test_invalid(bytes: &[u8], expected: DecodeError) {
    let decoder = yaxpeax_hexagon::InstDecoder::default();
    let err = decoder.decode(&mut yaxpeax_arch::U8Reader::new(bytes)).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn alu32() {
    test_invalid(&0b0111_0000010_00000_11_0_0_0000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_0000000_00011_11_1_0_1101_000_00100u32.to_le_bytes(), "{ if (!P1.new) R4 = aslh(R3) }");
    test_display(&0b0111_0000011_00001_11_0_0_0000_000_00100u32.to_le_bytes(), "{ R4 = R1 }");

    test_display(&0b0111_0001011_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ R2.L = #0x4020 }");
    test_display(&0b0111_0001101_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ R2.L = #0x8020 }");
    test_display(&0b0111_0010101_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ R2.H = #0x8020 }");

    test_display(&0b0111_0011001_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ R16 = mux(P1, R6, #7) }");
    test_display(&0b0111_0011101_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ R16 = mux(P1, #7, R6) }");

    test_display(&0b0111_0011000_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R17:16 = combine(R6, #7) }");
    test_display(&0b0111_0011001_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R17:16 = combine(#7, R6) }");
    test_display(&0b0111_0011010_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R16 = cmp.eq(R6, #7) }");
    test_display(&0b0111_0011011_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R16 = !cmp.eq(R6, #7) }");

    test_display(&0b0111_0100010_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ if (P2) R16 = add(R6, #7) }");
    test_display(&0b0111_0100010_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (P2.new) R16 = add(R6, #7) }");
    test_display(&0b0111_0100110_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (!P2.new) R16 = add(R6, #7) }");

    test_display(&0b0111_0101001_00110_11_1_0_0000_111_00001u32.to_le_bytes(), "{ P1 = cmp.eq(R6, #-249) }");
    test_display(&0b0111_0101001_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ P1 = !cmp.eq(R6, #-249) }");
    test_display(&0b0111_0101011_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ P1 = !cmp.gt(R6, #-249) }");
    test_display(&0b0111_0101100_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ P1 = !cmp.gtu(R6, #0x107) }");
    test_invalid(&0b0111_0101101_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b0111_0101110_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_0110001_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ R17 = and(R6, #-249) }");
    test_display(&0b0111_0110011_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ R17 = sub(#-249, R6) }");
    test_display(&0b0111_0110101_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ R17 = or(R6, #-249) }");
    test_invalid(&0b0111_0110111_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b0111_0111111_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_1000110_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ R17 = #-15929 }");

    test_display(&0b0111_1100010_00000_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R17:16 = combine(#7, #-127) }");
    test_display(&0b0111_1100100_10000_11_1_0_0000_111_10000u32.to_le_bytes(), "{ R17:16 = combine(#7, #0x21) }");

    test_invalid(&0b0111_1101100_10000_11_1_0_0000_111_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_1110010_00100_11_0_0_0000_111_10000u32.to_le_bytes(), "{ if (P2) R16 = #1031 }");
    test_display(&0b0111_1110110_00100_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (!P2.new) R16 = #1031 }");
}
