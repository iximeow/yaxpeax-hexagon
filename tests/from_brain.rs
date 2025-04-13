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

// mix of seen-in-the-wild extenders and stuff i made up
#[test]
fn extenders() {
    /*
     * extendable instructions covered in this test:
     *        // it turns out these are encoded by applying an extender to the `mem{b,ub,...}(gp+#u16)` forms
     * [ ]:   Rd = mem{b,ub,h,uh,w,d}(##U32)
     *        // predicated loads
     * [ ]:   if ([!]Pt[.new]) Rd = mem{b,ub,h,uh,w,d} (Rs + ##U32)
     * [ ]:   Rd = mem{b,ub,h,uh,w,d} (Rs + ##U32)
     * [ ]:   Rd = mem{b,ub,h,uh,w,d} (Re=##U32)
     * [ ]:   Rd = mem{b,ub,h,uh,w,d} (Rt<<#u2 + ##U32)
     * [ ]:   if ([!]Pt[.new]) Rd = mem{b,ub,h,uh,w,d} (##U32)
     *        // it turns out these are encoded by applying an extender to the `mem{b,ub,...}(gp+#u16)` forms
     * [ ]:   mem{b,h,w,d}(##U32) = Rs[.new]
     *        // predicated stores
     * [ ]:   if ([!]Pt[.new]) mem{b,h,w,d}(Rs + ##U32) = Rt[.new]
     * [ ]:   mem{b,h,w,d}(Rs + ##U32) = Rt[.new]
     * [ ]:   mem{b,h,w,d}(Rd=##U32) = Rt[.new]
     * [ ]:   mem{b,h,w,d}(Ru<<#u2 + ##U32) = Rt[.new]
     * [ ]:   if ([!]Pt[.new]) mem{b,h,w,d}(##U32) = Rt[.new]
     * [ ]:   [if [!]Ps] memw(Rs + #u6) = ##U32 // constant store
     * [ ]:   memw(Rs + Rt<<#u2) = ##U32 // constant store
     * [ ]:   if (cmp.xx(Rs.new,##U32)) jump:hint target
     * [ ]:   Rd = ##u32
     * [ ]:   Rdd = combine(Rs,##u32)
     * [ ]:   Rdd = combine(##u32,Rs)
     * [ ]:   Rdd = combine(##u32,#s8)
     * [ ]:   Rdd = combine(#s8,##u32)
     * [ ]:   Rd = mux(Pu, Rs,##u32)
     * [ ]:   Rd = mux(Pu, ##u32, Rs)
     * [ ]:   Rd = mux(Pu,##u32,#s8)
     * [ ]:   if ([!]Pu[.new]) Rd = add(Rs,##u32)
     * [ ]:   if ([!]Pu[.new]) Rd = ##u32
     * [ ]:   Pd = [!]cmp.eq (Rs,##u32)
     * [ ]:   Pd = [!]cmp.gt (Rs,##u32)
     * [ ]:   Pd = [!]cmp.gtu (Rs,##u32)
     * [ ]:   Rd = [!]cmp.eq(Rs,##u32)
     * [ ]:   Rd = and(Rs,##u32)
     * [ ]:   Rd = or(Rs,##u32)
     * [ ]:   Rd = sub(##u32,Rs)
     * [ ]:   Rd = add(Rs,##s32)
     * [ ]:   Rd = mpyi(Rs,##u32)
     * [ ]:   Rd += mpyi(Rs,##u32)
     * [ ]:   Rd -= mpyi(Rs,##u32)
     * [ ]:   Rx += add(Rs,##u32)
     * [ ]:   Rx -= add(Rs,##u32)
     * [x]:   Rd = ##u32
     * [ ]:   Rd = add(Rs,##s32)
     * [ ]:   jump (PC + ##s32)
     * [ ]:   call (PC + ##s32)
     * [ ]:   if ([!]Pu) call (PC + ##s32)
     * [ ]:   Pd = spNloop0(PC+##s32,Rs/#U10)
     * [ ]:   loop0/1 (PC+##s32,#Rs/#U10)
     * [ ]:   Rd = add(pc,##s32)
     * [ ]:   Rd = add(##u32,mpyi(Rs,#u6))
     * [ ]:   Rd = add(##u32,mpyi(Rs,Rt))
     * [ ]:   Rd = add(Rs,add(Rt,##u32))
     * [x]:   Rd = add(Rs,sub(##u32,Rt))
     * [x]:   Rd = sub(##u32,add(Rs,Rt))
     * [x]:   Rd = or(Rs,and(Rt,##u32))
     * [ ]:   Rx = add/sub/and/or (##u32,asl/asr/lsr(Rx,#U5))
     * [ ]:   Rx = add/sub/and/or (##u32,asl/asr/lsr(Rs,Rx))
     * [ ]:   Rx = add/sub/and/or (##u32,asl/asr/lsr(Rx,Rs))
     * [ ]:   Pd = cmpb/h.{eq,gt,gtu} (Rs,##u32)
     */

    // HELP! not sure how extenders combined with shifted constants should work.
    // for example: `Rdd=memd(gp+#u16:3)` can be extended. if u16 is `0...1111`, where the low
    // three bits are all set, the address used for gp-relative addressing would be `1111000`. as
    // an extended constant, would this also be `1111000` and illegal for extension as bits other
    // than the low six are set? or is this `000111` with an unaligned address but otherwise
    // extended and legal for execution?
    //
    // i am going to assume that the shifted immediate is used.
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0x00, 0x48,
    ], "{ memb(##0x20f) = r6 }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0x40, 0x48,
    ], "{ memh(##0x21e) = r6 }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0x80, 0x48,
    ], "{ memw(##0x23c) = r6 }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0xa0, 0x48,
    ], "{ memb(##0x20f) = r6.new }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xce, 0xa0, 0x48,
    ], "{ memh(##0x21e) = r6.new }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xd6, 0xa0, 0x48,
    ], "{ memw(##0x23c) = r6.new }");
    // the immediate of 1111 << 3 shifts a 1 out of the low 6 bits and invalidates the operand.
    test_invalid(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0xc0, 0x48,
    ], DecodeError::InvalidOperand);
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x07, 0xc6, 0xc0, 0x48,
    ], "{ memd(##0x238) = r7:6 }");

    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x00, 0x49,
    ], "{ r6 = memb(##0x20f) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x20, 0x49,
    ], "{ r6 = memub(##0x20f) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x40, 0x49,
    ], "{ r6 = memh(##0x21e) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x60, 0x49,
    ], "{ r6 = memuh(##0x21e) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x80, 0x49,
    ], "{ r6 = memw(##0x23c) }");
    // the immediate of 1111 << 3 shifts a 1 out of the low 6 bits and invalidates the operand.
    test_invalid(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0xc0, 0x49,
    ], DecodeError::InvalidOperand);
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc0, 0xc0, 0x49,
    ], "{ r7:6 = memd(##0x238) }");

    // HELP! it is somewhat unfortunate that extended offsets don't get the ## treatment like
    // immediates. having different operands for all immediate-extended forms seems like a kind of
    // bad idea though.
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x05, 0x91,
    ], "{ r6 = memb(r5+#527) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x25, 0x91,
    ], "{ r6 = memub(r5+#527) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x45, 0x91,
    ], "{ r6 = memh(r5+#542) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x65, 0x91,
    ], "{ r6 = memuh(r5+#542) }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x85, 0x91,
    ], "{ r6 = memw(r5+#572) }");
    // the immediate of 1111 << 3 shifts a 1 out of the low 6 bits and invalidates the operand.
    test_invalid(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0xc5, 0x91,
    ], DecodeError::InvalidOperand);
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc0, 0xc5, 0x91,
    ], "{ r7:6 = memd(r5+#568) }");


    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0x0f, 0xc6, 0x00, 0x48,
    ], "{ memb(##0x20f) = r6 }");
    test_display(&[
        0x08, 0x40, 0x00, 0x00,
        0xe6, 0xc1, 0x00, 0x49,
    ], "{ r6 = memb(##0x20f) }");

    test_display(&[
        0x0f, 0x40, 0x92, 0x6e, // r15 = syscfg
        0x04, 0x40, 0xe1, 0x0f, // extender
        0xf1, 0xc0, 0x00, 0x78, // r17 = #whatever
    ], "{ r15 = syscfg; r17 = ##0xfe100107 }");

    // the fact that something generated this packet is a little hilarious
    test_display(&[
        0x00, 0x40, 0x00, 0x00, // extender (0)
        0xf1, 0xc1, 0x91, 0x76, // r17 = or(r17, #whatever)
    ], "{ r17 = or(r17, ##0xf) }");

    test_display(&[
        0x04, 0x40, 0x00, 0x00, // extender (4 == 0x100)
        0xf1, 0xc1, 0x91, 0x76, // r17 = or(r17, #whatever)
    ], "{ r17 = or(r17, ##0x10f) }");

    test_display(&[
        0x04, 0x40, 0x00, 0x00, // extender (4 == 0x100)
        0xf1, 0xc1, 0x51, 0x76, // r17 = sub(#whatever, r17)
    ], "{ r17 = sub(##0x10f, r17) }");

    test_display(&[
        0x04, 0x40, 0x00, 0x00, // extender (4 == 0x100)
        0xf1, 0xc1, 0x11, 0x76, // r17 = and(r17, #whatever)
    ], "{ r17 = and(r17, ##0x10f) }");
}

