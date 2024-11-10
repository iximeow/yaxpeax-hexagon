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

#[track_caller]
fn test_invalid(bytes: &[u8], expected: DecodeError) {
    let decoder = yaxpeax_hexagon::InstDecoder::default();
    let err = decoder.decode(&mut yaxpeax_arch::U8Reader::new(bytes)).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn supervisor() {
    test_display(&0b0110_0111000_00010_11_0011010_0000110u32.to_le_bytes(), "{ S6 = R2 }");
    test_display(&0b0110_1101000_00010_11_0011010_0000110u32.to_le_bytes(), "{ S7:6 = R3:2 }");

    test_display(&0b0110_11101_0100010_11_000000000_00110u32.to_le_bytes(), "{ R6 = S34 }");
    test_display(&0b0110_11110_0100010_11_000000000_00110u32.to_le_bytes(), "{ R7:6 = S35:34 }");

    test_display(&0b0110_1100000_00010_11_0_01101_00000000u32.to_le_bytes(), "{ tlbw(R3:2, R13) }");
    test_invalid(&0b0110_1100000_00010_11_1_01101_00000000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0110_1100010_00010_11_0_00000000_01000u32.to_le_bytes(), "{ R9:8 = tlbr(R2) }");
    test_display(&0b0110_1100100_00010_11_0_00000000_01000u32.to_le_bytes(), "{ R8 = tlbp(R2) }");

    test_display(&0b0110_1100101_00010_11_0_00000000_01000u32.to_le_bytes(), "{ tlbinvasid(R2) }");
    test_display(&0b0110_1100110_00010_11_0_01001000_01000u32.to_le_bytes(), "{ R8 = ctlbw(R3:2, R9) }");
    test_invalid(&0b0110_1100110_00010_11_1_00000000_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0110_1100111_00010_11_0_00000000_01000u32.to_le_bytes(), "{ R8 = tlboc(R3:2) }");
}

#[test]
fn general() {
    test_display(&0b0001_0110000_01001_11_00_0111_000_00100u32.to_le_bytes(), "{ R17 = #7; jump $+#8 }");
    test_display(&0b0001_0111000_01001_11_00_0111_000_00100u32.to_le_bytes(), "{ R7 = R17; jump $+#8 }");

    test_display(&0b0001_0011011_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ P1 = cmp.gtu(R17, #3); if (!P1.new) jump:nt $+#-508 }");
    test_display(&0b0001_0011101_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ P1 = tstbit(R17, #0x0); if (P1.new) jump:nt $+#-508 }");
    test_display(&0b0001_0011111_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ P1 = tstbit(R17, #0x0); if (!P1.new) jump:nt $+#-508 }");
    test_display(&0b0001_0011111_11001_11_10_0011_000_00010u32.to_le_bytes(), "{ P1 = tstbit(R17, #0x0); if (!P1.new) jump:t $+#-508 }");

    test_display(&0b0001_0101011_11001_11_10_0111_000_00010u32.to_le_bytes(), "{ P0 = cmp.gtu(R17, R7); if (!P0.new) jump:t $+#-508 }");
    test_display(&0b0001_0101011_11001_11_11_0111_000_00010u32.to_le_bytes(), "{ P1 = cmp.gtu(R17, R7); if (!P1.new) jump:t $+#-508 }");

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

    test_display(&0b0011_0010110_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (P1.new) R17:16 = memd(R4 + R2<<3) }");
    test_display(&0b0011_0010110_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (P1.new) R17:16 = memd(R4 + R2<<3) }");

    test_display(&0b0011_0101011_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (!P1) memh(R4 + R2<<3) = R16.H }");
    test_display(&0b0011_0101101_00100_11_1_0_0010_101_10010u32.to_le_bytes(), "{ if (!P1) memw(R4 + R2<<3) = R2.new }");

    test_display(&0b0011_0101101_00100_11_1_0_0010_101_10010u32.to_le_bytes(), "{ if (!P1) memw(R4 + R2<<3) = R2.new }");

    test_invalid(&0b0011_0101111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0110001_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0110111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0111001_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0111111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1000000_00100_11_1_0_0010_101_11111u32.to_le_bytes(), "{ if (P1) memb(R4+#5) = #-1 }");
    test_display(&0b0011_1000100_00100_11_1_0_0010_101_11111u32.to_le_bytes(), "{ if (!P1) memb(R4+#5) = #-1 }");
    test_invalid(&0b0011_1000111_00100_11_1_0_0010_101_11111u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1010000_00100_11_1_0_0010_100_11111u32.to_le_bytes(), "{ R31 = memb(R4 + R2<<3) }");
    test_display(&0b0011_1010001_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ R31:30 = memub(R4 + R2<<3) }");

    test_display(&0b0011_1011010_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ memh(R4 + R2<<3) = R30 }");
    test_display(&0b0011_1011011_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ memh(R4 + R2<<3) = R30.H }");
    test_display(&0b0011_1011101_00100_11_1_0_0010_100_10110u32.to_le_bytes(), "{ memw(R4 + R2<<3) = R6.new }");

    test_display(&0b0010_00000001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00000001_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(R4.new, R2)) jump:t $+#812 }");
    test_display(&0b0010_00000101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.eq(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00001001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gt(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00001101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gt(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00010001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gtu(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00010101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(R4.new, R2)) jump:nt $+#812 }");
    test_display(&0b0010_00011001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gt(R2, R4.new)) jump:nt $+#812 }");
    test_display(&0b0010_00100101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(R2, R4.new)) jump:nt $+#812 }");
    test_invalid(&0b0010_00101101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0010_00110101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0010_00111101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0010_01000001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(R4.new, #2)) jump:nt $+#812 }");
    test_display(&0b0010_01010101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(R4.new, #2)) jump:t $+#812 }");
    test_display(&0b0010_01011001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (tstbit(R4.new, #0)) jump:nt $+#812 }");
    test_display(&0b0010_01011101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!tstbit(R4.new, #0)) jump:t $+#812 }");
    test_display(&0b0010_01101101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gt(R4.new, #-1)) jump:t $+#812 }");

    test_invalid(&0b0101_000_0100_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_000_0101_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ callr R1 }");
    test_display(&0b0101_000_0110_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ callrh R1 }");
    test_invalid(&0b0101_000_0111_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_000_1000_00001_11_0_00010_000_00000u32.to_le_bytes(), "{ if (P2) callr R1 }");
    test_display(&0b0101_000_1001_00001_11_0_00010_000_00000u32.to_le_bytes(), "{ if (!P2) callr R1 }");
    test_invalid(&0b0101_000_1010_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b0101_001_0011_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_001_0100_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ jumpr R1 }");
    test_display(&0b0101_001_0101_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ hintjr(R1) }");
    test_display(&0b0101_001_0110_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ jumprh R1 }");
    test_invalid(&0b0101_001_0111_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0101_001_1000_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0101_001_1001_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_001_1010_00001_11_0_01011_000_00000u32.to_le_bytes(), "{ if (P3.new) jumpr:nt R1 }");
    test_display(&0b0101_001_1011_00001_11_0_01011_000_00000u32.to_le_bytes(), "{ if (!P3.new) jumpr:nt R1 }");
    test_invalid(&0b0101_001_1100_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0101_010_0000_00000_11_0_01000_000_00010u32.to_le_bytes(), "{ trap0(#0x40) }");
    test_display(&0b0101_010_0010_00010_11_0_01000_000_00010u32.to_le_bytes(), "{ pause(#0x240) }");
    test_display(&0b0101_010_0100_00010_11_0_01000_000_00010u32.to_le_bytes(), "{ trap1(R2, #0x40) }");
    test_invalid(&0b0101_010_0110_00010_11_0_01000_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0101_011_0101_00010_11_0_01000_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_011_0110_00010_11_0_00000_000_00000u32.to_le_bytes(), "{ icinva(R2) }");
    test_display(&0b0101_011_1110_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ isync }");
    test_display(&0b0101_011_1111_00000_11_0_10000_000_00010u32.to_le_bytes(), "{ unpause }");

    test_display(&0b0101_100_0000_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ jump $+#4 }");
    test_display(&0b0101_101_0000_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ call $+#4 }");

    test_display(&0b0101_110_0000_00000_11_0_00001_000_00010u32.to_le_bytes(), "{ if (P1) jump:nt $+#4 }");
    test_display(&0b0101_110_0001_00000_11_0_01001_000_00010u32.to_le_bytes(), "{ if (!P1.new) jump:nt $+#4 }");
    test_display(&0b0101_110_1001_00000_11_0_00001_000_00010u32.to_le_bytes(), "{ if (!P1) call $+#4 }");
    test_invalid(&0b0101_110_1001_00000_11_0_01001_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0110_0010001_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ C22 = R6 }");
    test_display(&0b0110_0011001_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ C23:22 = R7:6 }");
    test_display(&0b0110_1000000_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ R23:22 = C7:6 }");
    test_display(&0b0110_1010000_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ R22 = C6 }");
    test_display(&0b0110_1010010_01001_11_0_000101_00_10110u32.to_le_bytes(), "{ R22 = add(pc, #0x5) }");

    test_display(&0b1000_0000000_00100_11_000110_000_10110u32.to_le_bytes(), "{ R23:22 = asr(R5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_001_10110u32.to_le_bytes(), "{ R23:22 = lsr(R5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_010_10110u32.to_le_bytes(), "{ R23:22 = asl(R5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_011_10110u32.to_le_bytes(), "{ R23:22 = rol(R5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000000_100_10110u32.to_le_bytes(), "{ R23:22 = vsathub(R5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_101_10110u32.to_le_bytes(), "{ R23:22 = vsatwuh(R5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_110_10110u32.to_le_bytes(), "{ R23:22 = vsatwh(R5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_111_10110u32.to_le_bytes(), "{ R23:22 = vsathb(R5:4) }");
    test_display(&0b1000_0000001_00100_11_000110_000_10110u32.to_le_bytes(), "{ R23:22 = vasrh(R5:4, #0x6):raw }");
    test_invalid(&0b1000_0000001_00100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_010110_000_10110u32.to_le_bytes(), DecodeError::InvalidOperand);

    test_display(&0b1000_0000010_00100_11_000110_000_10110u32.to_le_bytes(), "{ R23:22 = vasrw(R5:4, #0x6) }");
    test_display(&0b1000_0000010_00100_11_000110_001_10110u32.to_le_bytes(), "{ R23:22 = vlsrw(R5:4, #0x6) }");
    test_display(&0b1000_0000010_00100_11_000110_010_10110u32.to_le_bytes(), "{ R23:22 = vaslw(R5:4, #0x6) }");
    test_invalid(&0b1000_0000010_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000010_00100_11_000110_100_10110u32.to_le_bytes(), "{ R23:22 = vabsh(R5:4) }");
    test_display(&0b1000_0000010_00100_11_000110_101_10110u32.to_le_bytes(), "{ R23:22 = vabsh(R5:4):sat }");
    test_display(&0b1000_0000010_00100_11_000110_110_10110u32.to_le_bytes(), "{ R23:22 = vabsw(R5:4) }");
    test_display(&0b1000_0000010_00100_11_000110_111_10110u32.to_le_bytes(), "{ R23:22 = vabsw(R5:4):sat }");

    test_invalid(&0b1000_0000011_00100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_0000100_00100_11_000110_000_10110u32.to_le_bytes(), "{ R23:22 = vasrh(R5:4, #0x6) }");
    test_display(&0b1000_0000100_00100_11_000110_001_10110u32.to_le_bytes(), "{ R23:22 = vlsrh(R5:4, #0x6) }");
    test_display(&0b1000_0000100_00100_11_000110_010_10110u32.to_le_bytes(), "{ R23:22 = vaslh(R5:4, #0x6) }");
    test_invalid(&0b1000_0000100_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000100_00100_11_000110_100_10110u32.to_le_bytes(), "{ R23:22 = not(R5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_101_10110u32.to_le_bytes(), "{ R23:22 = neg(R5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_110_10110u32.to_le_bytes(), "{ R23:22 = abs(R5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_111_10110u32.to_le_bytes(), "{ R23:22 = vconj(R5:4):sat }");

    test_invalid(&0b1000_0000101_00100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b1000_0000101_00100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_0000110_00100_11_000110_100_10110u32.to_le_bytes(), "{ R23:22 = deinterleave(R5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_101_10110u32.to_le_bytes(), "{ R23:22 = interleave(R5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_110_10110u32.to_le_bytes(), "{ R23:22 = brev(R5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_111_10110u32.to_le_bytes(), "{ R23:22 = asr(R5:4, #0x6):rnd }");

    test_display(&0b1000_0000111_00100_11_000110_000_10110u32.to_le_bytes(), "{ R23:22 = convert_df2d(R5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_001_10110u32.to_le_bytes(), "{ R23:22 = convert_df2ud(R5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_010_10110u32.to_le_bytes(), "{ R23:22 = convert_ud2df(R5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_011_10110u32.to_le_bytes(), "{ R23:22 = convert_d2df(R5:4) }");

    test_invalid(&0b1000_0000111_00100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000111_00100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000111_00100_11_000110_110_10110u32.to_le_bytes(), "{ R23:22 = convert_df2d(R5:4):chop }");
    test_display(&0b1000_0000111_00100_11_000110_111_10110u32.to_le_bytes(), "{ R23:22 = convert_df2ud(R5:4):chop }");

    test_display(&0b1000_0001101_00100_11_000110_111_10110u32.to_le_bytes(), "{ R23:22 = extractu(R5:4, #0x6, #0x2f) }");
    test_display(&0b1000_1111011_00100_11_000110_111_10110u32.to_le_bytes(), "{ R22 = insert(R4, #0x6, #0x1f) }");

    test_display(&0b1001_0000000_00010_11_0_00000_000_11110u32.to_le_bytes(), "{ R31:30 = deallocframe(R2):raw }");
    test_display(&0b1001_0010000_00010_11_000_000_000_00011u32.to_le_bytes(), "{ R3 = memw_locked(R2) }");
    test_display(&0b1001_0010000_00010_11_001_000_000_00011u32.to_le_bytes(), "{ R3 = memw_aq(R2) }");
    test_display(&0b1001_0010000_00010_11_010_000_000_00100u32.to_le_bytes(), "{ R5:4 = memd_locked(R2) }");
    test_display(&0b1001_0010000_00010_11_011_000_000_00100u32.to_le_bytes(), "{ R5:4 = memd_aq(R2) }");
    test_invalid(&0b1001_0010000_00010_11_000_000_001_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_001_000_001_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_010_000_001_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_011_000_001_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_000_000_010_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_001_000_010_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_010_000_010_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_011_000_010_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_000_000_011_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_001_000_011_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_010_000_011_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1001_0010000_00010_11_011_000_011_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0100000_00010_11_000_111_111_11111u32.to_le_bytes(), "{ dcfetch(R2+#16376) }");
    test_display(&0b1001_0110000_00010_11_0000_00_000_10000u32.to_le_bytes(), "{ R17:16 = dealloc_return(R2):raw }");
    test_invalid(&0b1001_0110000_00010_11_0001_00_000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110000_00010_11_0010_01_000_10000u32.to_le_bytes(), "{ if (P1.new) R17:16 = dealloc_return(R2):nt:raw }");
    test_display(&0b1001_0110000_00010_11_0100_01_000_10000u32.to_le_bytes(), "{ if (P1) R17:16 = dealloc_return(R2):raw }");
    test_display(&0b1001_0110000_00010_11_0110_01_000_10000u32.to_le_bytes(), "{ if (P1.new) R17:16 = dealloc_return(R2):t:raw }");
    test_invalid(&0b1001_0110000_00010_11_1000_01_000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110000_00010_11_1010_01_000_10000u32.to_le_bytes(), "{ if (!P1.new) R17:16 = dealloc_return(R2):nt:raw }");
    test_display(&0b1001_0110000_00010_11_1100_01_000_10000u32.to_le_bytes(), "{ if (!P1) R17:16 = dealloc_return(R2):raw }");
    test_display(&0b1001_0110000_00010_11_1110_01_000_10000u32.to_le_bytes(), "{ if (!P1.new) R17:16 = dealloc_return(R2):t:raw }");
    test_display(&0b1001_0110001_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R16 = membh(R2+#3648) }");
    test_display(&0b1001_0110010_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memh_fifo(R2+#3648) }");
    test_display(&0b1001_0110011_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R16 = memubh(R2+#3648) }");
    test_display(&0b1001_0110100_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memb_fifo(R2+#1824) }");
    test_display(&0b1001_0110101_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memubh(R2+#7296) }");
    test_invalid(&0b1001_0000110_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110111_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = membh(R2+#7296) }");
    test_display(&0b1001_0111000_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R16 = memb(R2+#14592) }");
    test_display(&0b1001_0111001_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memub(R2+#14592) }");
    test_display(&0b1001_0111010_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R16 = memh(R2+#14592) }");
    test_display(&0b1001_0111011_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memuh(R2+#14592) }");
    test_display(&0b1001_0111100_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R16 = memw(R2+#14592) }");
    test_invalid(&0b1001_0001101_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0111110_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ R17:16 = memd(R2+#14592) }");
    test_invalid(&0b1001_0001111_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_0000101_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ memw_locked(R2, P3)=R4 }");
    test_invalid(&0b1010_0000101_00010_11_0_00100_000_00111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0000111_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ memd_locked(R2, P3)=R5:R4 }");
    test_invalid(&0b1010_0000111_00010_11_0_00100_000_00111u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1011_1000001_00100_11_1_0_0000_001_10110u32.to_le_bytes(), "{ R22 = add(R4, #-31999) }");

    test_display(&0b1111_1001000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R6 = and(R4, R3) }");
    test_display(&0b1111_1001001_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R6 = or(R4, R3) }");
    test_invalid(&0b1111_1001010_00100_11_1_00011_001_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1111_1001011_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R6 = xor(R4, R3) }");

    test_display(&0b1111_1011000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R6 = add(R4, R3) }");
    test_display(&0b1111_1011001_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R6 = sub(R4, R3) }");
    test_invalid(&0b1111_1011101_00100_11_1_00011_001_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1111_1101000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (P1.new) R7:6 = contains(R4, R3) }");

    // TODO: testcase for Rn=add(pc,#nn)
}