// mentioned in the V62 manual, not later?
// not sure if these are still what they seem in later versions, but until demonstrated
// otherwise...
#[test]
fn supervisor() {
    test_invalid(&0b0101_010_1100_00010_11_0_01000_000_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_010_1101_00010_11_0_01000_000_00110u32.to_le_bytes(), "{ r6 = icdatar(r2) }");
    test_display(&0b0101_010_1110_00010_11_0_01000_000_00110u32.to_le_bytes(), "{ ictagw(r2, r8) }");
    test_display(&0b0101_010_1111_00010_11_0_01000_000_00110u32.to_le_bytes(), "{ r6 = ictagr(r2) }");
    test_display(&0b0101_011_0110_00010_11_0_10000_000_00110u32.to_le_bytes(), "{ ickill }");
    test_display(&0b0101_011_0111_00010_11_0_01000_000_00110u32.to_le_bytes(), "{ icinvidx(r2) }");

    test_display(&0b0110_01_00000_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ swi(r6) }");
    test_display(&0b0110_01_00000_00110_11_0_00010_001_10110u32.to_le_bytes(), "{ cswi(r6) }");
    test_display(&0b0110_01_00000_00110_11_0_00010_011_10110u32.to_le_bytes(), "{ ciad(r6) }");
    test_display(&0b0110_01_00010_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ wait(r6) }");
    test_display(&0b0110_01_00010_00110_11_0_00010_010_10110u32.to_le_bytes(), "{ resume(r6) }");
    test_display(&0b0110_01_00011_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ stop(r6) }");
    test_display(&0b0110_01_00011_00110_11_0_00010_001_10110u32.to_le_bytes(), "{ start(r6) }");
    test_display(&0b0110_01_00011_00110_11_0_00010_010_10110u32.to_le_bytes(), "{ nmi(r6) }");
    test_display(&0b0110_01_00100_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ setimask(p2, r6) }");
    test_display(&0b0110_01_00100_00110_11_0_00010_011_10110u32.to_le_bytes(), "{ siad(r6) }");
    test_display(&0b0110_01_01000_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ crswap(r6, sgp0) }");
    test_display(&0b0110_01_01001_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ crswap(r6, sgp1) }");
    test_display(&0b0110_01_10000_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ r22 = getimask(r6) }");
    test_display(&0b0110_11_00001_00110_11_0_00010_000_10110u32.to_le_bytes(), "{ brkpt }");
    test_display(&0b0110_11_00001_00110_11_0_00010_001_00000u32.to_le_bytes(), "{ tlblock }");
    test_display(&0b0110_11_00001_00110_11_0_00010_011_00000u32.to_le_bytes(), "{ k0lock }");
    test_display(&0b0110_11_01100_00110_11_0_00010_000_00000u32.to_le_bytes(), "{ crswap(r7:6, sgp1:0) }");
    test_invalid(&0b0110_11_01100_00110_11_0_00010_000_00001u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_display(&0b0110_11_10011_00110_11_0_00010_011_10110u32.to_le_bytes(), "{ r22 = iassignr(r6) }");

    // ok
    test_display(&0b0110_0111000_00010_11_0011010_0000110u32.to_le_bytes(), "{ ssr = r2 }");
    test_display(&0b0110_1101000_00010_11_0011010_0000110u32.to_le_bytes(), "{ s7:6 = r3:2 }");

    test_display(&0b0110_11101_0100010_11_000000000_00110u32.to_le_bytes(), "{ r6 = isdbcfg1 }");
    test_display(&0b0110_11110_0100010_11_000000000_00110u32.to_le_bytes(), "{ r7:6 = s35:34 }");

    test_display(&0b0110_1100000_00010_11_0_01101_00000000u32.to_le_bytes(), "{ tlbw(r3:2, r13) }");
    test_invalid(&0b0110_1100000_00010_11_1_01101_00000000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0110_1100010_00010_11_0_00000000_01000u32.to_le_bytes(), "{ r9:8 = tlbr(r2) }");
    test_display(&0b0110_1100100_00010_11_0_00000000_01000u32.to_le_bytes(), "{ r8 = tlbp(r2) }");

    test_display(&0b0110_1100101_00010_11_0_00000000_01000u32.to_le_bytes(), "{ tlbinvasid(r2) }");
    test_display(&0b0110_1100110_00010_11_0_01001000_01000u32.to_le_bytes(), "{ r8 = ctlbw(r3:2, r9) }");
    test_invalid(&0b0110_1100110_00010_11_1_00000000_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0110_1100111_00010_11_0_00000000_01000u32.to_le_bytes(), "{ r8 = tlboc(r3:2) }");
}

// tests grouped by prefix. not very principled but it's something.

#[test]
fn inst_0001() {
    test_display(&0b0001_0110000_01001_11_00_0111_000_00100u32.to_le_bytes(), "{ r17 = #7; jump $+0x8 }");
    test_display(&0b0001_0111000_01001_11_00_0111_000_00100u32.to_le_bytes(), "{ r7 = r17; jump $+0x8 }");

    test_display(&0b0001_0011011_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ p1 = cmp.gtu(r17, #3); if (!p1.new) jump:nt $-0x1fc }");
    test_display(&0b0001_0011101_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ p1 = tstbit(r17, #0x0); if (p1.new) jump:nt $-0x1fc }");
    test_display(&0b0001_0011111_11001_11_00_0011_000_00010u32.to_le_bytes(), "{ p1 = tstbit(r17, #0x0); if (!p1.new) jump:nt $-0x1fc }");
    test_display(&0b0001_0011111_11001_11_10_0011_000_00010u32.to_le_bytes(), "{ p1 = tstbit(r17, #0x0); if (!p1.new) jump:t $-0x1fc }");

    test_display(&0b0001_0101011_11001_11_10_0111_000_00010u32.to_le_bytes(), "{ p0 = cmp.gtu(r17, r7); if (!p0.new) jump:t $-0x1fc }");
    test_display(&0b0001_0101011_11001_11_11_0111_000_00010u32.to_le_bytes(), "{ p1 = cmp.gtu(r17, r7); if (!p1.new) jump:t $-0x1fc }");
}

#[test]
fn inst_0010() {
    test_display(&0b0010_00000001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00000001_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(r4.new, r2)) jump:t $+0x32c }");
    test_display(&0b0010_00000101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.eq(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00001001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gt(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00001101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gt(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00010001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gtu(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00010101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(r4.new, r2)) jump:nt $+0x32c }");
    test_display(&0b0010_00011001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.gt(r2, r4.new)) jump:nt $+0x32c }");
    test_display(&0b0010_00100101_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(r2, r4.new)) jump:nt $+0x32c }");
    test_invalid(&0b0010_00101101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0010_00110101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0010_00111101_0100_11_0_00010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0010_01000001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (cmp.eq(r4.new, #2)) jump:nt $+0x32c }");
    test_display(&0b0010_01010101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gtu(r4.new, #2)) jump:t $+0x32c }");
    test_display(&0b0010_01011001_0100_11_0_00010_100_10110u32.to_le_bytes(), "{ if (tstbit(r4.new, #0)) jump:nt $+0x32c }");
    test_display(&0b0010_01011101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!tstbit(r4.new, #0)) jump:t $+0x32c }");
    test_display(&0b0010_01101101_0100_11_1_00010_100_10110u32.to_le_bytes(), "{ if (!cmp.gt(r4.new, #-1)) jump:t $+0x32c }");
}

#[test]
fn inst_0011() {
    test_display(&0b0011_0010110_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (p1.new) r17:16 = memd(r4 + r2<<3) }");
    test_display(&0b0011_0010110_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (p1.new) r17:16 = memd(r4 + r2<<3) }");

    test_display(&0b0011_0101011_00100_11_1_0_0010_101_10000u32.to_le_bytes(), "{ if (!p1) memh(r4 + r2<<3) = r16.h }");
    test_display(&0b0011_0101101_00100_11_1_0_0010_101_10010u32.to_le_bytes(), "{ if (!p1) memw(r4 + r2<<3) = r2.new }");

    test_display(&0b0011_0101101_00100_11_1_0_0010_101_10010u32.to_le_bytes(), "{ if (!p1) memw(r4 + r2<<3) = r2.new }");

    test_invalid(&0b0011_0101111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0110001_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0110111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0111001_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0011_0111111_00100_11_1_0_0010_101_10010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1000000_00100_11_1_0_0010_101_11111u32.to_le_bytes(), "{ if (p1) memb(r4+#5) = #-1 }");
    test_display(&0b0011_1000100_00100_11_1_0_0010_101_11111u32.to_le_bytes(), "{ if (!p1) memb(r4+#5) = #-1 }");
    test_invalid(&0b0011_1000111_00100_11_1_0_0010_101_11111u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1010000_00100_11_1_0_0010_100_11111u32.to_le_bytes(), "{ lr = memb(r4 + r2<<3) }");
    test_display(&0b0011_1010001_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ fp = memub(r4 + r2<<3) }");

    test_display(&0b0011_1011010_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ memh(r4 + r2<<3) = fp }");
    test_display(&0b0011_1011011_00100_11_1_0_0010_100_11110u32.to_le_bytes(), "{ memh(r4 + r2<<3) = r30.h }");
    test_display(&0b0011_1011101_00100_11_1_0_0010_100_10110u32.to_le_bytes(), "{ memw(r4 + r2<<3) = r6.new }");

    test_display(&0b0011_1100000_00100_11_1_1_0010_100_10110u32.to_le_bytes(), "{ memb(r4+#37) = #-106 }");
    test_display(&0b0011_1100001_00100_11_1_1_0010_100_10110u32.to_le_bytes(), "{ memh(r4+#74) = #-106 }");
    test_display(&0b0011_1100010_00100_11_1_1_0010_100_10110u32.to_le_bytes(), "{ memw(r4+#148) = #-106 }");
    test_invalid(&0b0011_1100011_00100_11_1_1_0010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1110000_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memb(r4+#37) += r22 }");
    test_display(&0b0011_1110000_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memb(r4+#37) -= r22 }");
    test_display(&0b0011_1110000_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memb(r4+#37) &= r22 }");
    test_display(&0b0011_1110000_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memb(r4+#37) |= r22 }");
    test_display(&0b0011_1110001_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memh(r4+#74) += r22 }");
    test_display(&0b0011_1110001_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memh(r4+#74) -= r22 }");
    test_display(&0b0011_1110001_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memh(r4+#74) &= r22 }");
    test_display(&0b0011_1110001_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memh(r4+#74) |= r22 }");
    test_display(&0b0011_1110010_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memw(r4+#148) += r22 }");
    test_display(&0b0011_1110010_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memw(r4+#148) -= r22 }");
    test_display(&0b0011_1110010_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memw(r4+#148) &= r22 }");
    test_display(&0b0011_1110010_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memw(r4+#148) |= r22 }");
    test_invalid(&0b0011_1110011_00100_11_0_1_0010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0011_1111000_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memb(r4+#37) += #0x16 }");
    test_display(&0b0011_1111000_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memb(r4+#37) -= #0x16 }");
    test_display(&0b0011_1111000_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memb(r4+#37) = clrbit(#0x16) }");
    test_display(&0b0011_1111000_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memb(r4+#37) = setbit(#0x16) }");
    test_display(&0b0011_1111001_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memh(r4+#74) += #0x16 }");
    test_display(&0b0011_1111001_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memh(r4+#74) -= #0x16 }");
    test_display(&0b0011_1111001_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memh(r4+#74) = clrbit(#0x16) }");
    test_display(&0b0011_1111001_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memh(r4+#74) = setbit(#0x16) }");
    test_display(&0b0011_1111010_00100_11_0_1_0010_100_10110u32.to_le_bytes(), "{ memw(r4+#148) += #0x16 }");
    test_display(&0b0011_1111010_00100_11_0_1_0010_101_10110u32.to_le_bytes(), "{ memw(r4+#148) -= #0x16 }");
    test_display(&0b0011_1111010_00100_11_0_1_0010_110_10110u32.to_le_bytes(), "{ memw(r4+#148) = clrbit(#0x16) }");
    test_display(&0b0011_1111010_00100_11_0_1_0010_111_10110u32.to_le_bytes(), "{ memw(r4+#148) = setbit(#0x16) }");
    test_invalid(&0b0011_1111011_00100_11_0_1_0010_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
}

#[test]
fn inst_0100() {
    test_display(&0b0100_0000_000_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2) memb(r10+#45) = r7 }");
    test_display(&0b0100_0000_010_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2) memh(r10+#90) = r7 }");
    test_display(&0b0100_0000_011_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2) memh(r10+#90) = r7.h }");
    test_display(&0b0100_0000_100_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2) memw(r10+#180) = r7 }");
    test_display(&0b0100_0000_101_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2) memb(r10+#45) = r7.new }");
    test_display(&0b0100_0000_101_01010_11_1_01111_011_01010u32.to_le_bytes(), "{ if (p2) memh(r10+#90) = r7.new }");
    test_display(&0b0100_0000_101_01010_11_1_10111_011_01010u32.to_le_bytes(), "{ if (p2) memw(r10+#180) = r7.new }");
    test_invalid(&0b0100_0000_101_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0000_110_01010_11_1_00110_011_01010u32.to_le_bytes(), "{ if (p2) memd(r10+#360) = r7:6 }");
    test_invalid(&0b0100_0000_111_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0010_000_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2) memb(r10+#45) = r7 }");
    test_display(&0b0100_0010_010_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2) memh(r10+#90) = r7 }");
    test_display(&0b0100_0010_011_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2) memh(r10+#90) = r7.h }");
    test_display(&0b0100_0010_100_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2) memw(r10+#180) = r7 }");
    test_display(&0b0100_0010_101_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2) memb(r10+#45) = r7.new }");
    test_display(&0b0100_0010_101_01010_11_1_01111_011_01010u32.to_le_bytes(), "{ if (!p2) memh(r10+#90) = r7.new }");
    test_display(&0b0100_0010_101_01010_11_1_10111_011_01010u32.to_le_bytes(), "{ if (!p2) memw(r10+#180) = r7.new }");
    test_invalid(&0b0100_0010_101_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0010_110_01010_11_1_00110_011_01010u32.to_le_bytes(), "{ if (!p2) memd(r10+#360) = r7:6 }");
    test_invalid(&0b0100_0010_111_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0100_000_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2.new) memb(r10+#45) = r7 }");
    test_display(&0b0100_0100_010_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2.new) memh(r10+#90) = r7 }");
    test_display(&0b0100_0100_011_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2.new) memh(r10+#90) = r7.h }");
    test_display(&0b0100_0100_100_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2.new) memw(r10+#180) = r7 }");
    test_display(&0b0100_0100_101_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (p2.new) memb(r10+#45) = r7.new }");
    test_display(&0b0100_0100_101_01010_11_1_01111_011_01010u32.to_le_bytes(), "{ if (p2.new) memh(r10+#90) = r7.new }");
    test_display(&0b0100_0100_101_01010_11_1_10111_011_01010u32.to_le_bytes(), "{ if (p2.new) memw(r10+#180) = r7.new }");
    test_invalid(&0b0100_0100_101_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0100_110_01010_11_1_00110_011_01010u32.to_le_bytes(), "{ if (p2.new) memd(r10+#360) = r7:6 }");
    test_invalid(&0b0100_0100_111_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0110_000_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memb(r10+#45) = r7 }");
    test_display(&0b0100_0110_010_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memh(r10+#90) = r7 }");
    test_display(&0b0100_0110_011_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memh(r10+#90) = r7.h }");
    test_display(&0b0100_0110_100_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memw(r10+#180) = r7 }");
    test_display(&0b0100_0110_101_01010_11_1_00111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memb(r10+#45) = r7.new }");
    test_display(&0b0100_0110_101_01010_11_1_01111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memh(r10+#90) = r7.new }");
    test_display(&0b0100_0110_101_01010_11_1_10111_011_01010u32.to_le_bytes(), "{ if (!p2.new) memw(r10+#180) = r7.new }");
    test_invalid(&0b0100_0110_101_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0110_110_01010_11_1_00110_011_01010u32.to_le_bytes(), "{ if (!p2.new) memd(r10+#360) = r7:6 }");
    test_invalid(&0b0100_0110_111_01010_11_1_11111_011_01010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    // now for some loads
    test_display(&0b0100_0001_000_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r8 = memb(r10+#45) }");
    test_display(&0b0100_0001_001_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r8 = memub(r10+#45) }");
    test_display(&0b0100_0001_010_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r8 = memh(r10+#90) }");
    test_display(&0b0100_0001_011_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r8 = memuh(r10+#90) }");
    test_display(&0b0100_0001_100_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r8 = memw(r10+#180) }");
    test_invalid(&0b0100_0001_101_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0001_110_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2) r9:8 = memd(r10+#360) }");
    test_invalid(&0b0100_0001_111_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0011_000_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r8 = memb(r10+#45) }");
    test_display(&0b0100_0011_001_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r8 = memub(r10+#45) }");
    test_display(&0b0100_0011_010_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r8 = memh(r10+#90) }");
    test_display(&0b0100_0011_011_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r8 = memuh(r10+#90) }");
    test_display(&0b0100_0011_100_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r8 = memw(r10+#180) }");
    test_invalid(&0b0100_0011_101_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0011_110_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2) r9:8 = memd(r10+#360) }");
    test_invalid(&0b0100_0011_111_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0101_000_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r8 = memb(r10+#45) }");
    test_display(&0b0100_0101_001_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r8 = memub(r10+#45) }");
    test_display(&0b0100_0101_010_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r8 = memh(r10+#90) }");
    test_display(&0b0100_0101_011_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r8 = memuh(r10+#90) }");
    test_display(&0b0100_0101_100_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r8 = memw(r10+#180) }");
    test_invalid(&0b0100_0101_101_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0101_110_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (p2.new) r9:8 = memd(r10+#360) }");
    test_invalid(&0b0100_0101_111_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_0111_000_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r8 = memb(r10+#45) }");
    test_display(&0b0100_0111_001_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r8 = memub(r10+#45) }");
    test_display(&0b0100_0111_010_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r8 = memh(r10+#90) }");
    test_display(&0b0100_0111_011_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r8 = memuh(r10+#90) }");
    test_display(&0b0100_0111_100_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r8 = memw(r10+#180) }");
    test_invalid(&0b0100_0111_101_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_0111_110_01010_11_1_10101_101_01000u32.to_le_bytes(), "{ if (!p2.new) r9:8 = memd(r10+#360) }");
    test_invalid(&0b0100_0111_111_01010_11_1_11101_101_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_100_0000_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memb(gp+#0x1533) = r6 }");
    test_display(&0b0100_100_0010_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memh(gp+#0x2a66) = r6 }");
    test_display(&0b0100_100_0011_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memh(gp+#0x2a66) = r6.h }");
    test_display(&0b0100_100_0100_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memw(gp+#0x54cc) = r6 }");
    test_display(&0b0100_100_0101_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memb(gp+#0x1533) = r6.new }");
    test_display(&0b0100_100_0101_01010_11_1_01110_001_10011u32.to_le_bytes(), "{ memh(gp+#0x2a66) = r6.new }");
    test_display(&0b0100_100_0101_01010_11_1_10110_001_10011u32.to_le_bytes(), "{ memw(gp+#0x54cc) = r6.new }");
    test_invalid(&0b0100_100_0101_01010_11_1_11110_001_10011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_100_0110_01010_11_1_00110_001_10011u32.to_le_bytes(), "{ memd(gp+#0xa998) = r7:6 }");
    test_invalid(&0b0100_100_0111_01010_11_1_11110_001_10011u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0100_100_1000_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r6 = memb(gp+#0x1533) }");
    test_display(&0b0100_100_1001_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r6 = memub(gp+#0x1533) }");
    test_display(&0b0100_100_1010_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r6 = memh(gp+#0x2a66) }");
    test_display(&0b0100_100_1011_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r6 = memuh(gp+#0x2a66) }");
    test_display(&0b0100_100_1100_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r6 = memw(gp+#0x54cc) }");
    test_invalid(&0b0100_100_1101_01010_11_1_00110011_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0100_100_1110_01010_11_1_00110011_00110u32.to_le_bytes(), "{ r7:6 = memd(gp+#0xa998) }");
    test_invalid(&0b0100_100_1111_01010_11_1_00110011_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
}

#[test]
fn inst_0101() {
    test_invalid(&0b0101_000_0100_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_000_0101_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ callr r1 }");
    test_display(&0b0101_000_0110_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ callrh r1 }");
    test_invalid(&0b0101_000_0111_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_000_1000_00001_11_0_00010_000_00000u32.to_le_bytes(), "{ if (p2) callr r1 }");
    test_display(&0b0101_000_1001_00001_11_0_00010_000_00000u32.to_le_bytes(), "{ if (!p2) callr r1 }");
    test_invalid(&0b0101_000_1010_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b0101_001_0011_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_001_0100_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ jumpr r1 }");
    test_display(&0b0101_001_0101_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ hintjr(r1) }");
    test_display(&0b0101_001_0110_00001_11_0_00000_000_00000u32.to_le_bytes(), "{ jumprh r1 }");
    test_invalid(&0b0101_001_0111_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0101_001_1000_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0101_001_1001_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_001_1010_00001_11_0_01011_000_00000u32.to_le_bytes(), "{ if (p3.new) jumpr:nt r1 }");
    test_display(&0b0101_001_1011_00001_11_0_01011_000_00000u32.to_le_bytes(), "{ if (!p3.new) jumpr:nt r1 }");
    test_invalid(&0b0101_001_1100_00001_11_0_00000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0101_010_0000_00000_11_0_01000_000_00010u32.to_le_bytes(), "{ trap0(#0x40) }");
    test_display(&0b0101_010_0010_00010_11_0_01000_000_00010u32.to_le_bytes(), "{ pause(#0x240) }");
    test_display(&0b0101_010_0100_00010_11_0_01000_000_00010u32.to_le_bytes(), "{ trap1(r2, #0x40) }");
    test_invalid(&0b0101_010_0110_00010_11_0_01000_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b0101_011_0101_00010_11_0_01000_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0101_011_0110_00010_11_0_00000_000_00000u32.to_le_bytes(), "{ icinva(r2) }");
    test_display(&0b0101_011_1110_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ isync }");
    test_display(&0b0101_011_1111_00000_11_0_10000_000_00010u32.to_le_bytes(), "{ unpause }");

    test_display(&0b0101_100_0000_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ jump $+0x4 }");
    test_display(&0b0101_101_0000_00000_11_0_00000_000_00010u32.to_le_bytes(), "{ call $+0x4 }");

    test_display(&0b0101_110_0000_00000_11_0_00001_000_00010u32.to_le_bytes(), "{ if (p1) jump:nt $+0x4 }");
    test_display(&0b0101_110_0001_00000_11_0_01001_000_00010u32.to_le_bytes(), "{ if (!p1.new) jump:nt $+0x4 }");
    test_display(&0b0101_110_1001_00000_11_0_00001_000_00010u32.to_le_bytes(), "{ if (!p1) call $+0x4 }");
    test_invalid(&0b0101_110_1001_00000_11_0_01001_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
}

#[test]
fn inst_0110() {
    test_display(&0b0110_0000000_00110_11_0_00010_000_01000u32.to_le_bytes(), "{ loop0($+0x24, r6) }");
    test_display(&0b0110_0000001_00110_11_0_00010_000_01000u32.to_le_bytes(), "{ loop1($+0x24, r6) }");
    test_invalid(&0b0110_0000010_00110_11_0_00010_000_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b0110_0000011_00110_11_0_00010_000_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b0110_0000101_00110_11_0_00010_000_01000u32.to_le_bytes(), "{ p3 = sp1loop0($+0x24, r6) }");
    test_display(&0b0110_0000110_00110_11_0_00010_000_01000u32.to_le_bytes(), "{ p3 = sp2loop0($+0x24, r6) }");
    test_display(&0b0110_0000111_00110_11_0_00010_000_01000u32.to_le_bytes(), "{ p3 = sp3loop0($+0x24, r6) }");

    // TODO: test signed (negative) offsets
    test_display(&0b0110_0001001_00110_11_1_000101_00_10110u32.to_le_bytes(), "{ if (r6!=#0) jump:nt $-0x1ad4 }");
    test_display(&0b0110_0001001_00110_11_1_100101_00_10110u32.to_le_bytes(), "{ if (r6!=#0) jump:t $-0x1ad4 }");
    test_display(&0b0110_0001011_00110_11_1_000101_00_10110u32.to_le_bytes(), "{ if (r6>=#0) jump:nt $-0x1ad4 }");
    test_display(&0b0110_0001011_00110_11_1_100101_00_10110u32.to_le_bytes(), "{ if (r6>=#0) jump:t $-0x1ad4 }");
    test_display(&0b0110_0001101_00110_11_1_000101_00_10110u32.to_le_bytes(), "{ if (r6==#0) jump:nt $-0x1ad4 }");
    test_display(&0b0110_0001101_00110_11_1_100101_00_10110u32.to_le_bytes(), "{ if (r6==#0) jump:t $-0x1ad4 }");
    test_display(&0b0110_0001111_00110_11_1_000101_00_10110u32.to_le_bytes(), "{ if (r6<=#0) jump:nt $-0x1ad4 }");
    test_display(&0b0110_0001111_00110_11_1_100101_00_10110u32.to_le_bytes(), "{ if (r6<=#0) jump:t $-0x1ad4 }");

    test_display(&0b0110_0010001_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ c22 = r6 }");
    test_display(&0b0110_0010010_00110_11_0_00000_000_01000u32.to_le_bytes(), "{ trace(r6) }");
    test_display(&0b0110_0010010_00110_11_0_00000_001_01000u32.to_le_bytes(), "{ diag(r6) }");
    test_display(&0b0110_0010010_00110_11_0_00100_010_01000u32.to_le_bytes(), "{ diag0(r7:6, r5:4) }");
    test_display(&0b0110_0010010_00110_11_0_00100_011_01000u32.to_le_bytes(), "{ diag1(r7:6, r5:4) }");

    test_display(&0b0110_0011001_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ c23:22 = r7:6 }");

    test_display(&0b0110_1000000_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ r23:22 = c7:6 }");

    test_display(&0b0110_1001000_00110_11_0_000101_00_01010u32.to_le_bytes(), "{ loop0($+0x24, #0xd2) }");
    test_display(&0b0110_1001001_00110_11_0_000101_00_01010u32.to_le_bytes(), "{ loop1($+0x24, #0xd2) }");
    test_display(&0b0110_1001101_00110_11_0_000101_00_01010u32.to_le_bytes(), "{ p3 = sp1loop0($+0x24, #0xd2) }");
    test_display(&0b0110_1001110_00110_11_0_000101_00_01010u32.to_le_bytes(), "{ p3 = sp2loop0($+0x24, #0xd2) }");
    test_display(&0b0110_1001111_00110_11_0_000101_00_01010u32.to_le_bytes(), "{ p3 = sp3loop0($+0x24, #0xd2) }");

    test_display(&0b0110_1010000_00110_11_0_000101_00_10110u32.to_le_bytes(), "{ r22 = m0 }");
    test_display(&0b0110_1010010_01001_11_0_000101_00_10110u32.to_le_bytes(), "{ r22 = add(pc, #0x5) }");

    test_display(&0b0110_1011000_00011_11_0_000100_00_00001u32.to_le_bytes(), "{ p1 = and(p2, p3) }");
    test_display(&0b0110_1011000_00011_11_1_000101_00_10001u32.to_le_bytes(), "{ p1 = fastcorner9(p3, p2) }");
    test_display(&0b0110_1011000_10011_11_0_000100_00_00001u32.to_le_bytes(), "{ p1 = and(p2, and(p3, p0)) }");
    test_display(&0b0110_1011000_10011_11_1_000101_00_10001u32.to_le_bytes(), "{ p1 = !fastcorner9(p3, p2) }");
    test_display(&0b0110_1011001_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p2, p3) }");
    test_display(&0b0110_1011001_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = and(p3, or(p2, p0)) }");
    test_display(&0b0110_1011010_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = xor(p2, p3) }");
    test_display(&0b0110_1011010_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p3, and(p2, p0)) }");
    test_display(&0b0110_1011011_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = and(p2, !p3) }");
    test_display(&0b0110_1011011_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p3, or(p2, p0)) }");
    test_display(&0b0110_1011100_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = any8(p3) }");
    test_display(&0b0110_1011100_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = and(p3, and(p2, !p0)) }");
    test_display(&0b0110_1011101_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = all8(p3) }");
    test_display(&0b0110_1011101_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = and(p3, or(p2, !p0)) }");
    test_display(&0b0110_1011110_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = not(p3) }");
    test_display(&0b0110_1011110_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p3, and(p2, !p0)) }");
    test_display(&0b0110_1011111_00011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p2, !p3) }");
    test_display(&0b0110_1011111_10011_11_0_000100_00_10001u32.to_le_bytes(), "{ p1 = or(p3, or(p2, !p0)) }");

    test_display(&0b0110_1111111_01010_11_0_001100_10_00011u32.to_le_bytes(), "{ r3 = movlen(r6, r11:10) }");
}

#[test]
fn inst_0111() {
    test_invalid(&0b0111_0000010_00000_11_0_0_0000_000_00000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_0000000_00011_11_1_0_1101_000_00100u32.to_le_bytes(), "{ if (!p1.new) r4 = aslh(r3) }");
    test_display(&0b0111_0000011_00001_11_0_0_0000_000_00100u32.to_le_bytes(), "{ r4 = r1 }");
    test_invalid(&0b0111_0000011_00001_11_1_0_0000_000_00100u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_0001011_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ r2.l = #0x4020 }");
    test_display(&0b0111_0001101_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ r2.l = #0x8020 }");
    test_display(&0b0111_0010101_00010_11_0_0_0000_001_00000u32.to_le_bytes(), "{ r2.h = #0x8020 }");

    test_display(&0b0111_0011001_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ r16 = mux(p1, r6, #7) }");
    test_display(&0b0111_0011101_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ r16 = mux(p1, #7, r6) }");

    test_display(&0b0111_0011000_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r17:16 = combine(r6, #7) }");
    test_display(&0b0111_0011001_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r17:16 = combine(#7, r6) }");
    test_display(&0b0111_0011010_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r16 = cmp.eq(r6, #7) }");
    test_display(&0b0111_0011011_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r16 = !cmp.eq(r6, #7) }");

    test_display(&0b0111_0100010_00110_11_0_0_0000_111_10000u32.to_le_bytes(), "{ if (p2) r16 = add(r6, #7) }");
    test_display(&0b0111_0100010_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (p2.new) r16 = add(r6, #7) }");
    test_display(&0b0111_0100110_00110_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = add(r6, #7) }");

    test_display(&0b0111_0101001_00110_11_1_0_0000_111_00001u32.to_le_bytes(), "{ p1 = cmp.eq(r6, #-249) }");
    test_display(&0b0111_0101001_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ p1 = !cmp.eq(r6, #-249) }");
    test_display(&0b0111_0101011_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ p1 = !cmp.gt(r6, #-249) }");
    test_display(&0b0111_0101100_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ p1 = !cmp.gtu(r6, #0x107) }");
    test_invalid(&0b0111_0101101_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b0111_0101110_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_0110001_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ r17 = and(r6, #-249) }");
    test_display(&0b0111_0110011_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ r17 = sub(#-249, r6) }");
    test_display(&0b0111_0110101_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ r17 = or(r6, #-249) }");
    test_invalid(&0b0111_0110111_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b0111_0111111_00110_11_1_0_0000_111_10001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_1000110_00110_11_1_0_0000_111_10001u32.to_le_bytes(), "{ r17 = #-15929 }");

    test_display(&0b0111_1100010_00000_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r17:16 = combine(#7, #-127) }");
    test_display(&0b0111_1100100_10000_11_1_0_0000_111_10000u32.to_le_bytes(), "{ r17:16 = combine(#7, #0x21) }");

    test_invalid(&0b0111_1101100_10000_11_1_0_0000_111_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b0111_1110010_00100_11_0_0_0000_111_10000u32.to_le_bytes(), "{ if (p2) r16 = #1031 }");
    test_display(&0b0111_1110110_00100_11_1_0_0000_111_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = #1031 }");
}

#[test]
fn inst_1000() {
    test_display(&0b1000_0000000_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = asr(r5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = lsr(r5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = asl(r5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = rol(r5:4, #0x6) }");
    test_display(&0b1000_0000000_00100_11_000000_100_10110u32.to_le_bytes(), "{ r23:22 = vsathub(r5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_101_10110u32.to_le_bytes(), "{ r23:22 = vsatwuh(r5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_110_10110u32.to_le_bytes(), "{ r23:22 = vsatwh(r5:4) }");
    test_display(&0b1000_0000000_00100_11_000000_111_10110u32.to_le_bytes(), "{ r23:22 = vsathb(r5:4) }");
    test_display(&0b1000_0000001_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vasrh(r5:4, #0x6):raw }");
    test_invalid(&0b1000_0000001_00100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000001_00100_11_010110_000_10110u32.to_le_bytes(), DecodeError::InvalidOperand);

    test_display(&0b1000_0000010_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vasrw(r5:4, #0x6) }");
    test_display(&0b1000_0000010_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vlsrw(r5:4, #0x6) }");
    test_display(&0b1000_0000010_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vaslw(r5:4, #0x6) }");
    test_invalid(&0b1000_0000010_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000010_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vabsh(r5:4) }");
    test_display(&0b1000_0000010_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vabsh(r5:4):sat }");
    test_display(&0b1000_0000010_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vabsw(r5:4) }");
    test_display(&0b1000_0000010_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vabsw(r5:4):sat }");

    test_invalid(&0b1000_0000011_00100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_0000100_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vasrh(r5:4, #0x6) }");
    test_display(&0b1000_0000100_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vlsrh(r5:4, #0x6) }");
    test_display(&0b1000_0000100_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vaslh(r5:4, #0x6) }");
    test_invalid(&0b1000_0000100_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000100_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = not(r5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = neg(r5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = abs(r5:4) }");
    test_display(&0b1000_0000100_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vconj(r5:4):sat }");

    test_invalid(&0b1000_0000101_00100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b1000_0000101_00100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000101_00100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_0000110_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = deinterleave(r5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = interleave(r5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = brev(r5:4) }");
    test_display(&0b1000_0000110_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = asr(r5:4, #0x6):rnd }");

    test_display(&0b1000_0000111_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = convert_df2d(r5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = convert_df2ud(r5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = convert_ud2df(r5:4) }");
    test_display(&0b1000_0000111_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = convert_d2df(r5:4) }");

    test_invalid(&0b1000_0000111_00100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1000_0000111_00100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0000111_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = convert_df2d(r5:4):chop }");
    test_display(&0b1000_0000111_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = convert_df2ud(r5:4):chop }");

    test_display(&0b1000_0001101_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = extractu(r5:4, #0x6, #0x2f) }");

    test_display(&0b1000_0010000_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= asr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= asl(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= rol(r5:4, #0x6) }");

    test_display(&0b1000_0010000_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += asr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += asl(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += rol(r5:4, #0x6) }");

    test_display(&0b1000_0010010_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 &= asr(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 &= lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 &= asl(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 &= rol(r5:4, #0x6) }");

    test_display(&0b1000_0010010_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 |= asr(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 |= lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 |= asl(r5:4, #0x6) }");
    test_display(&0b1000_0010010_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 |= rol(r5:4, #0x6) }");

    test_display(&0b1000_0010100_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 ^= asr(r5:4, #0x6) }");
    test_display(&0b1000_0010100_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 ^= lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010100_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 ^= asl(r5:4, #0x6) }");
    test_display(&0b1000_0010100_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 ^= rol(r5:4, #0x6) }");

    test_display(&0b1000_0010000_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= asr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= lsr(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= asl(r5:4, #0x6) }");
    test_display(&0b1000_0010000_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= rol(r5:4, #0x6) }");

    test_display(&0b1000_0011101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = insert(r21:20, #0x6, #0x2f) }");

    test_display(&0b1000_0100_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vsxtbh(r20) }");
    test_display(&0b1000_0100_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vzxtbh(r20) }");
    test_display(&0b1000_0100_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vsxthw(r20) }");
    test_display(&0b1000_0100_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vzxthw(r20) }");
    test_display(&0b1000_0100_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vsplath(r20) }");
    test_display(&0b1000_0100_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vsplatb(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = convert_sf2df(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = convert_uw2df(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = convert_w2df(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = convert_sf2ud(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = convert_sf2d(r20) }");
    test_display(&0b1000_0100_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = convert_sf2ud(r20):chop }");
    test_display(&0b1000_0100_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = convert_sf2d(r20):chop }");
    test_invalid(&0b1000_0100_100_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_0101_000_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = tstbit(r20, #0x6) }");
    test_display(&0b1000_0101_001_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = !tstbit(r20, #0x6) }");
    test_display(&0b1000_0101_010_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = r20 }");
    test_invalid(&0b1000_0101_011_10100_11_000110_111_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0101_100_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = bitsclr(r20, #0x6) }");
    test_display(&0b1000_0101_101_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = !bitsclr(r20, #0x6) }");
    test_invalid(&0b1000_0101_110_10100_11_000110_111_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1000_0101_111_10100_11_000110_111_00010u32.to_le_bytes(), "{ p2 = sfclass(r20, #0x6) }");

    test_display(&0b1000_0110_000_00000_11_000010_111_01010u32.to_le_bytes(), "{ r11:10 = mask(p2) }");

    test_display(&0b1000_0111_001_00001_11_000010_001_01010u32.to_le_bytes(), "{ r10 = tableidxb(r1, #0x9, #2):raw }");
    test_display(&0b1000_0111_011_00001_11_000010_001_01010u32.to_le_bytes(), "{ r10 = tableidxh(r1, #0x9, #2):raw }");
    test_display(&0b1000_0111_101_00001_11_000010_001_01010u32.to_le_bytes(), "{ r10 = tableidxw(r1, #0x9, #2):raw }");
    test_display(&0b1000_0111_111_00001_11_000010_001_01010u32.to_le_bytes(), "{ r10 = tableidxd(r1, #0x9, #2):raw }");

    test_display(&0b1000_1000_000_00010_11_000111_000_00110u32.to_le_bytes(), "{ r6 = vsathub(r3:2) }");
    test_display(&0b1000_1000_000_00010_11_000111_001_00110u32.to_le_bytes(), "{ r6 = convert_df2sf(r3:2) }");
    test_display(&0b1000_1000_000_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = vsatwh(r3:2) }");
    test_display(&0b1000_1000_000_00010_11_000111_100_00110u32.to_le_bytes(), "{ r6 = vsatwuh(r3:2) }");
    test_display(&0b1000_1000_000_00010_11_000111_110_00110u32.to_le_bytes(), "{ r6 = vsathb(r3:2) }");
    test_display(&0b1000_1000_001_00010_11_000111_001_00110u32.to_le_bytes(), "{ r6 = convert_ud2sf(r3:2) }");
    test_display(&0b1000_1000_010_00010_11_000111_000_00110u32.to_le_bytes(), "{ r6 = clb(r3:2) }");
    test_display(&0b1000_1000_010_00010_11_000111_001_00110u32.to_le_bytes(), "{ r6 = convert_d2sf(r3:2) }");
    test_display(&0b1000_1000_010_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = cl0(r3:2) }");
    test_display(&0b1000_1000_010_00010_11_000111_100_00110u32.to_le_bytes(), "{ r6 = cl1(r3:2) }");
    test_display(&0b1000_1000_011_00010_11_000111_000_00110u32.to_le_bytes(), "{ r6 = normamt(r3:2) }");
    test_display(&0b1000_1000_011_00010_11_000111_001_00110u32.to_le_bytes(), "{ r7:6 = convert_sf2ud(r2) }");
    test_display(&0b1000_1000_011_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = add(clb(r3:2), #7) }");
    test_display(&0b1000_1000_011_00010_11_000111_011_00110u32.to_le_bytes(), "{ r6 = popcount(r3:2) }");
    test_display(&0b1000_1000_011_00010_11_000111_100_00110u32.to_le_bytes(), "{ r6 = vasrhub(r3:2, #0x7):raw }");
    test_display(&0b1000_1000_011_00010_11_000111_101_00110u32.to_le_bytes(), "{ r6 = vasrhub(r3:2, #0x7):sat }");
    test_display(&0b1000_1000_100_00010_11_000111_000_00110u32.to_le_bytes(), "{ r6 = vtrunohb(r3:2) }");
    test_display(&0b1000_1000_100_00010_11_000111_001_00110u32.to_le_bytes(), "{ r7:6 = convert_sf2d(r2) }");
    test_display(&0b1000_1000_100_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = vtrunehb(r3:2) }");
    test_display(&0b1000_1000_100_00010_11_000111_100_00110u32.to_le_bytes(), "{ r6 = vrndwh(r3:2) }");
    test_display(&0b1000_1000_100_00010_11_000111_110_00110u32.to_le_bytes(), "{ r6 = vrndwh(r3:2):sat }");
    test_display(&0b1000_1000_101_00010_11_000111_001_00110u32.to_le_bytes(), "{ r7:6 = convert_sf2ud(r2):chop }");
    test_display(&0b1000_1000_110_00010_11_000111_000_00110u32.to_le_bytes(), "{ r6 = sat(r3:2) }");
    test_display(&0b1000_1000_110_00010_11_000111_001_00110u32.to_le_bytes(), "{ r6 = round(r3:2):sat }");
    test_display(&0b1000_1000_110_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = vasrw(r3:2, #7) }");
    test_display(&0b1000_1000_110_00010_11_000111_100_00110u32.to_le_bytes(), "{ r7:6 = bitsplit(r2, #0x7) }");
    test_display(&0b1000_1000_110_00010_11_000111_101_00110u32.to_le_bytes(), "{ r6 = clip(r2, #0x7) }");
    test_display(&0b1000_1000_110_00010_11_000111_110_00110u32.to_le_bytes(), "{ r7:6 = vclip(r3:2, #0x7) }");
    test_display(&0b1000_1000_111_00010_11_000111_001_00110u32.to_le_bytes(), "{ r7:6 = convert_sf2d(r2):chop }");
    test_display(&0b1000_1000_111_00010_11_000111_010_00110u32.to_le_bytes(), "{ r6 = ct0(r3:2) }");
    test_display(&0b1000_1000_111_00010_11_000111_100_00110u32.to_le_bytes(), "{ r6 = ct1(r3:2) }");

    test_display(&0b1000_1001_000_00011_11_000001_000_00110u32.to_le_bytes(), "{ r6 = vitpack(p3, p1) }");
    test_display(&0b1000_1001_010_00011_11_000000_000_00110u32.to_le_bytes(), "{ r6 = p3 }");

    test_display(&0b1000_1010_101_00010_11_011010_101_00110u32.to_le_bytes(), "{ r7:6 = extract(r3:2, #0x1a, #0x2d) }");

    test_display(&0b1000_1011_001_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = convert_uw2sf(r2) }");
    test_display(&0b1000_1011_010_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = convert_w2sf(r2) }");
    test_display(&0b1000_1011_011_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = convert_sf2uw(r2) }");
    test_display(&0b1000_1011_011_00010_11_011010_001_00110u32.to_le_bytes(), "{ r6 = convert_sf2uw(r2):chop }");
    test_display(&0b1000_1011_100_00010_11_011010_010_00110u32.to_le_bytes(), "{ r6 = convert_sf2w(r2) }");
    test_display(&0b1000_1011_100_00010_11_011010_011_00110u32.to_le_bytes(), "{ r6 = convert_sf2w(r2):chop }");
    test_display(&0b1000_1011_101_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = sffixupr(r2) }");
    test_display(&0b1000_1011_111_00010_11_011010_011_00110u32.to_le_bytes(), "{ r6, p3 = sfinvsqrta(r2) }");

    test_display(&0b1000_1100_000_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = asr(r2, #0x1a) }");
    test_display(&0b1000_1100_000_00010_11_011010_001_00110u32.to_le_bytes(), "{ r6 = lsr(r2, #0x1a) }");
    test_display(&0b1000_1100_000_00010_11_011010_010_00110u32.to_le_bytes(), "{ r6 = asl(r2, #0x1a) }");
    test_display(&0b1000_1100_000_00010_11_011010_011_00110u32.to_le_bytes(), "{ r6 = rol(r2, #0x1a) }");
    test_display(&0b1000_1100_000_00010_11_011010_100_00110u32.to_le_bytes(), "{ r6 = clb(r2) }");
    test_display(&0b1000_1100_000_00010_11_011010_101_00110u32.to_le_bytes(), "{ r6 = cl0(r2) }");
    test_display(&0b1000_1100_000_00010_11_011010_110_00110u32.to_le_bytes(), "{ r6 = cl1(r2) }");
    test_display(&0b1000_1100_000_00010_11_011010_111_00110u32.to_le_bytes(), "{ r6 = normamt(r2) }");

    test_display(&0b1000_1100_001_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = add(clb(r2), #26) }");

    test_display(&0b1000_1100_010_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = asr(r2, #0x1a):rnd }");
    test_display(&0b1000_1100_010_00010_11_011010_010_00110u32.to_le_bytes(), "{ r6 = asl(r2, #0x1a):sat }");
    test_display(&0b1000_1100_010_00010_11_011010_100_00110u32.to_le_bytes(), "{ r6 = ct0(r2) }");
    test_display(&0b1000_1100_010_00010_11_011010_101_00110u32.to_le_bytes(), "{ r6 = ct1(r2) }");
    test_display(&0b1000_1100_010_00010_11_011010_110_00110u32.to_le_bytes(), "{ r6 = brev(r2) }");
    test_display(&0b1000_1100_010_00010_11_011010_111_00110u32.to_le_bytes(), "{ r6 = vsplatb(r2) }");

    test_display(&0b1000_1100_100_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = vsathb(r2) }");
    test_display(&0b1000_1100_100_00010_11_011010_010_00110u32.to_le_bytes(), "{ r6 = vsathub(r2) }");
    test_display(&0b1000_1100_100_00010_11_011010_100_00110u32.to_le_bytes(), "{ r6 = abs(r2) }");
    test_display(&0b1000_1100_100_00010_11_011010_101_00110u32.to_le_bytes(), "{ r6 = abs(r2):sat }");
    test_display(&0b1000_1100_100_00010_11_011010_110_00110u32.to_le_bytes(), "{ r6 = neg(r2):sat }");
    test_display(&0b1000_1100_100_00010_11_011010_111_00110u32.to_le_bytes(), "{ r6 = swiz(r2) }");

    test_display(&0b1000_1100_110_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = setbit(r2, #0x1a) }");
    test_display(&0b1000_1100_110_00010_11_011010_001_00110u32.to_le_bytes(), "{ r6 = clrbit(r2, #0x1a) }");
    test_display(&0b1000_1100_110_00010_11_011010_010_00110u32.to_le_bytes(), "{ r6 = togglebit(r2, #0x1a) }");

    test_display(&0b1000_1100_110_00010_11_011010_100_00110u32.to_le_bytes(), "{ r6 = sath(r2) }");
    test_display(&0b1000_1100_110_00010_11_011010_101_00110u32.to_le_bytes(), "{ r6 = satuh(r2) }");
    test_display(&0b1000_1100_110_00010_11_011010_110_00110u32.to_le_bytes(), "{ r6 = satub(r2) }");
    test_display(&0b1000_1100_110_00010_11_011010_111_00110u32.to_le_bytes(), "{ r6 = satb(r2) }");

    test_display(&0b1000_1100_111_00010_11_011010_000_00110u32.to_le_bytes(), "{ r6 = cround(r2, #0x1a) }");
    test_display(&0b1000_1100_111_00010_11_011010_010_00110u32.to_le_bytes(), "{ r7:6 = cround(r3:2, #0x1a) }");
    test_display(&0b1000_1100_111_00010_11_011010_100_00110u32.to_le_bytes(), "{ r6 = round(r2, #0x1a) }");
    test_display(&0b1000_1100_111_00010_11_011010_110_00110u32.to_le_bytes(), "{ r6 = round(r2, #0x1a):sat }");

    test_display(&0b1000_1101011_00100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = extractu(r4, #0x6, #0x18) }");
    test_display(&0b1000_1101011_00100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = mask(#0x6, #0x18) }");
    test_display(&0b1000_1101111_00100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = extract(r4, #0x6, #0x18) }");
    test_invalid(&0b1000_1101111_00100_11_100110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_1110000_00100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= asr(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= lsr(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= asl(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= rol(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += asr(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += lsr(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += asl(r5:4, #0x6) }");
    test_display(&0b1000_1110000_00100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += rol(r5:4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_000_10110u32.to_le_bytes(), "{ r22 &= asr(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_001_10110u32.to_le_bytes(), "{ r22 &= lsr(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_010_10110u32.to_le_bytes(), "{ r22 &= asl(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_011_10110u32.to_le_bytes(), "{ r22 &= rol(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_100_10110u32.to_le_bytes(), "{ r22 |= asr(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_101_10110u32.to_le_bytes(), "{ r22 |= lsr(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_110_10110u32.to_le_bytes(), "{ r22 |= asl(r4, #0x6) }");
    test_display(&0b1000_1110010_00100_11_000110_111_10110u32.to_le_bytes(), "{ r22 |= rol(r4, #0x6) }");
    test_display(&0b1000_1110100_00100_11_000110_000_10110u32.to_le_bytes(), "{ r22 ^= asr(r4, #0x6) }");
    test_display(&0b1000_1110100_00100_11_000110_001_10110u32.to_le_bytes(), "{ r22 ^= lsr(r4, #0x6) }");
    test_display(&0b1000_1110100_00100_11_000110_010_10110u32.to_le_bytes(), "{ r22 ^= asl(r4, #0x6) }");
    test_display(&0b1000_1110100_00100_11_000110_011_10110u32.to_le_bytes(), "{ r22 ^= rol(r4, #0x6) }");
    test_invalid(&0b1000_1110100_00100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1000_1111011_00100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = insert(r4, #0x6, #0x1f) }");
}

#[test]
fn inst_1001() {
    test_display(&0b1001_0000000_00010_11_0_00000_000_11110u32.to_le_bytes(), "{ r31:30 = deallocframe(r2):raw }");
    test_display(&0b1001_0010000_00010_11_000_000_000_00011u32.to_le_bytes(), "{ r3 = memw_locked(r2) }");
    test_display(&0b1001_0010000_00010_11_001_000_000_00011u32.to_le_bytes(), "{ r3 = memw_aq(r2) }");
    test_display(&0b1001_0010000_00010_11_010_000_000_00100u32.to_le_bytes(), "{ r5:4 = memd_locked(r2) }");
    test_display(&0b1001_0010000_00010_11_011_000_000_00100u32.to_le_bytes(), "{ r5:4 = memd_aq(r2) }");
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
    test_display(&0b1001_0100000_00010_11_000_111_111_11111u32.to_le_bytes(), "{ dcfetch(r2+#16376) }");
    test_display(&0b1001_0110000_00010_11_0000_00_000_10000u32.to_le_bytes(), "{ r17:16 = dealloc_return(r2):raw }");
    test_invalid(&0b1001_0110000_00010_11_0001_00_000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110000_00010_11_0010_01_000_10000u32.to_le_bytes(), "{ if (p1.new) r17:16 = dealloc_return(r2):nt:raw }");
    test_display(&0b1001_0110000_00010_11_0100_01_000_10000u32.to_le_bytes(), "{ if (p1) r17:16 = dealloc_return(r2):raw }");
    test_display(&0b1001_0110000_00010_11_0110_01_000_10000u32.to_le_bytes(), "{ if (p1.new) r17:16 = dealloc_return(r2):t:raw }");
    test_invalid(&0b1001_0110000_00010_11_1000_01_000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110000_00010_11_1010_01_000_10000u32.to_le_bytes(), "{ if (!p1.new) r17:16 = dealloc_return(r2):nt:raw }");
    test_display(&0b1001_0110000_00010_11_1100_01_000_10000u32.to_le_bytes(), "{ if (!p1) r17:16 = dealloc_return(r2):raw }");
    test_display(&0b1001_0110000_00010_11_1110_01_000_10000u32.to_le_bytes(), "{ if (!p1.new) r17:16 = dealloc_return(r2):t:raw }");
    test_display(&0b1001_0110001_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = membh(r2+#3648) }");
    test_display(&0b1001_0110010_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2+#3648) }");
    test_display(&0b1001_0110011_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memubh(r2+#3648) }");
    test_display(&0b1001_0110100_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2+#1824) }");
    test_display(&0b1001_0110101_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2+#7296) }");
    test_invalid(&0b1001_0000110_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0110111_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r17:16 = membh(r2+#7296) }");
    test_display(&0b1001_0111000_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memb(r2+#1824) }");
    test_display(&0b1001_0111001_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memub(r2+#1824) }");
    test_display(&0b1001_0111010_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memh(r2+#3648) }");
    test_display(&0b1001_0111011_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memuh(r2+#3648) }");
    test_display(&0b1001_0111100_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r16 = memw(r2+#7296) }");
    test_invalid(&0b1001_0001101_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_0111110_00010_11_1001_00_000_10000u32.to_le_bytes(), "{ r17:16 = memd(r2+#14592) }");
    test_invalid(&0b1001_0001111_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);

    // TODO: exercise these
    test_display(&0b1001_1000001_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = membh(r2++#0xe:circ(m1)) }");
    test_display(&0b1001_1000010_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2++#0xe:circ(m1)) }");
    test_display(&0b1001_1000011_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memubh(r2++#0xe:circ(m1)) }");
    test_display(&0b1001_1000100_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2++#0x7:circ(m1)) }");
    test_display(&0b1001_1000101_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2++#0x1c:circ(m1)) }");
    test_display(&0b1001_1000111_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r17:16 = membh(r2++#0x1c:circ(m1)) }");

    test_display(&0b1001_1000001_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = membh(r2++I:circ(m1)) }");
    test_display(&0b1001_1000010_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2++I:circ(m1)) }");
    test_display(&0b1001_1000011_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memubh(r2++I:circ(m1)) }");
    test_display(&0b1001_1000100_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2++I:circ(m1)) }");
    test_display(&0b1001_1000101_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2++I:circ(m1)) }");
    test_display(&0b1001_1000111_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r17:16 = membh(r2++I:circ(m1)) }");

    test_display(&0b1001_1001000_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memb(r2++#0x7:circ(m1)) }");
    test_display(&0b1001_1001001_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memub(r2++#0x7:circ(m1)) }");
    test_display(&0b1001_1001010_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memh(r2++#0xe:circ(m1)) }");
    test_display(&0b1001_1001011_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memuh(r2++#0xe:circ(m1)) }");
    test_display(&0b1001_1001100_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r16 = memw(r2++#0x1c:circ(m1)) }");
    test_display(&0b1001_1001110_00010_11_1000_00111_10000u32.to_le_bytes(), "{ r17:16 = memd(r2++#0x38:circ(m1)) }");

    test_display(&0b1001_1001000_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memb(r2++I:circ(m1)) }");
    test_display(&0b1001_1001001_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memub(r2++I:circ(m1)) }");
    test_display(&0b1001_1001010_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memh(r2++I:circ(m1)) }");
    test_display(&0b1001_1001011_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memuh(r2++I:circ(m1)) }");
    test_display(&0b1001_1001100_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r16 = memw(r2++I:circ(m1)) }");
    test_display(&0b1001_1001110_00010_11_1000_10000_10000u32.to_le_bytes(), "{ r17:16 = memd(r2++I:circ(m1)) }");
    test_display(&0b1001_1001111_00010_11_0010_00000_10000u32.to_le_bytes(), "{ r17:16 = pmemcpy(r8, r3:2) }");
    test_display(&0b1001_1001111_00010_11_0010_00001_10000u32.to_le_bytes(), "{ r17:16 = linecpy(r8, r3:2) }");
    test_invalid(&0b1001_1001111_00010_11_0010_00010_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1001_1010001_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = membh(r2++#0xe) }");
    test_display(&0b1001_1010010_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2++#0xe) }");
    test_display(&0b1001_1010011_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memubh(r2++#0xe) }");
    test_display(&0b1001_1010100_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2++#0x7) }");
    test_display(&0b1001_1010101_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2++#0x1c) }");
    test_display(&0b1001_1010111_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r17:16 = membh(r2++#0x1c) }");

    test_display(&0b1001_1010001_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = membh(r2=#0x23) }");
    test_display(&0b1001_1010010_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2=#0x23) }");
    test_display(&0b1001_1010011_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memubh(r2=#0x23) }");
    test_display(&0b1001_1010100_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2=#0x23) }");
    test_display(&0b1001_1010101_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2=#0x23) }");
    test_display(&0b1001_1010111_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r17:16 = membh(r2=#0x23) }");

    test_display(&0b1001_1011000_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memb(r2++#0x7) }");
    test_display(&0b1001_1011001_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memub(r2++#0x7) }");
    test_display(&0b1001_1011010_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memh(r2++#0xe) }");
    test_display(&0b1001_1011011_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memuh(r2++#0xe) }");
    test_display(&0b1001_1011100_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r16 = memw(r2++#0x1c) }");
    test_display(&0b1001_1011110_00010_11_0000_00111_10000u32.to_le_bytes(), "{ r17:16 = memd(r2++#0x38) }");

    test_display(&0b1001_1011000_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memb(r2=#0x23) }");
    test_display(&0b1001_1011001_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memub(r2=#0x23) }");
    test_display(&0b1001_1011010_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memh(r2=#0x23) }");
    test_display(&0b1001_1011011_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memuh(r2=#0x23) }");
    test_display(&0b1001_1011100_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r16 = memw(r2=#0x23) }");
    test_display(&0b1001_1011110_00010_11_0110_00011_10000u32.to_le_bytes(), "{ r17:16 = memd(r2=#0x23) }");

    test_display(&0b1001_1011000_00010_11_1001_00011_10000u32.to_le_bytes(), "{ if (p2) r16 = memb(r2++#0x3) }");
    test_display(&0b1001_1011000_00010_11_1011_00011_10000u32.to_le_bytes(), "{ if (!p2) r16 = memb(r2++#0x3) }");
    test_display(&0b1001_1011000_00010_11_1101_00011_10000u32.to_le_bytes(), "{ if (p2.new) r16 = memb(r2++#0x3) }");
    test_display(&0b1001_1011000_00010_11_1111_00011_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = memb(r2++#0x3) }");
    test_display(&0b1001_1011011_00010_11_1001_00011_10000u32.to_le_bytes(), "{ if (p2) r16 = memuh(r2++#0x6) }");
    test_display(&0b1001_1011011_00010_11_1011_00011_10000u32.to_le_bytes(), "{ if (!p2) r16 = memuh(r2++#0x6) }");
    test_display(&0b1001_1011011_00010_11_1101_00011_10000u32.to_le_bytes(), "{ if (p2.new) r16 = memuh(r2++#0x6) }");
    test_display(&0b1001_1011011_00010_11_1111_00011_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = memuh(r2++#0x6) }");
    test_display(&0b1001_1011110_00010_11_1001_00011_10000u32.to_le_bytes(), "{ if (p2) r17:16 = memd(r2++#0x18) }");
    test_display(&0b1001_1011110_00010_11_1011_00011_10000u32.to_le_bytes(), "{ if (!p2) r17:16 = memd(r2++#0x18) }");
    test_display(&0b1001_1011110_00010_11_1101_00011_10000u32.to_le_bytes(), "{ if (p2.new) r17:16 = memd(r2++#0x18) }");
    test_display(&0b1001_1011110_00010_11_1111_00011_10000u32.to_le_bytes(), "{ if (!p2.new) r17:16 = memd(r2++#0x18) }");

    // 1001_1100
    test_display(&0b1001_1100001_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = membh(r2++m1) }");
    test_display(&0b1001_1100010_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2++m1) }");
    test_display(&0b1001_1100011_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memubh(r2++m1) }");
    test_display(&0b1001_1100100_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2++m1) }");
    test_display(&0b1001_1100101_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2++m1) }");
    test_display(&0b1001_1100111_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r17:16 = membh(r2++m1) }");

    test_display(&0b1001_1100001_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = membh(r2<<1 + 0x23) }");
    test_display(&0b1001_1100010_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2<<1 + 0x23) }");
    test_display(&0b1001_1100011_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memubh(r2<<1 + 0x23) }");
    test_display(&0b1001_1100100_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2<<1 + 0x23) }");
    test_display(&0b1001_1100101_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2<<1 + 0x23) }");
    test_display(&0b1001_1100111_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r17:16 = membh(r2<<1 + 0x23) }");

    // 1001_1101
    test_display(&0b1001_1101000_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memb(r2++m1) }");
    test_display(&0b1001_1101001_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memub(r2++m1) }");
    test_display(&0b1001_1101010_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memh(r2++m1) }");
    test_display(&0b1001_1101011_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memuh(r2++m1) }");
    test_display(&0b1001_1101100_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r16 = memw(r2++m1) }");
    test_invalid(&0b1001_1101101_00010_11_1000_00000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_1101110_00010_11_1000_00000_10000u32.to_le_bytes(), "{ r17:16 = memd(r2++m1) }");
    test_invalid(&0b1001_1101111_00010_11_1000_00000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1001_1101000_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memb(r2<<1 + 0x23) }");
    test_display(&0b1001_1101001_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memub(r2<<1 + 0x23) }");
    test_display(&0b1001_1101010_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memh(r2<<1 + 0x23) }");
    test_display(&0b1001_1101011_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memuh(r2<<1 + 0x23) }");
    test_display(&0b1001_1101100_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r16 = memw(r2<<1 + 0x23) }");
    test_invalid(&0b1001_1101101_00010_11_0110_00111_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_1101110_00010_11_0110_00111_10000u32.to_le_bytes(), "{ r17:16 = memd(r2<<1 + 0x23) }");
    test_invalid(&0b1001_1101111_00010_11_0110_00111_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    // skipped forward to 1001_1110
    test_invalid(&0b1001_1110000_00010_11_1001_01000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_1110001_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = membh(r2++m1:brev) }");
    test_display(&0b1001_1110010_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r17:16 = memh_fifo(r2++m1:brev) }");
    test_display(&0b1001_1110011_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memubh(r2++m1:brev) }");
    test_display(&0b1001_1110100_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r17:16 = memb_fifo(r2++m1:brev) }");
    test_display(&0b1001_1110101_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r17:16 = memubh(r2++m1:brev) }");
    test_invalid(&0b1001_1110110_00010_11_1001_01000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_1110111_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r17:16 = membh(r2++m1:brev) }");

    test_invalid(&0b1001_1110000_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110001_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110010_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110011_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110100_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110101_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110110_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1001_1110111_00010_11_1001_01100_10000u32.to_le_bytes(), DecodeError::InvalidOperand);

    test_display(&0b1001_1111000_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memb(r2++m1:brev) }");
    test_display(&0b1001_1111001_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memub(r2++m1:brev) }");
    test_display(&0b1001_1111010_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memh(r2++m1:brev) }");
    test_display(&0b1001_1111011_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memuh(r2++m1:brev) }");
    test_display(&0b1001_1111100_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r16 = memw(r2++m1:brev) }");
    test_invalid(&0b1001_1111101_00010_11_1001_01000_10000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1001_1111110_00010_11_1001_01000_10000u32.to_le_bytes(), "{ r17:16 = memd(r2++m1:brev) }");

    test_display(&0b1001_1111000_00010_11_1001_01100_10000u32.to_le_bytes(), "{ if (p2) r16 = memb(r2=#0x5) }");
    test_display(&0b1001_1111000_00010_11_1011_01100_10000u32.to_le_bytes(), "{ if (!p2) r16 = memb(r2=#0x5) }");
    test_display(&0b1001_1111000_00010_11_1101_01100_10000u32.to_le_bytes(), "{ if (p2.new) r16 = memb(r2=#0x5) }");
    test_display(&0b1001_1111000_00010_11_1111_01100_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = memb(r2=#0x5) }");
    test_display(&0b1001_1111011_00010_11_1001_01100_10000u32.to_le_bytes(), "{ if (p2) r16 = memuh(r2=#0x5) }");
    test_display(&0b1001_1111011_00010_11_1011_01100_10000u32.to_le_bytes(), "{ if (!p2) r16 = memuh(r2=#0x5) }");
    test_display(&0b1001_1111011_00010_11_1101_01100_10000u32.to_le_bytes(), "{ if (p2.new) r16 = memuh(r2=#0x5) }");
    test_display(&0b1001_1111011_00010_11_1111_01100_10000u32.to_le_bytes(), "{ if (!p2.new) r16 = memuh(r2=#0x5) }");
    test_display(&0b1001_1111110_00010_11_1001_01100_10000u32.to_le_bytes(), "{ if (p2) r17:16 = memd(r2=#0x5) }");
    test_display(&0b1001_1111110_00010_11_1011_01100_10000u32.to_le_bytes(), "{ if (!p2) r17:16 = memd(r2=#0x5) }");
    test_display(&0b1001_1111110_00010_11_1101_01100_10000u32.to_le_bytes(), "{ if (p2.new) r17:16 = memd(r2=#0x5) }");
    test_display(&0b1001_1111110_00010_11_1111_01100_10000u32.to_le_bytes(), "{ if (!p2.new) r17:16 = memd(r2=#0x5) }");
}

#[test]
fn inst_1010() {
    test_display(&0b1010_0000000_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ dccleana(r2) }");
    test_display(&0b1010_0000001_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ dcinva(r2) }");
    test_display(&0b1010_0000010_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ dccleaninva(r2) }");
    test_invalid(&0b1010_0000011_00010_11_0_00100_000_00011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0000011_00010_11_0_00100_000_01111u32.to_le_bytes(), "{ release(r2):at }");
    test_display(&0b1010_0000011_00010_11_0_00100_001_01111u32.to_le_bytes(), "{ release(r2):st }");
    test_display(&0b1010_0000100_00010_11_0_00000_001_01111u32.to_le_bytes(), "{ allocframe(r2, #0x178):raw }");
    test_invalid(&0b1010_0000100_00010_11_0_01000_001_01111u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_display(&0b1010_0000101_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ memw_locked(r2, p3) = r4 }");
    test_invalid(&0b1010_0000101_00010_11_0_00100_000_00111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0000101_00010_11_0_00100_000_01011u32.to_le_bytes(), "{ memw_rl(r2):at = r4 }");
    test_display(&0b1010_0000101_00010_11_0_00100_001_01011u32.to_le_bytes(), "{ memw_rl(r2):st = r4 }");
    test_display(&0b1010_0000110_00010_11_0_00000_001_01111u32.to_le_bytes(), "{ dczeroa(r2) }");
    test_invalid(&0b1010_0000110_00010_11_1_00000_001_01111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0000111_00010_11_0_00100_000_00011u32.to_le_bytes(), "{ memd_locked(r2, p3) = r5:4 }");
    test_invalid(&0b1010_0000111_00010_11_0_00100_000_00111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0000111_00010_11_0_00100_000_01011u32.to_le_bytes(), "{ memd_rl(r2):at = r5:4 }");
    test_display(&0b1010_0000111_00010_11_0_00100_001_01011u32.to_le_bytes(), "{ memd_rl(r2):st = r5:4 }");
    test_display(&0b1010_0010000_00010_11_0_00100_001_01011u32.to_le_bytes(), "{ dckill }");
    test_display(&0b1010_0110000_00010_11_0_00100_000_01011u32.to_le_bytes(), "{ l2fetch(r2, r4) }");
    test_invalid(&0b1010_0110000_00010_11_0_00100_001_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0110100_00010_11_0_00100_000_01011u32.to_le_bytes(), "{ l2fetch(r2, r5:4) }");

    test_display(&0b1010_0101000_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memb(r2+#1291) = r4 }");
    test_display(&0b1010_0101010_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memh(r2+#2582) = r4 }");
    test_display(&0b1010_0101011_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memh(r2+#2582) = r4.h }");
    test_display(&0b1010_0101100_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memw(r2+#5164) = r4 }");
    test_display(&0b1010_0101101_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memb(r2+#1291) = r4.new }");
    test_display(&0b1010_0101101_00010_11_1_01100_000_01011u32.to_le_bytes(), "{ memh(r2+#2582) = r4.new }");
    test_display(&0b1010_0101101_00010_11_1_10100_000_01011u32.to_le_bytes(), "{ memw(r2+#5164) = r4.new }");
    test_invalid(&0b1010_0101101_00010_11_1_11100_000_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_0101110_00010_11_1_00100_000_01011u32.to_le_bytes(), "{ memd(r2+#10328) = r5:4 }");
    test_invalid(&0b1010_0101111_00010_11_1_11100_000_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1000000_00010_11_1_00000_000_01011u32.to_le_bytes(), "{ barrier }");
    test_display(&0b1010_1000000_00010_11_1_00000_111_01011u32.to_le_bytes(), "{ r11 = dmsyncht }");
    test_invalid(&0b1010_1000000_00010_11_1_00000_001_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1000000_00010_11_1_00000_010_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1000000_00010_11_1_00000_100_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1000001_00010_11_1_00000_100_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1000010_00010_11_1_00000_111_01011u32.to_le_bytes(), "{ syncht }");

    test_display(&0b1010_1001000_00010_11_1_00011_000_00010u32.to_le_bytes(), "{ memb(r2++I:circ(m1)) = r3 }");
    test_invalid(&0b1010_1001001_00010_11_1_00011_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1001010_00010_11_1_00011_000_00010u32.to_le_bytes(), "{ memh(r2++I:circ(m1)) = r3 }");
    test_display(&0b1010_1001011_00010_11_1_00011_000_00010u32.to_le_bytes(), "{ memh(r2++I:circ(m1)) = r3.h }");
    test_display(&0b1010_1001100_00010_11_1_00011_000_00010u32.to_le_bytes(), "{ memw(r2++I:circ(m1)) = r3 }");
    test_display(&0b1010_1001101_00010_11_1_00011_000_00010u32.to_le_bytes(), "{ memb(r2++I:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_1_01011_000_00010u32.to_le_bytes(), "{ memh(r2++I:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_1_10011_000_00010u32.to_le_bytes(), "{ memw(r2++I:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_0_10011_000_00010u32.to_le_bytes(), "{ memw(r2++I:circ(m0)) = r3.new }");
    test_invalid(&0b1010_1001101_00010_11_1_11011_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1001110_00010_11_1_00100_000_00010u32.to_le_bytes(), "{ memd(r2++I:circ(m1)) = r5:4 }");
    test_invalid(&0b1010_1001111_00010_11_1_00100_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1001000_00010_11_1_00011_010_01000u32.to_le_bytes(), "{ memb(r2++#0x9:circ(m1)) = r3 }");
    test_invalid(&0b1010_1001001_00010_11_1_00011_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1001010_00010_11_1_00011_010_01000u32.to_le_bytes(), "{ memh(r2++#0x12:circ(m1)) = r3 }");
    test_display(&0b1010_1001011_00010_11_1_00011_010_01000u32.to_le_bytes(), "{ memh(r2++#0x12:circ(m1)) = r3.h }");
    test_display(&0b1010_1001100_00010_11_1_00011_010_01000u32.to_le_bytes(), "{ memw(r2++#0x24:circ(m1)) = r3 }");
    test_display(&0b1010_1001101_00010_11_1_00011_010_01000u32.to_le_bytes(), "{ memb(r2++#0x9:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_1_01011_010_01000u32.to_le_bytes(), "{ memh(r2++#0x12:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_1_10011_010_01000u32.to_le_bytes(), "{ memw(r2++#0x24:circ(m1)) = r3.new }");
    test_display(&0b1010_1001101_00010_11_0_10011_010_01000u32.to_le_bytes(), "{ memw(r2++#0x24:circ(m0)) = r3.new }");
    test_invalid(&0b1010_1001101_00010_11_1_11011_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1001110_00010_11_1_00100_010_01000u32.to_le_bytes(), "{ memd(r2++#0x48:circ(m1)) = r5:4 }");
    test_invalid(&0b1010_1001111_00010_11_1_00100_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1011000_00010_11_0_00011_010_01000u32.to_le_bytes(), "{ memb(r2+#9) = r3 }");
    test_invalid(&0b1010_1011001_00010_11_0_00011_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1011010_00010_11_0_00011_010_01000u32.to_le_bytes(), "{ memh(r2+#18) = r3 }");
    test_display(&0b1010_1011011_00010_11_0_00011_010_01000u32.to_le_bytes(), "{ memh(r2+#18) = r3.h }");
    test_display(&0b1010_1011100_00010_11_0_00011_010_01000u32.to_le_bytes(), "{ memw(r2+#36) = r3 }");
    test_display(&0b1010_1011101_00010_11_0_00011_010_01000u32.to_le_bytes(), "{ memb(r2+#9) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_01011_010_01000u32.to_le_bytes(), "{ memh(r2+#18) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_10011_010_01000u32.to_le_bytes(), "{ memw(r2+#36) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_10011_010_01000u32.to_le_bytes(), "{ memw(r2+#36) = r3.new }");
    test_invalid(&0b1010_1011101_00010_11_0_11011_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1011110_00010_11_0_00100_010_01000u32.to_le_bytes(), "{ memd(r2+#72) = r5:4 }");
    test_invalid(&0b1010_1011111_00010_11_0_00100_010_01000u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1011000_00010_11_0_00011_101_01001u32.to_le_bytes(), "{ memb(r2=#0x29) = r3 }");
    test_invalid(&0b1010_1011001_00010_11_0_00011_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1011010_00010_11_0_00011_101_01001u32.to_le_bytes(), "{ memh(r2=#0x29) = r3 }");
    test_display(&0b1010_1011011_00010_11_0_00011_101_01001u32.to_le_bytes(), "{ memh(r2=#0x29) = r3.h }");
    test_display(&0b1010_1011100_00010_11_0_00011_101_01001u32.to_le_bytes(), "{ memw(r2=#0x29) = r3 }");
    test_display(&0b1010_1011101_00010_11_0_00011_101_01001u32.to_le_bytes(), "{ memb(r2=#0x29) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_01011_101_01001u32.to_le_bytes(), "{ memh(r2=#0x29) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_10011_101_01001u32.to_le_bytes(), "{ memw(r2=#0x29) = r3.new }");
    test_display(&0b1010_1011101_00010_11_0_10011_101_01001u32.to_le_bytes(), "{ memw(r2=#0x29) = r3.new }");
    test_invalid(&0b1010_1011101_00010_11_0_11011_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1011110_00010_11_0_00100_101_01001u32.to_le_bytes(), "{ memd(r2=#0x29) = r5:4 }");
    test_invalid(&0b1010_1011111_00010_11_0_00100_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1011000_00010_11_1_00100_010_01011u32.to_le_bytes(), "{ if (p3) memb(r2+#9) = r4 }");
    test_display(&0b1010_1011000_00010_11_1_00100_010_01111u32.to_le_bytes(), "{ if (!p3) memb(r2+#9) = r4 }");
    test_display(&0b1010_1011010_00010_11_1_00100_010_01011u32.to_le_bytes(), "{ if (p3) memh(r2+#18) = r4 }");
    test_display(&0b1010_1011010_00010_11_1_00100_010_01111u32.to_le_bytes(), "{ if (!p3) memh(r2+#18) = r4 }");
    test_display(&0b1010_1011011_00010_11_1_00100_010_01011u32.to_le_bytes(), "{ if (p3) memh(r2+#18) = r4.h }");
    test_display(&0b1010_1011011_00010_11_1_00100_010_01111u32.to_le_bytes(), "{ if (!p3) memh(r2+#18) = r4.h }");
    test_display(&0b1010_1011100_00010_11_1_00100_010_01011u32.to_le_bytes(), "{ if (p3) memw(r2+#36) = r4 }");
    test_display(&0b1010_1011100_00010_11_1_00100_010_01111u32.to_le_bytes(), "{ if (!p3) memw(r2+#36) = r4 }");

    test_display(&0b1010_1011000_00010_11_1_00100_110_01011u32.to_le_bytes(), "{ if (p3.new) memb(r2+#9) = r4 }");
    test_display(&0b1010_1011000_00010_11_1_00100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memb(r2+#9) = r4 }");
    test_display(&0b1010_1011010_00010_11_1_00100_110_01011u32.to_le_bytes(), "{ if (p3.new) memh(r2+#18) = r4 }");
    test_display(&0b1010_1011010_00010_11_1_00100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memh(r2+#18) = r4 }");
    test_display(&0b1010_1011011_00010_11_1_00100_110_01011u32.to_le_bytes(), "{ if (p3.new) memh(r2+#18) = r4.h }");
    test_display(&0b1010_1011011_00010_11_1_00100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memh(r2+#18) = r4.h }");
    test_display(&0b1010_1011100_00010_11_1_00100_110_01011u32.to_le_bytes(), "{ if (p3.new) memw(r2+#36) = r4 }");
    test_display(&0b1010_1011100_00010_11_1_00100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memw(r2+#36) = r4 }");

    test_display(&0b1010_1011101_00010_11_1_00100_010_01011u32.to_le_bytes(), "{ if (p3) memb(r2+#9) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_00100_010_01111u32.to_le_bytes(), "{ if (!p3) memb(r2+#9) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_00100_110_01011u32.to_le_bytes(), "{ if (p3.new) memb(r2+#9) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_00100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memb(r2+#9) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_01100_010_01011u32.to_le_bytes(), "{ if (p3) memh(r2+#18) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_01100_010_01111u32.to_le_bytes(), "{ if (!p3) memh(r2+#18) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_01100_110_01011u32.to_le_bytes(), "{ if (p3.new) memh(r2+#18) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_01100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memh(r2+#18) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_10100_010_01011u32.to_le_bytes(), "{ if (p3) memw(r2+#36) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_10100_010_01111u32.to_le_bytes(), "{ if (!p3) memw(r2+#36) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_10100_110_01011u32.to_le_bytes(), "{ if (p3.new) memw(r2+#36) = r4.new }");
    test_display(&0b1010_1011101_00010_11_1_10100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memw(r2+#36) = r4.new }");
    test_invalid(&0b1010_1011101_00010_11_1_11100_010_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1011101_00010_11_1_11100_010_01111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1011101_00010_11_1_11100_110_01011u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1011101_00010_11_1_11100_110_01111u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1011110_00010_11_1_10100_010_01011u32.to_le_bytes(), "{ if (p3) memd(r2+#72) = r21:20 }");
    test_display(&0b1010_1011110_00010_11_1_10100_010_01111u32.to_le_bytes(), "{ if (!p3) memd(r2+#72) = r21:20 }");
    test_display(&0b1010_1011110_00010_11_1_10100_110_01011u32.to_le_bytes(), "{ if (p3.new) memd(r2+#72) = r21:20 }");
    test_display(&0b1010_1011110_00010_11_1_10100_110_01111u32.to_le_bytes(), "{ if (!p3.new) memd(r2+#72) = r21:20 }");

    test_display(&0b1010_1101000_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memb(r2++m1) = r6 }");
    test_invalid(&0b1010_1101001_00010_11_1_00110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1101010_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memh(r2++m1) = r6 }");
    test_display(&0b1010_1101011_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memh(r2++m1) = r6.h }");
    test_display(&0b1010_1101100_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memw(r2++m1) = r6 }");
    test_display(&0b1010_1101101_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memb(r2++m1) = r6.new }");
    test_display(&0b1010_1101101_00010_11_1_01110_001_01001u32.to_le_bytes(), "{ memh(r2++m1) = r6.new }");
    test_display(&0b1010_1101101_00010_11_1_10110_001_01001u32.to_le_bytes(), "{ memw(r2++m1) = r6.new }");
    test_invalid(&0b1010_1101101_00010_11_1_11110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1101110_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memd(r2++m1) = r7:6 }");
    test_invalid(&0b1010_1101111_00010_11_1_11110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1101000_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memb(r2<<2 + 0x29) = r6 }");
    test_invalid(&0b1010_1101001_00010_11_1_00110_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1101010_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memh(r2<<2 + 0x29) = r6 }");
    test_display(&0b1010_1101011_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memh(r2<<2 + 0x29) = r6.h }");
    test_display(&0b1010_1101100_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memw(r2<<2 + 0x29) = r6 }");
    test_display(&0b1010_1101101_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memb(r2<<2 + 0x29) = r6.new }");
    test_display(&0b1010_1101101_00010_11_1_01110_101_01001u32.to_le_bytes(), "{ memh(r2<<2 + 0x29) = r6.new }");
    test_display(&0b1010_1101101_00010_11_1_10110_101_01001u32.to_le_bytes(), "{ memw(r2<<2 + 0x29) = r6.new }");
    test_invalid(&0b1010_1101101_00010_11_1_11110_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1101110_00010_11_1_00110_101_01001u32.to_le_bytes(), "{ memd(r2<<2 + 0x29) = r7:6 }");
    test_invalid(&0b1010_1101111_00010_11_1_11110_101_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1111000_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memb(r2++m1:brev) = r6 }");
    test_invalid(&0b1010_1111001_00010_11_1_00110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1111010_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memh(r2++m1:brev) = r6 }");
    test_display(&0b1010_1111011_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memh(r2++m1:brev) = r6.h }");
    test_display(&0b1010_1111100_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memw(r2++m1:brev) = r6 }");
    test_display(&0b1010_1111101_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memb(r2++m1:brev) = r6.new }");
    test_display(&0b1010_1111101_00010_11_1_01110_001_01001u32.to_le_bytes(), "{ memh(r2++m1:brev) = r6.new }");
    test_display(&0b1010_1111101_00010_11_1_10110_001_01001u32.to_le_bytes(), "{ memw(r2++m1:brev) = r6.new }");
    test_invalid(&0b1010_1111101_00010_11_1_11110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1111110_00010_11_1_00110_001_01001u32.to_le_bytes(), "{ memd(r2++m1:brev) = r7:6 }");
    test_invalid(&0b1010_1111111_00010_11_1_11110_001_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1111000_00010_11_0_00110_110_01001u32.to_le_bytes(), "{ if (p1) memb(#0x29) = r6 }");
    test_display(&0b1010_1111000_00010_11_0_00110_110_01101u32.to_le_bytes(), "{ if (!p1) memb(#0x29) = r6 }");
    test_display(&0b1010_1111000_00010_11_1_00110_110_01001u32.to_le_bytes(), "{ if (p1.new) memb(#0x29) = r6 }");
    test_display(&0b1010_1111000_00010_11_1_00110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memb(#0x29) = r6 }");
    test_invalid(&0b1010_1111001_00010_11_0_00110_110_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1111001_00010_11_0_00110_110_01101u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1111001_00010_11_1_00110_110_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1111001_00010_11_1_00110_110_01101u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1010_1111010_00010_11_0_00110_110_01001u32.to_le_bytes(), "{ if (p1) memh(#0x29) = r6 }");
    test_display(&0b1010_1111010_00010_11_0_00110_110_01101u32.to_le_bytes(), "{ if (!p1) memh(#0x29) = r6 }");
    test_display(&0b1010_1111010_00010_11_1_00110_110_01001u32.to_le_bytes(), "{ if (p1.new) memh(#0x29) = r6 }");
    test_display(&0b1010_1111010_00010_11_1_00110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memh(#0x29) = r6 }");
    test_display(&0b1010_1111011_00010_11_0_00110_110_01001u32.to_le_bytes(), "{ if (p1) memh(#0x29) = r6.h }");
    test_display(&0b1010_1111011_00010_11_0_00110_110_01101u32.to_le_bytes(), "{ if (!p1) memh(#0x29) = r6.h }");
    test_display(&0b1010_1111011_00010_11_1_00110_110_01001u32.to_le_bytes(), "{ if (p1.new) memh(#0x29) = r6.h }");
    test_display(&0b1010_1111011_00010_11_1_00110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memh(#0x29) = r6.h }");
    test_display(&0b1010_1111100_00010_11_0_00110_110_01001u32.to_le_bytes(), "{ if (p1) memw(#0x29) = r6 }");
    test_display(&0b1010_1111100_00010_11_0_00110_110_01101u32.to_le_bytes(), "{ if (!p1) memw(#0x29) = r6 }");
    test_display(&0b1010_1111100_00010_11_1_00110_110_01001u32.to_le_bytes(), "{ if (p1.new) memw(#0x29) = r6 }");
    test_display(&0b1010_1111100_00010_11_1_00110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memw(#0x29) = r6 }");

    test_display(&0b1010_1111101_00010_11_000_110_110_01001u32.to_le_bytes(), "{ if (p1) memb(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_000_110_110_01101u32.to_le_bytes(), "{ if (!p1) memb(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_001_110_110_01001u32.to_le_bytes(), "{ if (p1) memh(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_001_110_110_01101u32.to_le_bytes(), "{ if (!p1) memh(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_010_110_110_01001u32.to_le_bytes(), "{ if (p1) memw(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_010_110_110_01101u32.to_le_bytes(), "{ if (!p1) memw(#0x29) = r6.new }");
    test_invalid(&0b1010_1111101_00010_11_011_110_110_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1111101_00010_11_011_110_110_01101u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1111101_00010_11_100_110_110_01001u32.to_le_bytes(), "{ if (p1.new) memb(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_100_110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memb(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_101_110_110_01001u32.to_le_bytes(), "{ if (p1.new) memh(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_101_110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memh(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_110_110_110_01001u32.to_le_bytes(), "{ if (p1.new) memw(#0x29) = r6.new }");
    test_display(&0b1010_1111101_00010_11_110_110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memw(#0x29) = r6.new }");
    test_invalid(&0b1010_1111101_00010_11_111_110_110_01001u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1010_1111101_00010_11_111_110_110_01101u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1010_1111110_00010_11_0_00110_110_01001u32.to_le_bytes(), "{ if (p1) memd(#0x29) = r7:6 }");
    test_display(&0b1010_1111110_00010_11_0_00110_110_01101u32.to_le_bytes(), "{ if (!p1) memd(#0x29) = r7:6 }");
    test_display(&0b1010_1111110_00010_11_1_00110_110_01001u32.to_le_bytes(), "{ if (p1.new) memd(#0x29) = r7:6 }");
    test_display(&0b1010_1111110_00010_11_1_00110_110_01101u32.to_le_bytes(), "{ if (!p1.new) memd(#0x29) = r7:6 }");
}


#[test]
fn inst_1011() {
    test_display(&0b1011_1000001_00100_11_1_0_0000_001_10110u32.to_le_bytes(), "{ r22 = add(r4, #-31999) }");
}

#[test]
fn inst_1100() {
    test_display(&0b1100_0000_000_00100_11_0_1_0000_010_10110u32.to_le_bytes(), "{ r23:22 = valignb(r17:16, r5:4, #0x2) }");
    test_display(&0b1100_0000_100_00100_11_0_1_0000_010_10110u32.to_le_bytes(), "{ r23:22 = vspliceb(r5:4, r17:16, #0x2) }");
    test_display(&0b1100_0001_000_00100_11_0_1_0000_000_10110u32.to_le_bytes(), "{ r23:22 = extractu(r5:4, r17:16) }");
    test_display(&0b1100_0001_000_00100_11_0_1_0000_010_10110u32.to_le_bytes(), "{ r23:22 = shuffeb(r5:4, r17:16) }");
    test_display(&0b1100_0001_000_00100_11_0_1_0000_100_10110u32.to_le_bytes(), "{ r23:22 = shuffob(r17:16, r5:4) }");
    test_display(&0b1100_0001_000_00100_11_0_1_0000_110_10110u32.to_le_bytes(), "{ r23:22 = shuffeh(r17:16, r5:4) }");

    test_display(&0b1100_0001_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vxaddsubw(r21:20, r7:6):sat }");
    test_display(&0b1100_0001_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = vaddhub(r21:20, r7:6):sat }");
    test_display(&0b1100_0001_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vxsubaddw(r21:20, r7:6):sat }");
    test_invalid(&0b1100_0001_010_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0001_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vxaddsubh(r21:20, r7:6):sat }");
    test_invalid(&0b1100_0001_010_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0001_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vxsubaddh(r21:20, r7:6):sat }");
    test_invalid(&0b1100_0001_010_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1100_0001_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = shuffoh(r7:6, r21:20) }");
    test_invalid(&0b1100_0001_100_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0001_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vtrunewh(r21:20, r7:6) }");
    test_display(&0b1100_0001_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = vtrunehb(r21:20, r7:6) }");
    test_display(&0b1100_0001_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vtrunowh(r21:20, r7:6) }");
    test_display(&0b1100_0001_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vtrunohb(r21:20, r7:6) }");
    test_display(&0b1100_0001_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = lfs(r21:20, r7:6) }");
    test_invalid(&0b1100_0001_100_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1100_0001_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vxaddsubh(r21:20, r7:6):rnd:>>1:sat }");
    test_display(&0b1100_0001_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vxsubaddh(r21:20, r7:6):rnd:>>1:sat }");
    test_display(&0b1100_0001_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = extractu(r21:20, r7:6) }");
    test_display(&0b1100_0001_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = decbin(r21:20, r7:6) }");

    test_display(&0b1100_0010_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = valignb(r7:6, r21:20, p1) }");
    test_display(&0b1100_0010_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = valignb(r7:6, r21:20, p1) }");
    test_display(&0b1100_0010_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = valignb(r7:6, r21:20, p1) }");
    test_display(&0b1100_0010_011_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = valignb(r7:6, r21:20, p1) }");
    test_display(&0b1100_0010_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vspliceb(r21:20, r7:6, p1) }");
    test_invalid(&0b1100_0010_101_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0010_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = add(r21:20, r7:6, p1):carry }");
    test_display(&0b1100_0010_111_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = sub(r21:20, r7:6, p1):carry }");

    test_display(&0b1100_0011_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vasrw(r21:20, r6) }");
    test_display(&0b1100_0011_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vlsrw(r21:20, r6) }");
    test_display(&0b1100_0011_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vaslw(r21:20, r6) }");
    test_display(&0b1100_0011_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vlslw(r21:20, r6) }");
    test_display(&0b1100_0011_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vasrh(r21:20, r6) }");
    test_display(&0b1100_0011_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vlsrh(r21:20, r6) }");
    test_display(&0b1100_0011_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vaslh(r21:20, r6) }");
    test_display(&0b1100_0011_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vlslh(r21:20, r6) }");
    test_display(&0b1100_0011_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = asr(r21:20, r6) }");
    test_display(&0b1100_0011_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = lsr(r21:20, r6) }");
    test_display(&0b1100_0011_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = asl(r21:20, r6) }");
    test_display(&0b1100_0011_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = lsl(r21:20, r6) }");

    test_display(&0b1100_0011_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vcrotate(r21:20, r6) }");
    test_display(&0b1100_0011_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vcnegh(r21:20, r6) }");
    test_invalid(&0b1100_0011_110_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0011_110_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vrcrotate(r21:20, r6, #0x2) }");

    test_display(&0b1100_0100_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = addasl(r6, r20, #0x3) }");
    test_invalid(&0b1100_0100_001_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0100_011_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0100_101_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0100_111_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_invalid(&0b1100_0101_000_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0101_000_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0101_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = vasrw(r20, r6) }");
    test_invalid(&0b1100_0101_000_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0101_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = cmpyiwh(r21:20, r6):<<1:rnd:sat }");
    test_display(&0b1100_0101_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 = cmpyiwh(r21:20, r6*):<<1:rnd:sat }");
    test_display(&0b1100_0101_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = cmpyrwh(r21:20, r6):<<1:rnd:sat }");
    test_display(&0b1100_0101_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = cmpyrwh(r21:20, r6*):<<1:rnd:sat }");

    test_display(&0b1100_0110_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = asr(r20, r6):sat }");
    test_invalid(&0b1100_0110_000_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0110_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = asl(r20, r6):sat }");
    test_invalid(&0b1100_0110_000_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0110_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = asr(r20, r6) }");
    test_display(&0b1100_0110_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = lsr(r20, r6) }");
    test_display(&0b1100_0110_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = asl(r20, r6) }");
    test_display(&0b1100_0110_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = lsl(r20, r6) }");

    test_invalid(&0b1100_0110_100_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0110_100_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0110_100_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0110_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = lsl(#-24, r6) }");
    test_display(&0b1100_0110_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = lsl(#-23, r6) }");

    test_display(&0b1100_0110_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cround(r20, r6) }");
    test_display(&0b1100_0110_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cround(r21:20, r6) }");
    test_display(&0b1100_0110_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = round(r20, r6) }");
    test_display(&0b1100_0110_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = round(r20, r6):sat }");

    test_display(&0b1100_0111_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = tstbit(r20, r6) }");
    test_display(&0b1100_0111_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = !tstbit(r20, r6) }");
    test_display(&0b1100_0111_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = bitsset(r20, r6) }");
    test_display(&0b1100_0111_011_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = !bitsset(r20, r6) }");
    test_display(&0b1100_0111_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = bitsclr(r20, r6) }");
    test_display(&0b1100_0111_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = !bitsclr(r20, r6) }");
    test_invalid(&0b1100_0111_110_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0111_110_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0111_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ p2 = cmpb.gt(r20, r6) }");
    test_display(&0b1100_0111_110_10100_11_000110_011_10110u32.to_le_bytes(), "{ p2 = cmph.gt(r20, r6) }");
    test_display(&0b1100_0111_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ p2 = cmph.eq(r20, r6) }");
    test_display(&0b1100_0111_110_10100_11_000110_101_10110u32.to_le_bytes(), "{ p2 = cmph.gtu(r20, r6) }");
    test_display(&0b1100_0111_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = cmpb.eq(r20, r6) }");
    test_display(&0b1100_0111_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ p2 = cmpb.gtu(r20, r6) }");
    test_display(&0b1100_0111_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ p2 = sfcmp.ge(r20, r6) }");
    test_display(&0b1100_0111_111_10100_11_000110_001_10110u32.to_le_bytes(), "{ p2 = sfcmp.uo(r20, r6) }");
    test_invalid(&0b1100_0111_111_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_0111_111_10100_11_000110_011_10110u32.to_le_bytes(), "{ p2 = sfcmp.eq(r20, r6) }");
    test_display(&0b1100_0111_111_10100_11_000110_100_10110u32.to_le_bytes(), "{ p2 = sfcmp.gt(r20, r6) }");
    test_invalid(&0b1100_0111_111_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0111_111_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_0111_111_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1100_1000_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = insert(r20, r7:6) }");

    test_invalid(&0b1100_1001_000_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1001_010_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1001_100_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_1001_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = extractu(r20, r7:6) }");
    test_display(&0b1100_1001_111_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = extract(r20, r7:6) }");
    test_invalid(&0b1100_1001_111_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1001_111_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1100_1010_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = insert(r21:20, r7:6) }");
    test_invalid(&0b1100_1010_000_10100_11_100110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1100_1010_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 ^= xor(r21:20, r7:6) }");
    test_invalid(&0b1100_1010_100_10100_11_100110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1010_100_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1010_100_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1010_100_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1100_1010_110_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);


    test_display(&0b1100_1011_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 |= asr(r21:20, r6) }");
    test_display(&0b1100_1011_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 |= lsr(r21:20, r6) }");
    test_display(&0b1100_1011_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 |= asl(r21:20, r6) }");
    test_display(&0b1100_1011_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 |= lsl(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vrmaxh(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vrmaxw(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vrminh(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vrminw(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vrmaxuh(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vrmaxuw(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vrminuh(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vrminuw(r21:20, r6) }");
    test_display(&0b1100_1011_001_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 += vrcnegh(r21:20, r6) }");
    test_display(&0b1100_1011_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 &= asr(r21:20, r6) }");
    test_display(&0b1100_1011_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 &= lsr(r21:20, r6) }");
    test_display(&0b1100_1011_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 &= asl(r21:20, r6) }");
    test_display(&0b1100_1011_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 &= lsl(r21:20, r6) }");
    test_display(&0b1100_1011_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= asr(r21:20, r6) }");
    test_display(&0b1100_1011_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= lsr(r21:20, r6) }");
    test_display(&0b1100_1011_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 -= asl(r21:20, r6) }");
    test_display(&0b1100_1011_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 -= lsl(r21:20, r6) }");
    test_display(&0b1100_1011_101_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 += vrcrotate(r21:20, r6, #0x2) }");
    test_display(&0b1100_1011_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += asr(r21:20, r6) }");
    test_display(&0b1100_1011_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += lsr(r21:20, r6) }");
    test_display(&0b1100_1011_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += asl(r21:20, r6) }");
    test_display(&0b1100_1011_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += lsl(r21:20, r6) }");

    test_display(&0b1100_1100_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 |= asr(r20, r6) }");
    test_display(&0b1100_1100_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 |= lsr(r20, r6) }");
    test_display(&0b1100_1100_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 |= asl(r20, r6) }");
    test_display(&0b1100_1100_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 |= lsl(r20, r6) }");
    test_display(&0b1100_1100_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 &= asr(r20, r6) }");
    test_display(&0b1100_1100_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 &= lsr(r20, r6) }");
    test_display(&0b1100_1100_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 &= asl(r20, r6) }");
    test_display(&0b1100_1100_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 &= lsl(r20, r6) }");
    test_display(&0b1100_1100_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 -= asr(r20, r6) }");
    test_display(&0b1100_1100_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 -= lsr(r20, r6) }");
    test_display(&0b1100_1100_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 -= asl(r20, r6) }");
    test_display(&0b1100_1100_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 -= lsl(r20, r6) }");
    test_display(&0b1100_1100_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 += asr(r20, r6) }");
    test_display(&0b1100_1100_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 += lsr(r20, r6) }");
    test_display(&0b1100_1100_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += asl(r20, r6) }");
    test_display(&0b1100_1100_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 += lsl(r20, r6) }");
}

#[test]
fn inst_1101() {
    test_display(&0b1101_0000_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = parity(r21:20, r7:6) }");
    // bit 7 is `-` in the listing...
    test_display(&0b1101_0001_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vmux(p2, r21:20, r7:6) }");
    // bits 2, 3, 4 are `-` in the listing
    test_display(&0b1101_0010_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ p2 = vcmpw.eq(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ p2 = vcmpw.gt(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ p2 = vcmpw.gtu(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ p2 = vcmph.eq(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ p2 = vcmph.gt(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ p2 = vcmph.gtu(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ p2 = vcmpb.eq(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ p2 = vcmpb.gtu(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_100110_000_10110u32.to_le_bytes(), "{ p2 = any8(vcmpb.eq(r21:20, r7:6)) }");
    test_display(&0b1101_0010_000_10100_11_100110_001_10110u32.to_le_bytes(), "{ p2 = !any8(vcmpb.eq(r21:20, r7:6)) }");
    test_display(&0b1101_0010_000_10100_11_100110_010_10110u32.to_le_bytes(), "{ p2 = vcmpb.gt(r21:20, r7:6) }");
    test_display(&0b1101_0010_000_10100_11_100110_011_10110u32.to_le_bytes(), "{ p2 = tlbmatch(r21:20, r6) }");
    test_display(&0b1101_0010_000_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = boundscheck(r21:20, r7:6):raw:lo }");
    test_display(&0b1101_0010_000_10100_11_100110_101_10110u32.to_le_bytes(), "{ p2 = boundscheck(r21:20, r7:6):raw:hi }");

    test_display(&0b1101_0010_100_10100_11_100110_000_10110u32.to_le_bytes(), "{ p2 = cmp.gt(r21:20, r7:6) }");
    test_display(&0b1101_0010_100_10100_11_100110_010_10110u32.to_le_bytes(), "{ p2 = cmp.eq(r21:20, r7:6) }");
    test_display(&0b1101_0010_100_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = cmp.gtu(r21:20, r7:6) }");

    test_display(&0b1101_0011_000_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vaddub(r21:20, r7:6) }");
    test_display(&0b1101_0011_000_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vaddub(r21:20, r7:6):sat }");
    test_display(&0b1101_0011_000_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vaddh(r21:20, r7:6) }");
    test_display(&0b1101_0011_000_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vaddh(r21:20, r7:6):sat }");
    test_display(&0b1101_0011_000_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vadduh(r21:20, r7:6):sat }");
    test_display(&0b1101_0011_000_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vaddw(r21:20, r7:6) }");
    test_display(&0b1101_0011_000_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vaddw(r21:20, r7:6):sat }");
    test_display(&0b1101_0011_000_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = add(r21:20, r7:6) }");
    test_display(&0b1101_0011_001_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vsubub(r7:6, r21:20) }");
    test_display(&0b1101_0011_001_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vsubub(r7:6, r21:20):sat }");
    test_display(&0b1101_0011_001_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vsubh(r7:6, r21:20) }");
    test_display(&0b1101_0011_001_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vsubh(r7:6, r21:20):sat }");
    test_display(&0b1101_0011_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vsubuh(r7:6, r21:20):sat }");
    test_display(&0b1101_0011_001_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vsubw(r7:6, r21:20) }");
    test_display(&0b1101_0011_001_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vsubw(r7:6, r21:20):sat }");
    test_display(&0b1101_0011_001_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = sub(r7:6, r21:20) }");

    test_display(&0b1101_0011_010_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vavgub(r21:20, r7:6) }");
    test_display(&0b1101_0011_010_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vavgub(r21:20, r7:6):rnd }");
    test_display(&0b1101_0011_010_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vavgh(r21:20, r7:6) }");
    test_display(&0b1101_0011_010_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vavgh(r21:20, r7:6):rnd }");
    test_display(&0b1101_0011_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vavgh(r21:20, r7:6):crnd }");
    test_display(&0b1101_0011_010_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vavguh(r21:20, r7:6) }");
    test_display(&0b1101_0011_010_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vavguh(r21:20, r7:6):rnd }");

    test_display(&0b1101_0011_011_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vavgw(r21:20, r7:6) }");
    test_display(&0b1101_0011_011_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vavgw(r21:20, r7:6):rnd }");
    test_display(&0b1101_0011_011_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vavgw(r21:20, r7:6):crnd }");
    test_display(&0b1101_0011_011_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vavguw(r21:20, r7:6) }");
    test_display(&0b1101_0011_011_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vavguw(r21:20, r7:6):rnd }");
    test_display(&0b1101_0011_011_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = add(r21:20, r7:6):sat }");
    test_display(&0b1101_0011_011_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = add(r21:20, r7:6):raw:lo }");
    test_display(&0b1101_0011_011_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = add(r21:20, r7:6):raw:hi }");

    test_display(&0b1101_0011_100_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vnavgh(r7:6, r21:20) }");
    test_display(&0b1101_0011_100_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vnavgh(r7:6, r21:20):rnd:sat }");
    test_display(&0b1101_0011_100_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vnavgh(r7:6, r21:20):crnd:sat }");
    test_display(&0b1101_0011_100_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vnavgw(r7:6, r21:20) }");
    test_display(&0b1101_0011_100_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vnavgw(r7:6, r21:20):rnd:sat }");
    test_display(&0b1101_0011_100_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vnavgw(r7:6, r21:20):rnd:sat }");
    test_display(&0b1101_0011_100_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vnavgw(r7:6, r21:20):crnd:sat }");
    test_display(&0b1101_0011_100_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = vnavgw(r7:6, r21:20):crnd:sat }");

    test_display(&0b1101_0011_101_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vminub(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vminh(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vminuh(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vminw(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = vminuw(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = vmaxuw(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = min(r7:6, r21:20) }");
    test_display(&0b1101_0011_101_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = minu(r7:6, r21:20) }");

    test_display(&0b1101_0011_110_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = vmaxub(r7:6, r21:20) }");
    test_display(&0b1101_0011_110_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = vmaxh(r7:6, r21:20) }");
    test_display(&0b1101_0011_110_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = vmaxuh(r7:6, r21:20) }");
    test_display(&0b1101_0011_110_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = vmaxw(r7:6, r21:20) }");
    test_display(&0b1101_0011_110_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = max(r21:20, r7:6) }");
    test_display(&0b1101_0011_110_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = maxu(r21:20, r7:6) }");
    test_display(&0b1101_0011_110_10100_11_100110_110_10110u32.to_le_bytes(), "{ r23:22 = vmaxb(r7:6, r21:20) }");
    test_display(&0b1101_0011_110_10100_11_100110_111_10110u32.to_le_bytes(), "{ r23:22 = vminb(r7:6, r21:20) }");

    test_display(&0b1101_0011_111_10100_11_100110_000_10110u32.to_le_bytes(), "{ r23:22 = and(r21:20, r7:6) }");
    test_display(&0b1101_0011_111_10100_11_100110_001_10110u32.to_le_bytes(), "{ r23:22 = and(r21:20, ~r7:6) }");
    test_display(&0b1101_0011_111_10100_11_100110_010_10110u32.to_le_bytes(), "{ r23:22 = or(r21:20, r7:6) }");
    test_display(&0b1101_0011_111_10100_11_100110_011_10110u32.to_le_bytes(), "{ r23:22 = or(r21:20, ~r7:6) }");
    test_display(&0b1101_0011_111_10100_11_100110_100_10110u32.to_le_bytes(), "{ r23:22 = xor(r21:20, r7:6) }");
    test_invalid(&0b1101_0011_111_10100_11_100110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1101_0011_111_10100_11_100110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1101_0011_111_10100_11_100110_111_10110u32.to_le_bytes(), "{ r22 = modwrap(r20, r6) }");

    test_invalid(&0b1101_0100_000_10100_11_100110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1101_0100_001_10100_11_100110_101_10110u32.to_le_bytes(), "{ r23:22 = bitsplit(r20, r6) }");

    test_display(&0b1101_0101_000_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.l) }");
    test_display(&0b1101_0101_000_10100_11_100110_010_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.h) }");
    test_display(&0b1101_0101_000_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.l):sat }");
    test_display(&0b1101_0101_000_10100_11_100110_110_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.h):sat }");
    test_display(&0b1101_0101_001_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = sub(r6.l, r20.l) }");
    test_display(&0b1101_0101_001_10100_11_100110_010_10110u32.to_le_bytes(), "{ r22 = sub(r6.l, r20.h) }");
    test_display(&0b1101_0101_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = sub(r6.l, r20.l):sat }");
    test_display(&0b1101_0101_001_10100_11_100110_110_10110u32.to_le_bytes(), "{ r22 = sub(r6.l, r20.h):sat }");
    test_display(&0b1101_0101_010_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.l):<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_001_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.h):<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_010_10110u32.to_le_bytes(), "{ r22 = add(r6.h, r20.l):<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_011_10110u32.to_le_bytes(), "{ r22 = add(r6.h, r20.h):<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.l):sat:<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_101_10110u32.to_le_bytes(), "{ r22 = add(r6.l, r20.h):sat:<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_110_10110u32.to_le_bytes(), "{ r22 = add(r6.h, r20.l):sat:<<16 }");
    test_display(&0b1101_0101_010_10100_11_100110_111_10110u32.to_le_bytes(), "{ r22 = add(r6.h, r20.h):sat:<<16 }");

    test_display(&0b1101_0101_100_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = add(r20, r6):sat:deprecated }");
    test_display(&0b1101_0101_100_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = sub(r6, r20):sat:deprecated }");
    test_display(&0b1101_0101_101_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = min(r20, r6) }");
    test_display(&0b1101_0101_101_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = minu(r20, r6) }");
    test_display(&0b1101_0101_110_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = max(r20, r6) }");
    test_display(&0b1101_0101_110_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = maxu(r20, r6) }");
    test_display(&0b1101_0101_111_10100_11_100110_000_10110u32.to_le_bytes(), "{ r22 = parity(r20, r6) }");
    test_display(&0b1101_0101_111_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = parity(r20, r6) }");

    test_display(&0b1101_0110_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = sfmake(#0x334):pos }");
    test_display(&0b1101_0110_011_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = sfmake(#0x334):neg }");
    test_display(&0b1101_0110_111_10100_11_100110_000_10110u32.to_le_bytes(), "{ p2 = dfcmp.eq(r21:20, r7:6) }");
    test_display(&0b1101_0110_111_10100_11_100110_001_10110u32.to_le_bytes(), "{ p2 = dfcmp.gt(r21:20, r7:6) }");
    test_display(&0b1101_0110_111_10100_11_100110_011_10110u32.to_le_bytes(), "{ p2 = dfcmp.ge(r21:20, r7:6) }");
    test_display(&0b1101_0110_111_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = dfcmp.uo(r21:20, r7:6) }");

    test_display(&0b1101_0111_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = add(#0x2c, mpyi(r20, r6)) }");
    test_invalid(&0b1101_0111_110_10100_11_100110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1101_1000_110_10100_11_100110_100_10110u32.to_le_bytes(), "{ r6 = add(#0x2c, mpyi(r20, #0x36)) }");

    test_display(&0b1101_1001_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = dfmake(#0x334):pos }");
    test_display(&0b1101_1001_011_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 = dfmake(#0x334):neg }");
    test_invalid(&0b1101_1001_101_10100_11_100110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1101_1001_111_10100_11_100110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1101_1010_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 |= and(r20, #-204) }");
    test_display(&0b1101_1010_011_10100_11_100110_100_10110u32.to_le_bytes(), "{ r20 = or(r22, and(r20, #-204)) }");
    test_display(&0b1101_1010_101_10100_11_100110_100_10110u32.to_le_bytes(), "{ r22 |= or(r20, #-204) }");
    test_invalid(&0b1101_1010_111_10100_11_100110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1101_1011_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ r6 = add(r20, add(r22, #28)) }");
    test_display(&0b1101_1011_101_10100_11_100110_100_10110u32.to_le_bytes(), "{ r6 = add(r20, sub(#28, r22)) }");

    test_display(&0b1101_1100_000_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = vcmpb.eq(r21:20, #0x34) }");
    test_display(&0b1101_1100_000_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = vcmph.eq(r21:20, #0x34) }");
    test_display(&0b1101_1100_000_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = vcmpw.eq(r21:20, #0x34) }");
    test_invalid(&0b1101_1100_000_10100_11_100110_100_11110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1101_1100_001_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = vcmpb.gt(r21:20, #0x34) }");
    test_display(&0b1101_1100_001_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = vcmph.gt(r21:20, #0x34) }");
    test_display(&0b1101_1100_001_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = vcmpw.gt(r21:20, #0x34) }");
    test_invalid(&0b1101_1100_001_10100_11_100110_100_11110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1101_1100_010_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = vcmpb.gtu(r21:20, #0x34) }");
    test_display(&0b1101_1100_010_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = vcmph.gtu(r21:20, #0x34) }");
    test_display(&0b1101_1100_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ p2 = vcmpw.gtu(r21:20, #0x34) }");
    test_invalid(&0b1101_1100_010_10100_11_100110_100_11110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    // top bit of the immediate must be 0
    test_invalid(&0b1101_1100_010_10100_11_110110_100_00110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1101_1100_010_10100_11_110110_100_01110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1101_1100_010_10100_11_110110_100_10110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1101_1100_010_10100_11_110110_100_11110u32.to_le_bytes(), DecodeError::InvalidOperand);

    test_display(&0b1101_1100_100_10100_11_100010_100_10110u32.to_le_bytes(), "{ p2 = dfclass(r21:20, #0x14) }");

    test_display(&0b1101_1101_000_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = cmpb.gt(r20, #0x34) }");
    test_display(&0b1101_1101_000_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = cmph.gt(r20, #0x34) }");
    test_display(&0b1101_1101_001_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = cmpb.eq(r20, #0x34) }");
    test_display(&0b1101_1101_001_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = cmph.eq(r20, #0x34) }");
    test_display(&0b1101_1101_010_10100_11_100110_100_00110u32.to_le_bytes(), "{ p2 = cmpb.gtu(r20, #0x34) }");
    test_display(&0b1101_1101_010_10100_11_100110_100_01110u32.to_le_bytes(), "{ p2 = cmph.gtu(r20, #0x34) }");
    test_invalid(&0b1101_1101_010_10100_11_110110_100_00110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1101_1101_010_10100_11_110110_100_01110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_invalid(&0b1101_1101_011_10100_11_110110_100_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1101_1101_011_10100_11_110110_100_01110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1101_1110_010_10100_11_100110_100_00000u32.to_le_bytes(), "{ r20 = and(#0x98, asl(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_00010u32.to_le_bytes(), "{ r20 = or(#0x98, asl(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_00100u32.to_le_bytes(), "{ r20 = add(#0x98, asl(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_00110u32.to_le_bytes(), "{ r20 = sub(#0x98, asl(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_10000u32.to_le_bytes(), "{ r20 = and(#0x98, lsr(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_10010u32.to_le_bytes(), "{ r20 = or(#0x98, lsr(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_10100u32.to_le_bytes(), "{ r20 = add(#0x98, lsr(r20, #0x6)) }");
    test_display(&0b1101_1110_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ r20 = sub(#0x98, lsr(r20, #0x6)) }");

    test_display(&0b1101_1111_010_10100_11_100110_100_10110u32.to_le_bytes(), "{ r6 = add(r22, mpyi(#0xb0, r20)) }");
    test_display(&0b1101_1111_110_10100_11_100110_100_10110u32.to_le_bytes(), "{ r6 = add(r22, mpyi(r20, #0xb0)) }");
}

#[test]
fn inst_1110() {
    test_display(&0b1110_0000_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = +mpyi(r20, #0x34) }");
    test_display(&0b1110_0000_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = -mpyi(r20, #0x34) }");
    test_display(&0b1110_0001_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += mpyi(r20, #0x34) }");
    test_display(&0b1110_0001_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 -= mpyi(r20, #0x34) }");
    test_display(&0b1110_0010_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += add(r20, #52) }");
    test_display(&0b1110_0010_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 -= add(r20, #52) }");
    test_display(&0b1110_0011_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r6 = add(r22, mpyi(r6, r20)) }");

    test_display(&0b1110_0100_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.l) }");
    test_display(&0b1110_0100_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0100_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.h) }");
    test_display(&0b1110_0100_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0100_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.l) }");
    test_display(&0b1110_0100_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0100_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.h) }");
    test_display(&0b1110_0100_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.h):<<1 }");

    test_display(&0b1110_0100_001_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.l):rnd }");
    test_display(&0b1110_0100_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.l):<<1:rnd }");
    test_display(&0b1110_0100_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.h):rnd }");
    test_display(&0b1110_0100_101_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.l, r6.h):<<1:rnd }");
    test_display(&0b1110_0100_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.l):rnd }");
    test_display(&0b1110_0100_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.l):<<1:rnd }");
    test_display(&0b1110_0100_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.h):rnd }");
    test_display(&0b1110_0100_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20.h, r6.h):<<1:rnd }");

    test_display(&0b1110_0100_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.l, r6.l) }");
    test_display(&0b1110_0100_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0100_010_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.l, r6.h) }");
    test_display(&0b1110_0100_110_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0100_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.h, r6.l) }");
    test_display(&0b1110_0100_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0100_010_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.h, r6.h) }");
    test_display(&0b1110_0100_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20.h, r6.h):<<1 }");

    test_invalid(&0b1110_0100_011_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_111_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_011_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_111_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_011_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_111_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_011_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_0100_111_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1110_0101_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = mpy(r20, r6) }");
    test_display(&0b1110_0101_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = cmpyi(r20, r6) }");
    test_display(&0b1110_0101_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cmpyr(r20, r6) }");
    test_invalid(&0b1110_0101_000_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1110_0101_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = mpyu(r20, r6) }");
    test_display(&0b1110_0101_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vmpybsu(r20, r6) }");
    test_display(&0b1110_0101_010_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = pmpyw(r20, r6) }");

    test_display(&0b1110_0101_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vmpybu(r20, r6) }");
    test_display(&0b1110_0101_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vpmpyh(r20, r6) }");

    test_display(&0b1110_0101_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyh(r21:20, r7:6):sat }");
    test_display(&0b1110_0101_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = cmpy(r20, r6):sat }");
    test_display(&0b1110_0101_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpyhsu(r20, r6):sat }");
    test_display(&0b1110_0101_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = cmpy(r20, r6*):sat }");
    test_display(&0b1110_0101_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_0101_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = cmpy(r20, r6):<<1:sat }");
    test_display(&0b1110_0101_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpyhsu(r20, r6):<<1:sat }");
    test_display(&0b1110_0101_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = cmpy(r20, r6*):<<1:sat }");

    test_display(&0b1110_0110_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.l, r6.l) }");
    test_display(&0b1110_0110_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.l, r6.h) }");
    test_display(&0b1110_0110_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.h, r6.l) }");
    test_display(&0b1110_0110_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.h, r6.h) }");
    test_display(&0b1110_0110_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.l, r6.l) }");
    test_display(&0b1110_0110_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.l, r6.h) }");
    test_display(&0b1110_0110_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.h, r6.l) }");
    test_display(&0b1110_0110_001_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.h, r6.h) }");
    test_display(&0b1110_0110_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.l, r6.l) }");
    test_display(&0b1110_0110_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.l, r6.h) }");
    test_display(&0b1110_0110_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.h, r6.l) }");
    test_display(&0b1110_0110_010_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.h, r6.h) }");
    test_display(&0b1110_0110_011_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.l, r6.l) }");
    test_display(&0b1110_0110_011_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.l, r6.h) }");
    test_display(&0b1110_0110_011_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.h, r6.l) }");
    test_display(&0b1110_0110_011_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.h, r6.h) }");
    test_display(&0b1110_0110_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0110_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0110_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0110_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20.h, r6.h):<<1 }");
    test_display(&0b1110_0110_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0110_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0110_101_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0110_101_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20.h, r6.h):<<1 }");
    test_display(&0b1110_0110_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0110_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0110_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0110_110_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20.h, r6.h):<<1 }");
    test_display(&0b1110_0110_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.l, r6.l):<<1 }");
    test_display(&0b1110_0110_111_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.l, r6.h):<<1 }");
    test_display(&0b1110_0110_111_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.h, r6.l):<<1 }");
    test_display(&0b1110_0110_111_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20.h, r6.h):<<1 }");

    test_display(&0b1110_0111_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpy(r20, r6) }");
    test_display(&0b1110_0111_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpy(r20, r6) }");
    test_display(&0b1110_0111_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 += mpyu(r20, r6) }");
    test_display(&0b1110_0111_011_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 -= mpyu(r20, r6) }");
    test_display(&0b1110_0111_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += cmpyi(r20, r6) }");
    test_display(&0b1110_0111_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vmpyh(r21:20, r7:6) }");
    test_display(&0b1110_0111_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vmpybu(r21:20, r7:6) }");
    test_display(&0b1110_0111_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vmpybsu(r21:20, r7:6) }");
    test_display(&0b1110_0111_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += cmpyr(r20, r6) }");
    test_display(&0b1110_0111_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 ^= pmpyw(r20, r6) }");
    test_display(&0b1110_0111_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 ^= vpmpyh(r20, r6) }");

    test_display(&0b1110_0111_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyh(r21:20, r7:6):sat }");
    test_display(&0b1110_0111_011_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyhsu(r20, r6):sat }");
    test_display(&0b1110_0111_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += cmpy(r20, r6):sat }");
    test_display(&0b1110_0111_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += cmpy(r20, r6*):sat }");
    test_display(&0b1110_0111_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 -= cmpy(r20, r6):sat }");
    test_display(&0b1110_0111_010_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 -= cmpy(r20, r6*):sat }");
    test_display(&0b1110_0111_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_0111_111_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyhsu(r20, r6):<<1:sat }");
    test_display(&0b1110_0111_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += cmpy(r20, r6):<<1:sat }");
    test_display(&0b1110_0111_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += cmpy(r20, r6*):<<1:sat }");
    test_display(&0b1110_0111_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 -= cmpy(r20, r6):<<1:sat }");
    test_display(&0b1110_0111_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 -= cmpy(r20, r6*):<<1:sat }");

    test_display(&0b1110_1000_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vrmpyh(r21:20, r7:6) }");
    test_display(&0b1110_1000_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfadd(r21:20, r7:6) }");
    test_display(&0b1110_1000_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vabsdiffw(r7:6, r21:20) }");
    test_display(&0b1110_1000_001_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfmax(r21:20, r7:6) }");
    test_display(&0b1110_1000_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vraddub(r21:20, r7:6) }");
    test_display(&0b1110_1000_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vrsadub(r21:20, r7:6) }");
    test_display(&0b1110_1000_010_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfmpyfix(r21:20, r7:6) }");
    test_display(&0b1110_1000_011_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vabsdiffh(r7:6, r21:20) }");
    test_display(&0b1110_1000_011_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cmpyiw(r21:20, r7:6) }");
    test_display(&0b1110_1000_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vrmpyu(r21:20, r7:6) }");
    test_display(&0b1110_1000_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cmpyrw(r21:20, r7:6) }");
    test_display(&0b1110_1000_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfsub(r21:20, r7:6) }");
    test_display(&0b1110_1000_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vabsdiffub(r7:6, r21:20) }");
    test_display(&0b1110_1000_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vdmpybsu(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_101_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfmpyll(r21:20, r7:6) }");
    test_display(&0b1110_1000_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 = vrmpysu(r21:20, r7:6) }");
    test_display(&0b1110_1000_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cmpyrw(r21:20, r7:6*) }");
    test_display(&0b1110_1000_110_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 = dfmin(r21:20, r7:6) }");
    test_display(&0b1110_1000_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r23:22 = vabsdiffb(r7:6, r21:20) }");
    test_display(&0b1110_1000_111_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = cmpyiw(r21:20, r7:6*) }");
    test_display(&0b1110_1000_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vrcmpys(r21:20, r7:6):<<1:sat:raw:hi }");
    test_display(&0b1110_1000_111_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vrcmpys(r21:20, r7:6):<<1:sat:raw:lo }");

    test_display(&0b1110_1000_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vdmpy(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweh(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vmpyeh(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywoh(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vrmpywoh(r21:20, r7:6) }");
    test_display(&0b1110_1000_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1000_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vcmpyr(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywoh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1000_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vrmpywoh(r21:20, r7:6) }");
    test_display(&0b1110_1000_010_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweuh(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vcmpyi(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_010_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywouh(r21:20, r7:6):sat }");
    test_display(&0b1110_1000_011_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweuh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1000_011_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywouh(r21:20, r7:6):rnd:sat }");

    test_display(&0b1110_1000_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vdmpy(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vmpyeh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywoh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_101_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 = vrmpywoh(r21:20, r7:6):<<1 }");
    test_display(&0b1110_1000_101_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1000_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vcmpyr(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywoh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1000_110_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 = vrmpywoh(r21:20, r7:6):<<1 }");
    test_display(&0b1110_1000_110_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweuh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_110_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 = vcmpyi(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywouh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1000_111_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 = vmpyweuh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1000_111_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 = vmpywouh(r21:20, r7:6):<<1:rnd:sat }");

    test_display(&0b1110_1001_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = vradduh(r21:20, r7:6) }");
    test_display(&0b1110_1001_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = vraddh(r21:20, r7:6) }");
    test_display(&0b1110_1001_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = cmpyiw(r21:20, r7:6*):<<1:sat }");
    test_display(&0b1110_1001_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyiw(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1001_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyrw(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1001_011_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyrw(r21:20, r7:6*):<<1:sat }");
    test_display(&0b1110_1001_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = vrcmpys(r21:20, r7:6):<<1:rnd:sat:raw:hi }");
    test_display(&0b1110_1001_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = vrcmpys(r21:20, r7:6):<<1:rnd:sat:raw:lo }");
    test_display(&0b1110_1001_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = cmpyiw(r21:20, r7:6*):<<1:rnd:sat }");
    test_display(&0b1110_1001_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyiw(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1001_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyrw(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1001_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = cmpyrw(r21:20, r7:6*):<<1:rnd:sat }");

    test_display(&0b1110_1001_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = vdmpy(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1001_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = vdmpy(r21:20, r7:6):<<1:rnd:sat }");

    test_display(&0b1110_1010_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += vrmpyh(r21:20, r7:6) }");
    test_display(&0b1110_1010_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += dfmpylh(r21:20, r7:6) }");
    test_display(&0b1110_1010_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vdmpybsu(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += vmpyeh(r21:20, r7:6) }");
    test_display(&0b1110_1010_001_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vcmpyr(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vraddub(r21:20, r7:6) }");
    test_display(&0b1110_1010_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += vrsadub(r21:20, r7:6) }");
    test_display(&0b1110_1010_010_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vcmpyi(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_010_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += cmpyiw(r21:20, r7:6*) }");
    test_display(&0b1110_1010_011_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += cmpyiw(r21:20, r7:6) }");

    test_display(&0b1110_1010_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vrmpybu(r21:20, r7:6) }");
    test_display(&0b1110_1010_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += cmpyrw(r21:20, r7:6) }");
    test_display(&0b1110_1010_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r23:22 += dfmpyhh(r21:20, r7:6) }");
    test_display(&0b1110_1010_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22, p1 = vacsh(r21:20, r7:6) }");
    test_display(&0b1110_1010_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vrcmpys(r21:20, r7:6):<<1:sat:raw:hi }");

    test_display(&0b1110_1010_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22 += vrmpybsu(r21:20, r7:6) }");
    test_display(&0b1110_1010_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r23:22 += cmpyrw(r21:20, r7:6*) }");
    test_display(&0b1110_1010_111_10100_11_000110_001_10110u32.to_le_bytes(), "{ r23:22, p1 = vminub(r21:20, r7:6) }");
    test_display(&0b1110_1010_111_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vrcmpys(r21:20, r7:6):<<1:sat:raw:lo }");

    test_display(&0b1110_1010_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vdmpy(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweh(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vmpyeh(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywoh(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1010_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vrmpyweh(r21:20, r7:6) }");
    test_display(&0b1110_1010_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywoh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1010_010_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweuh(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_010_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywouh(r21:20, r7:6):sat }");
    test_display(&0b1110_1010_011_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweuh(r21:20, r7:6):rnd:sat }");
    test_display(&0b1110_1010_011_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vrmpywoh(r21:20, r7:6) }");
    test_display(&0b1110_1010_011_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywouh(r21:20, r7:6):rnd:sat }");

    test_display(&0b1110_1010_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r23:22 += vdmpy(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vmpyeh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywoh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_101_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1010_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vrmpyweh(r21:20, r7:6):<<1 }");
    test_display(&0b1110_1010_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywoh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1010_110_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweuh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_110_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywouh(r21:20, r7:6):<<1:sat }");
    test_display(&0b1110_1010_111_10100_11_000110_101_10110u32.to_le_bytes(), "{ r23:22 += vmpyweuh(r21:20, r7:6):<<1:rnd:sat }");
    test_display(&0b1110_1010_111_10100_11_000110_110_10110u32.to_le_bytes(), "{ r23:22 += vrmpywoh(r21:20, r7:6):<<1 }");
    test_display(&0b1110_1010_111_10100_11_000110_111_10110u32.to_le_bytes(), "{ r23:22 += vmpywouh(r21:20, r7:6):<<1:rnd:sat }");

    test_invalid(&0b1110_1011_000_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1110_1011_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = sfsub(r20, r6) }");
    test_display(&0b1110_1011_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = sfadd(r20, r6) }");
    test_display(&0b1110_1011_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = sfmpy(r20, r6) }");
    test_display(&0b1110_1011_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = sfmax(r20, r6) }");
    test_display(&0b1110_1011_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = sfmin(r20, r6) }");
    test_display(&0b1110_1011_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = sffixupn(r20, r6) }");
    test_display(&0b1110_1011_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = sffixupd(r20, r6) }");
    test_display(&0b1110_1011_111_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22, p1 = sfrecipa(r20, r6) }");

    test_display(&0b1110_1100_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l) }");
    test_display(&0b1110_1100_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h) }");
    test_display(&0b1110_1100_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l) }");
    test_display(&0b1110_1100_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h) }");
    test_display(&0b1110_1100_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):sat }");
    test_display(&0b1110_1100_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):sat }");
    test_display(&0b1110_1100_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):sat }");
    test_display(&0b1110_1100_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):sat }");
    test_display(&0b1110_1100_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):rnd }");
    test_display(&0b1110_1100_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):rnd }");
    test_display(&0b1110_1100_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):rnd }");
    test_display(&0b1110_1100_001_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):rnd }");
    test_display(&0b1110_1100_001_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):rnd:sat }");
    test_display(&0b1110_1100_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):rnd:sat }");
    test_display(&0b1110_1100_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):rnd:sat }");
    test_display(&0b1110_1100_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):rnd:sat }");
    test_display(&0b1110_1100_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_1100_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_1100_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_1100_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):<<1 }");
    test_display(&0b1110_1100_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):<<1:sat }");
    test_display(&0b1110_1100_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):<<1:sat }");
    test_display(&0b1110_1100_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):<<1:sat }");
    test_display(&0b1110_1100_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):<<1:sat }");
    test_display(&0b1110_1100_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):<<1:rnd }");
    test_display(&0b1110_1100_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):<<1:rnd }");
    test_display(&0b1110_1100_101_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):<<1:rnd }");
    test_display(&0b1110_1100_101_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):<<1:rnd }");
    test_display(&0b1110_1100_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.l):<<1:rnd:sat }");
    test_display(&0b1110_1100_101_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 = mpy(r20.l, r6.h):<<1:rnd:sat }");
    test_display(&0b1110_1100_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.l):<<1:rnd:sat }");
    test_display(&0b1110_1100_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = mpy(r20.h, r6.h):<<1:rnd:sat }");

    test_display(&0b1110_1101_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpyi(r20, r6) }");
    test_display(&0b1110_1101_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6.h):<<1:sat }");
    test_display(&0b1110_1101_111_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6):<<1:sat }");
    test_display(&0b1110_1101_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6) }");
    test_display(&0b1110_1101_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6):rnd }");
    test_display(&0b1110_1101_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpyu(r20, r6) }");
    test_display(&0b1110_1101_011_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpysu(r20, r6) }");
    test_display(&0b1110_1101_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6.l):<<1:sat }");
    test_display(&0b1110_1101_101_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6):<<1 }");
    test_display(&0b1110_1101_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6.h):<<1:rnd:sat }");
    test_display(&0b1110_1101_111_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 = mpy(r20, r6.l):<<1:rnd:sat }");

    test_display(&0b1110_1101_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = cmpy(r20, r6):rnd:sat }");
    test_display(&0b1110_1101_011_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = cmpy(r20, r6*):rnd:sat }");
    test_display(&0b1110_1101_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = vmpyh(r20, r6):rnd:sat }");
    test_display(&0b1110_1101_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = cmpy(r20, r6):<<1:rnd:sat }");
    test_display(&0b1110_1101_111_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 = cmpy(r20, r6*):<<1:rnd:sat }");
    test_display(&0b1110_1101_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 = vmpyh(r20, r6):<<1:rnd:sat }");

    test_display(&0b1110_1110_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.l) }");
    test_display(&0b1110_1110_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.h) }");
    test_display(&0b1110_1110_000_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.l) }");
    test_display(&0b1110_1110_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.h) }");
    test_display(&0b1110_1110_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.l):sat }");
    test_display(&0b1110_1110_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.h):sat }");
    test_display(&0b1110_1110_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.l):sat }");
    test_display(&0b1110_1110_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.h):sat }");
    test_display(&0b1110_1110_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.l) }");
    test_display(&0b1110_1110_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.h) }");
    test_display(&0b1110_1110_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.l) }");
    test_display(&0b1110_1110_001_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.h) }");
    test_display(&0b1110_1110_001_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.l):sat }");
    test_display(&0b1110_1110_001_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.h):sat }");
    test_display(&0b1110_1110_001_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.l):sat }");
    test_display(&0b1110_1110_001_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.h):sat }");
    test_display(&0b1110_1110_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_1110_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_1110_100_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_1110_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.h):<<1 }");
    test_display(&0b1110_1110_100_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.l):<<1:sat }");
    test_display(&0b1110_1110_100_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 += mpy(r20.l, r6.h):<<1:sat }");
    test_display(&0b1110_1110_100_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.l):<<1:sat }");
    test_display(&0b1110_1110_100_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 += mpy(r20.h, r6.h):<<1:sat }");
    test_display(&0b1110_1110_101_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.l):<<1 }");
    test_display(&0b1110_1110_101_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.h):<<1 }");
    test_display(&0b1110_1110_101_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.l):<<1 }");
    test_display(&0b1110_1110_101_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.h):<<1 }");
    test_display(&0b1110_1110_101_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.l):<<1:sat }");
    test_display(&0b1110_1110_101_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.l, r6.h):<<1:sat }");
    test_display(&0b1110_1110_101_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.l):<<1:sat }");
    test_display(&0b1110_1110_101_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 -= mpy(r20.h, r6.h):<<1:sat }");

    test_invalid(&0b1110_1110_010_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_010_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_011_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_110_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_000_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_001_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_011_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_100_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_101_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_110_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_invalid(&0b1110_1110_111_10100_11_000110_111_10110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1110_1111_000_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 += mpyi(r20, r6) }");
    test_display(&0b1110_1111_000_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 += add(r20, r6) }");
    test_invalid(&0b1110_1111_000_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_display(&0b1110_1111_000_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 += sub(r6, r20) }");
    test_display(&0b1110_1111_000_10100_11_000110_100_10110u32.to_le_bytes(), "{ r22 += sfmpy(r20, r6) }");
    test_display(&0b1110_1111_000_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 -= sfmpy(r20, r6) }");
    test_display(&0b1110_1111_000_10100_11_000110_110_10110u32.to_le_bytes(), "{ r22 += sfmpy(r20, r6):lib }");
    test_display(&0b1110_1111_000_10100_11_000110_111_10110u32.to_le_bytes(), "{ r22 -= sfmpy(r20, r6):lib }");
    test_display(&0b1110_1111_001_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 |= and(r20, ~r6) }");
    test_display(&0b1110_1111_001_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 &= and(r20, ~r6) }");
    test_display(&0b1110_1111_001_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 ^= and(r20, ~r6) }");
    test_display(&0b1110_1111_010_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 &= and(r20, r6) }");
    test_display(&0b1110_1111_010_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 &= or(r20, r6) }");
    test_display(&0b1110_1111_010_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 &= xor(r20, r6) }");
    test_display(&0b1110_1111_010_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 |= and(r20, r6) }");
    test_display(&0b1110_1111_011_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 += mpy(r20, r6):<<1:sat }");
    test_display(&0b1110_1111_011_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 -= mpy(r20, r6):<<1:sat }");
    test_display(&0b1110_1111_011_10100_11_000110_101_10110u32.to_le_bytes(), "{ r22 += sfmpy(r20, r6, p1):scale }");
    test_display(&0b1110_1111_100_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 -= mpyi(r20, r6) }");
    test_display(&0b1110_1111_100_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 -= add(r20, r6) }");
    test_invalid(&0b1110_1111_100_10100_11_000110_010_10110u32.to_le_bytes(), DecodeError::InvalidOperand);
    test_display(&0b1110_1111_100_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 ^= xor(r20, r6) }");
    test_display(&0b1110_1111_110_10100_11_000110_000_10110u32.to_le_bytes(), "{ r22 |= or(r20, r6) }");
    test_display(&0b1110_1111_110_10100_11_000110_001_10110u32.to_le_bytes(), "{ r22 |= xor(r20, r6) }");
    test_display(&0b1110_1111_110_10100_11_000110_010_10110u32.to_le_bytes(), "{ r22 ^= and(r20, r6) }");
    test_display(&0b1110_1111_110_10100_11_000110_011_10110u32.to_le_bytes(), "{ r22 ^= or(r20, r6) }");
}

#[test]
fn inst_1111() {
    test_display(&0b1111_0001000_00100_11_0_00011_000_00110u32.to_le_bytes(), "{ r6 = and(r4, r3) }");
    test_display(&0b1111_0001001_00100_11_0_00011_000_00110u32.to_le_bytes(), "{ r6 = or(r4, r3) }");
    test_display(&0b1111_0001011_00100_11_0_00011_000_00110u32.to_le_bytes(), "{ r6 = xor(r4, r3) }");
    test_display(&0b1111_0001100_00100_11_0_00011_000_00110u32.to_le_bytes(), "{ r6 = and(r3, ~r4) }");
    test_display(&0b1111_0001101_00100_11_0_00011_000_00110u32.to_le_bytes(), "{ r6 = or(r3, ~r4) }");

    test_display(&0b1111_0010000_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ p2 = cmp.eq(r4, r3) }");
    test_display(&0b1111_0010000_00100_11_0_00011_000_10010u32.to_le_bytes(), "{ p2 = !cmp.eq(r4, r3) }");
    test_display(&0b1111_0010010_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ p2 = cmp.gt(r4, r3) }");
    test_display(&0b1111_0010010_00100_11_0_00011_000_10010u32.to_le_bytes(), "{ p2 = !cmp.gt(r4, r3) }");
    test_display(&0b1111_0010011_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ p2 = cmp.gtu(r4, r3) }");
    test_display(&0b1111_0010011_00100_11_0_00011_000_10010u32.to_le_bytes(), "{ p2 = !cmp.gtu(r4, r3) }");

    test_display(&0b1111_0011000_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = add(r4, r3) }");
    test_display(&0b1111_0011001_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = sub(r3, r4) }");
    test_display(&0b1111_0011010_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = cmp.eq(r4, r3) }");
    test_display(&0b1111_0011011_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = !cmp.eq(r4, r3) }");

    test_display(&0b1111_0011100_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = combine(r3.h, r4.h) }");
    test_display(&0b1111_0011101_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = combine(r3.h, r4.l) }");
    test_display(&0b1111_0011110_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = combine(r3.l, r4.h) }");
    test_display(&0b1111_0011111_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = combine(r3.l, r4.l) }");

    test_display(&0b1111_0100000_00100_11_0_00011_001_00010u32.to_le_bytes(), "{ r2 = mux(p1, r4, r3) }");
    test_display(&0b1111_0101000_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r3:2 = combine(r4, r3) }");
    test_display(&0b1111_0101100_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r3:2 = packhl(r4, r3) }");
    test_display(&0b1111_0110000_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vaddh(r4, r3) }");
    test_display(&0b1111_0110001_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vaddh(r4, r3):sat }");
    test_display(&0b1111_0110010_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = add(r4, r3):sat }");
    test_display(&0b1111_0110011_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vadduh(r4, r3):sat }");
    test_display(&0b1111_0110100_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vsubh(r3, r4) }");
    test_display(&0b1111_0110101_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vsubh(r3, r4):sat }");
    test_display(&0b1111_0110110_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = sub(r3, r4):sat }");
    test_display(&0b1111_0110111_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vsubuh(r3, r4):sat }");
    test_display(&0b1111_0111100_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vavgh(r4, r3) }");
    test_display(&0b1111_0111101_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vavgh(r4, r3):sat }");
    test_invalid(&0b1111_0111110_00100_11_0_00011_000_00010u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1111_0111111_00100_11_0_00011_000_00010u32.to_le_bytes(), "{ r2 = vnavgh(r4, r3) }");

    test_display(&0b1111_1001000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r6 = and(r4, r3) }");
    test_display(&0b1111_1001001_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r6 = or(r4, r3) }");
    test_invalid(&0b1111_1001010_00100_11_1_00011_001_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);
    test_display(&0b1111_1001011_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r6 = xor(r4, r3) }");

    test_display(&0b1111_1011000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r6 = add(r4, r3) }");
    test_display(&0b1111_1011001_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r6 = sub(r4, r3) }");
    test_invalid(&0b1111_1011101_00100_11_1_00011_001_00110u32.to_le_bytes(), DecodeError::InvalidOpcode);

    test_display(&0b1111_1101000_00100_11_1_00011_001_00110u32.to_le_bytes(), "{ if (p1.new) r7:6 = contains(r4, r3) }");
}

    // TODO: testcase for Rn=add(pc,#nn)
