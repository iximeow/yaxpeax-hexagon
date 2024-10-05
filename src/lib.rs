//! qualcomm `hexagon` decoder implemented as part of the `yaxpeax` project. implements traits
//! provided by `yaxpeax-arch`.
//!
//! decoder is written against the ISA described in `Qualcomm Hexagon V73`:
//! * retrieved 2024-09-21 from https://docs.qualcomm.com/bundle/publicresource/80-N2040-53_REV_AB_Qualcomm_Hexagon_V73_Programmers_Reference_Manual.pdf
//! * sha256: `44ebafd1119f725bd3c6ffb87499232520df9a0a6e3e3dc6ea329b15daed11a8`

use core::fmt;

use yaxpeax_arch::{AddressDiff, Arch, Decoder, LengthedInstruction, Reader};
use yaxpeax_arch::StandardDecodeError as DecodeError;

#[derive(Debug)]
pub struct Hexagon;

// TODO: cfg
mod display;

impl Arch for Hexagon {
    type Word = u8;
    /// V73 Section 3.3.7:
    /// > Packets should not wrap the *4GB address space*.
    type Address = u32;
    type Instruction = InstructionPacket;
    type DecodeError = yaxpeax_arch::StandardDecodeError;
    type Decoder = InstDecoder;
    type Operand = Operand;
}

#[derive(Debug, Copy, Clone, Default)]
struct Predicate {
    state: u8,
}

#[derive(Debug, Copy, Clone)]
enum BranchHint {
    Taken,
    NotTaken,
}

macro_rules! opcode_check {
    ($e:expr) => {
        if !$e {
            return Err(DecodeError::InvalidOpcode);
        }
    }
}

macro_rules! operand_check {
    ($e:expr) => {
        if !$e {
            return Err(DecodeError::InvalidOperand);
        }
    }
}

macro_rules! decode_opcode {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => {
                return Err(DecodeError::InvalidOpcode);
            }
        }
    }
}

impl Predicate {
    fn reg(num: u8) -> Self {
        assert!(num <= 0b11);
        Self { state: num }
    }

    fn num(&self) -> u8 {
        self.state & 0b11
    }

    fn set_negated(mut self, bit: bool) -> Self {
        if bit {
            assert!(self.state & 0b0100 == 0);
            self.state |= 0b0100;
        }
        self
    }

    fn negated(&self) -> bool {
        self.state & 0b0100 != 0
    }

    fn set_pred_new(mut self, bit: bool) -> Self {
        if bit {
            assert!(self.state & 0b1000 == 0);
            self.state |= 0b1000;
        }
        self
    }

    fn pred_new(&self) -> bool {
        self.state & 0b1000 != 0
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct LoopEnd {
    loops_ended: u8
}

impl LoopEnd {
    fn end_0(&self) -> bool {
        self.loops_ended & 0b01 != 0
    }

    fn end_1(&self) -> bool {
        self.loops_ended & 0b10 != 0
    }

    fn end_any(&self) -> bool {
        self.loops_ended != 0
    }

    /// NOT FOR PUBLIC
    fn mark_end(&mut self, lp: u8) {
        self.loops_ended |= 1 << lp;
    }
}

/// V73 Section 3.3.3:
/// > The assembler automatically rejects packets that oversubscribe the hardware resources.
///
/// but such bit patterns may exist. invalid packets likely mean the disassembler has walked into
/// invalid code, but should be decoded and shown as-is; the application using `yaxpeax-hexagon`
/// must decide what to do with bogus instruction packets.
#[derive(Debug, Copy, Clone, Default)]
pub struct InstructionPacket {
    /// each packet has up to four instructions (V73 Section 1.1.3)
    instructions: [Instruction; 4],
    /// the actual number of instructions in this packet
    instruction_count: u8,
    /// the number of 4-byte instruction words this packet occupies
    word_count: u8,
    /// how this packet interacts with hardware loops 0 and/or 1
    loop_effect: LoopEnd,
}

/// V5x Section 1.7.2 describes register access syntax. paraphrased:
///
/// registers may be written as `Rds[.elst]`.
///
/// `ds` describes the operand type and bit size:
/// | Symbol | Operand Type | Size (in Bits) |
/// |--------|--------------|----------------|
/// | d      | Destination  | 32             |
/// | dd     | Destination  | 64             |
/// | s      | Source 1     | 32             |
/// | ss     | Source 1     | 64             |
/// | t      | Source 2     | 32             |
/// | tt     | Source 2     | 64             |
/// | u      | Source 3     | 32             |
/// | uu     | Source 3     | 64             |
/// | x      | Source+Dest  | 32             |
/// | xx     | Source+Dest  | 64             |
///
/// `elst` describes access of the bit fields in register `Rds`. V5x Figure 1-4:
///
/// ```
/// |  .b[7] |  .b[6] |  .b[5] |  .b[4] |  .b[3] |  .b[2] |  .b[1] |  .b[0] |     signed bytes
/// | .ub[7] | .ub[6] | .ub[5] | .ub[4] | .ub[3] | .ub[2] | .ub[1] | .ub[0] |   unsigned bytes
/// |       .h[3]     |       .h[2]     |       .h[1]     |       .h[0]     |     signed halfwords
/// |      .uh[3]     |      .uh[2]     |      .uh[1]     |      .uh[0]     |   unsigned halfwords
/// |                .w[1]              |                .w[0]              |     signed words
/// |               .uw[1]              |               .uw[0]              |   unsigned words
/// ```
///
/// meanwhile a register can be accessed as a single element with some trailing specifiers. V5x
/// Table 1-2:
///
/// | Symbol | Meaning |
/// |--------|---------|
/// | .sN    | Bits `[N-1:0]` are treated as an N-bit signed number. For example, R0.s16 means the least significant 16 bits of R0 are treated as a 16-bit signed number. |
/// | .uN    | Bits `[N-1:0]` are treated as an N-bit unsigned number. |
/// | .H     | The most significant 16 bits of a 32-bit register. |
/// | .L     | The least significant 16 bits of a 32-bit register. |
///
/// and finally, "Duplex instructions" (V73 Section 3.6):
/// > Unlike Compound instructions, duplex instructions do not have distinctive syntax – in
/// > assembly code they appear identical to the instructions they are composed of. The assembler
/// > is responsible for recognizing when a pair of instructions can be encoded as a single duplex
/// > rather than a pair of regular instruction words.
///
/// V73 Section 10.3 discusses duplex instructions in more detail:
/// > A duplex is encoded as a 32-bit instruction with bits [15:14] set to 00. The sub-instructions
/// > that comprise a duplex are encoded as 13-bit fields in the duplex.
/// >
/// > The sub-instructions in a duplex always execute in slot 0 and slot 1.
#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    opcode: Opcode,
    dest: Option<Operand>,
    predicate: Option<Predicate>,
    negated: bool,
    branch_hinted: Option<BranchHint>,
    sources: [Operand; 3],
    sources_count: u8,
}

/// V73 Section 3.1 indicates that jumps have taken/not-taken hints, saturation can be a hint,
/// rounding can be a hint, predicate can be used for carry in/out, result shifting by fixed
/// counts, and load/store reordering prevention are all kinds of hints that may be present.
///
/// additionally, V73 Section 3.2 outlines instruction classes which relate to the available
/// execution units:
/// ```
/// XTYPE
///     XTYPE ALU           64-bit ALU operations
///     XTYPE BIT           Bit operations
///     XTYPE COMLPEX
///     XTYPE FP
///     XTYPE MPY
///     XTYPE PERM          Vector permut and format conversion
///     XTYPE PRED          Predicate operations
///     XTYPE SHIFT         Shift operations (with optional ALU)
/// ALU32                   32-bit ALU operations
///     ALU32 ALU           Arithmetic and logical
///     ALU32 PERM          Permute
///     ALU32 PRED          Predicate operations
/// CR
/// JR
/// J
/// LD
/// MEMOP
/// NV
///     NV J
///     NV ST
/// ST
/// SYSTEM
///     SYSTEM USER
/// ```
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Opcode {
    /// TODO: remove. should never be shown. implies an instruction was parially decoded but
    /// accepted?
    BUG,
    // V73 Section 10.9
    // > NOTE: When a constant extender is explicitly specified with a GP-relative load/store, the
    // > processor ignores the value in GP and creates the effective address directly from the 32-bit
    // > constant value.
    //
    // TODO: similar special interpretation of constant extender on 32-bit immediate operands and
    // 32-bit jump/call target addresses.

    Nop,

    // V73 page 214 ("Jump to address")
    Jump,
    Call,

    LoadMemb,
    LoadMemub,
    LoadMemh,
    LoadMemuh,
    LoadMemw,
    LoadMemd,

    StoreMemb,
    StoreMemh,
    StoreMemw,
    StoreMemd,

    Memb, Memub, Memh, Memuh, Memw, Memd,

    Membh,
    MemhFifo,
    Memubh,
    MembFifo,

    Aslh,
    Asrh,
    TransferRegister,
    Zxtb,
    Sxtb,
    Zxth,
    Sxth,

    TransferImmediate,

    Mux,

    Combine,
    CmpEq,
    CmpGt,
    CmpGtu,

    JumpEq,
    JumpNeq,
    JumpGt,
    JumpLe,
    JumpGtu,
    JumpLeu,
    JumpBitSet,
    JumpBitClear,

    Add,
    And,
    Sub,
    Or,

    Tlbw,
    Tlbr,
    Tlbp,
    TlbInvAsid,
    Ctlbw,
    Tlboc,
}

impl Opcode {
    fn cmp_str(&self) -> Option<&'static str> {
        match self {
            Opcode::JumpEq => { Some("cmp.eq") },
            Opcode::JumpNeq => { Some("!cmp.eq") },
            Opcode::JumpGt => { Some("cmp.gt") },
            Opcode::JumpLe => { Some("!cmp.gt") },
            Opcode::JumpGtu => { Some("cmp.gtu") },
            Opcode::JumpLeu => { Some("!cmp.gtu") },
            Opcode::JumpBitSet => { Some("tstbit") },
            Opcode::JumpBitClear => { Some("!tstbit") },
            _ => None
        }
    }
}

/// TODO: don't know if this will be useful, but this is how V73 is described.. it also appears to
/// be the overall structure of the processor at least back to V5x.
/// TODO: how far back does this organization reflect reality? all the way to V2?
enum ExecutionUnit {
    /// Load/store unit
    /// LD, ST, ALU32, MEMOP, NV, SYSTEM
    S0,
    /// Load/store unit
    /// LD, ST, ALU32
    S1,
    /// X unit
    /// XTYPE, ALU32, J, JR
    S2,
    /// X unit
    /// XTYPE, ALU32, J, CR
    S3
}

/// V73 Section 2.1:
/// > thirty-two 32-bit general-purpose registers (named R0 through R31)
///
// TODO: figure out what of this needs to stick around
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct GPR(u8);

// TODO: figure out what of this needs to stick around
#[allow(dead_code)]
impl GPR {
    const SP: GPR = GPR(29);
    const FP: GPR = GPR(30);
    const LR: GPR = GPR(31);
}

impl fmt::Display for GPR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const NAMES: [&'static str; 32] = [
            "R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7",
            "R8", "R9", "R10", "R11", "R12", "R13", "R14", "R15",
            "R16", "R17", "R18", "R19", "R20", "R21", "R22", "R23",
            "R24", "R25", "R26", "R27",
            // the three R29 through R31 general registers support subroutines and the Software
            // Stack. ... they have symbol aliases that indicate when these registers are accessed
            // as subroutine and stack registers (V73 Section 2.1)
            "R28", "SP", "FP", "LR",
        ];

        f.write_str(NAMES[self.0 as usize])
    }
}

/// V73 Section 2.1:
/// > the general registers can be specified as a pair that represent a single 64-bit register.
/// >
/// > NOTE: the first register in a register pair must always be odd-numbered, and the second must be
/// > the next lower register.
///
/// from Table 2-2, note there is an entry of `R31:R30 (LR:FP)`
struct RegPair(u8);

/// V73 Section 2.2:
/// > the Hexagon processor includes a set of 32-bit control registers that provide access to
/// > processor features such as the program counter, hardware loops, and vector predicates.
/// >
/// > unlike general registers, control registers are used as instruction operands only in the
/// > following cases:
/// > * instructions that require a specific control register as an operand
/// > * register transfer instructions
/// >
/// > NOTE: when a control register is used in a register transfer, the other operand must be a
/// > general register.
/// also V73 Section 2.2:
/// > the control registers have numeric aliases (C0 through C31).
///
/// while the names are written out first, the numeric form of the register is probably what is
/// used more often...
///
/// also, the `*LO/*HI` registers seem like they may be used in some circumstances as a pair
/// without the `LO/HI` suffixes, so there may need to be a `ControlRegPair` type too.
// TODO: figure out what of this needs to stick around
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct ControlReg(u8);

// TODO: figure out what of this needs to stick around
#[allow(dead_code)]
impl ControlReg {
    /// Loop start address register 0
    const SA0: ControlReg = ControlReg(0);
    /// Loop count register 0
    const LC0: ControlReg = ControlReg(1);
    /// Loop start address register 1
    const SA1: ControlReg = ControlReg(2);
    /// Loop count register 1
    const LC1: ControlReg = ControlReg(3);
    /// Predicate registers
    const PREDICATES: ControlReg = ControlReg(4);

    // C5 is unused

    /// Modifier register 0
    const M0: ControlReg = ControlReg(6);
    /// Modifier register 1
    const M1: ControlReg = ControlReg(7);
    /// User status register
    ///
    /// V73 Section 2.2.3:
    /// > USR stores the following status and control values:
    /// > * Cache prefetch enable
    /// > * Cache prefetch status
    /// > * Floating point modes
    /// > * Floating point status
    /// > * Hardware loop configuration
    /// > * Sticky Saturation overflow
    /// >
    /// > NOTE: A user control register transfer to USR cannot be gruoped in an instruction packet
    /// with a Floating point instruction.
    /// > NOTE: When a transfer to USR chagnes the enable trap bits [29:25], an isync instruction
    /// (Section 5.11) must execute before the new exception programming can take effect.
    const USR: ControlReg = ControlReg(8);
    /// Program counter
    const PC: ControlReg = ControlReg(9);
    /// User general pointer
    const UGP: ControlReg = ControlReg(10);
    /// Global pointer
    const GP: ControlReg = ControlReg(11);
    /// Circular start register 0
    const CS0: ControlReg = ControlReg(12);
    /// Circular start register 1
    const CS1: ControlReg = ControlReg(13);
    /// Cycle count registers
    ///
    /// according to V5x manual section 1.5, new in V5x
    const UPCYCLELO: ControlReg = ControlReg(14);
    /// Cycle count registers
    ///
    /// according to V5x manual section 1.5, new in V5x
    const UPCYCLEHI: ControlReg = ControlReg(15);
    /// Stack bounds register
    ///
    /// V73 Section 2.2.10:
    /// > The frame limit register (FRAMELIMIT) stores the low address of the memory area reserved
    /// > for the software stack (Section 7.3.1).
    const FRAMELIMIT: ControlReg = ControlReg(16);
    /// Stack smash register
    ///
    /// V73 Section 2.2.11:
    /// > The frame key register (FRAMEKEY) stores the key value that XOR-scrambles return
    /// > addresses when they are stored on the software tack (Section 7.3.2).
    const FRAMEKEY: ControlReg = ControlReg(17);
    /// Packet count registers
    ///
    /// v73 Section 2.2.12:
    /// > The packet count registers (PKTCOUNTLO to PKTCOUNTHI) store a 64-bit value containing the
    /// > current number of instruction packets exceuted since a PKTCOUNT registers was last
    /// > written to.
    const PKTCOUNTLO: ControlReg = ControlReg(18);
    /// Packet count registers
    const PKTCOUNTHI: ControlReg = ControlReg(19);

    // C20-C29 are reserved

    /// Qtimer registers
    ///
    /// V73 Section 2.2.13:
    /// > The QTimer registers (UTIMERLO to UTIMERHI) provide access to the QTimer global reference
    /// > count value. They enable Hexagon software to read the 64-bit time value without having to
    /// > perform an expensive advanced high-performance bus (AHB) load.
    /// > ...
    /// > These registers are read only – hardware automatically updates these registers to contain
    /// > the current QTimer value.
    const UTIMERLO: ControlReg = ControlReg(30);
    /// Qtimer registers
    const UTIMERHI: ControlReg = ControlReg(31);
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        panic!("partialeq")
    }
}

impl Instruction {
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            opcode: Opcode::BUG,
            dest: None,
            predicate: None,
            negated: false,
            branch_hinted: None,
            sources: [Operand::Nothing, Operand::Nothing, Operand::Nothing],
            sources_count: 0,
        }
    }
}

impl LengthedInstruction for InstructionPacket {
    type Unit = AddressDiff<<Hexagon as Arch>::Address>;
    fn min_size() -> Self::Unit {
        AddressDiff::from_const(4)
    }
    fn len(&self) -> Self::Unit {
        AddressDiff::from_const(self.word_count as u32 * 4)
    }
}

impl yaxpeax_arch::Instruction for InstructionPacket {
    // only know how to decode well-formed instructions at the moment
    fn well_defined(&self) -> bool { true }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Operand {
    Nothing,
    /*
    /// one of the 16 32-bit general purpose registers: `R0 (sp)` through `R15`.
    Register { num: u8 },
    /// one of the 16 32-bit general purpose registers, but a smaller part of it. typically
    /// sign-extended to 32b for processing.
    Subreg { num: u8, width: SizeCode },
    */

    PCRel32 { rel: i32 },

    /// `Rn`, a 32-bit register `R<reg>`
    Gpr { reg: u8 },
    /// `Cn`, a 32-bit control register `C<reg>`
    Cr { reg: u8 },
    /// `Sn`, a 32-bit supervisor register `S<reg>`
    Sr { reg: u8 },
    /// `Rn.new`, the version of a 32-bit register `R<reg>` after being written in this instruction
    /// packet.
    GprNew { reg: u8 },
    /// `Rn.L`, low 16 bits of `R<reg>`
    GprLow { reg: u8 },
    /// `Rn.H`, high 16 bits of `R<reg>`
    GprHigh { reg: u8 },
    /// `Rn:m`, register pair forming a 64-bit location
    Gpr64b { reg_low: u8 },
    /// `Cn:m`, control register pair forming a 64-bit location
    Cr64b { reg_low: u8 },
    /// `Sn:m`, control register pair forming a 64-bit location
    Sr64b { reg_low: u8 },

    /// `Pn`, a predicate register
    PredicateReg { reg: u8 },

    RegOffset { base: u8, offset: u32 },

    RegShiftedReg { base: u8, index: u8, shift: u8 },

    ImmU8 { imm: u8 },

    ImmU16 { imm: u16 },

    ImmI8 { imm: i8 },

    ImmI16 { imm: i16 },

    ImmI32 { imm: i32 },

    ImmU32 { imm: u32 },
}

impl Operand {
    fn gpr(num: u8) -> Self {
        Self::Gpr { reg: num }
    }

    fn cr(num: u8) -> Self {
        Self::Cr { reg: num }
    }

    fn sr(num: u8) -> Self {
        Self::Sr { reg: num }
    }

    fn gpr_new(num: u8) -> Self {
        Self::GprNew { reg: num }
    }

    fn gprpair(num: u8) -> Result<Self, yaxpeax_arch::StandardDecodeError> {
        operand_check!(num & 1 == 0);
        Ok(Self::Gpr64b { reg_low: num })
    }

    fn crpair(num: u8) -> Result<Self, yaxpeax_arch::StandardDecodeError> {
        operand_check!(num & 1 == 0);
        Ok(Self::Cr64b { reg_low: num })
    }

    fn srpair(num: u8) -> Result<Self, yaxpeax_arch::StandardDecodeError> {
        operand_check!(num & 1 == 0);
        Ok(Self::Sr64b { reg_low: num })
    }

    fn pred(num: u8) -> Self {
        Self::PredicateReg { reg: num }
    }

    fn imm_i8(num: i8) -> Self {
        Self::ImmI8 { imm: num }
    }

    fn imm_u8(num: u8) -> Self {
        Self::ImmU8 { imm: num }
    }

    fn imm_i16(num: i16) -> Self {
        Self::ImmI16 { imm: num }
    }

    fn imm_u16(num: u16) -> Self {
        Self::ImmU16 { imm: num }
    }

    fn imm_i32(num: i32) -> Self {
        Self::ImmI32 { imm: num }
    }

    fn imm_u32(num: u32) -> Self {
        Self::ImmU32 { imm: num }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SizeCode {
    S,
    B,
    W,
    A,
    L,
    D,
    UW,
}

impl fmt::Display for SizeCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SizeCode::S => "s",
            SizeCode::B => "b",
            SizeCode::W => "w",
            SizeCode::A => "a",
            SizeCode::L => "l",
            SizeCode::D => "d",
            SizeCode::UW => "uw",
        };

        f.write_str(text)
    }
}

impl SizeCode {
    fn bytes(&self) -> u8 {
        match self {
            SizeCode::S => 1,
            SizeCode::B => 1,
            SizeCode::W => 2,
            SizeCode::UW => 2,
            SizeCode::A => 3,
            SizeCode::L => 4,
            SizeCode::D => 8,
        }
    }
}

#[derive(Debug)]
pub struct InstDecoder { }

impl Default for InstDecoder {
    fn default() -> Self {
        InstDecoder {}
    }
}

trait DecodeHandler<T: Reader<<Hexagon as Arch>::Address, <Hexagon as Arch>::Word>> {
    #[inline(always)]
    fn read_u8(&mut self, words: &mut T) -> Result<u8, <Hexagon as Arch>::DecodeError> {
        let b = words.next()?;
        self.on_word_read(b);
        Ok(b)
    }
    #[inline(always)]
    fn read_u16(&mut self, words: &mut T) -> Result<u16, <Hexagon as Arch>::DecodeError> {
        let mut buf = [0u8; 2];
        words.next_n(&mut buf).ok().ok_or(DecodeError::ExhaustedInput)?;
        self.on_word_read(buf[0]);
        self.on_word_read(buf[1]);
        Ok(u16::from_le_bytes(buf))
    }
    #[inline(always)]
    fn read_u32(&mut self, words: &mut T) -> Result<u32, <Hexagon as Arch>::DecodeError> {
        let mut buf = [0u8; 4];
        words.next_n(&mut buf).ok().ok_or(DecodeError::ExhaustedInput)?;
        self.on_word_read(buf[0]);
        self.on_word_read(buf[1]);
        self.on_word_read(buf[2]);
        self.on_word_read(buf[3]);
        Ok(u32::from_le_bytes(buf))
    }
    fn read_inst_word(&mut self, words: &mut T) -> Result<u32, <Hexagon as Arch>::DecodeError>;
    fn on_decode_start(&mut self) {}
    fn on_decode_end(&mut self) {}
    fn start_instruction(&mut self);
    fn end_instruction(&mut self);
    fn on_loop_end(&mut self, loop_num: u8);
    fn on_opcode_decoded(&mut self, _opcode: Opcode) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn on_source_decoded(&mut self, _operand: Operand) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn on_dest_decoded(&mut self, _operand: Operand) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn inst_predicated(&mut self, num: u8, negated: bool, pred_new: bool) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn negate_result(&mut self) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn branch_hint(&mut self, hint_taken: bool) -> Result<(), <Hexagon as Arch>::DecodeError> { Ok(()) }
    fn on_word_read(&mut self, _word: <Hexagon as Arch>::Word) {}
}

impl<T: yaxpeax_arch::Reader<<Hexagon as Arch>::Address, <Hexagon as Arch>::Word>> DecodeHandler<T> for InstructionPacket {
    fn on_decode_start(&mut self) {
        self.instructions = [Instruction::default(); 4];
        self.instruction_count = 0;
        self.word_count = 0;
    }
    fn on_loop_end(&mut self, loop_num: u8) {
        self.loop_effect.mark_end(loop_num);
    }
    fn on_opcode_decoded(&mut self, opcode: Opcode) -> Result<(), <Hexagon as Arch>::DecodeError> {
        self.instructions[self.instruction_count as usize].opcode = opcode;
        Ok(())
    }
    fn on_source_decoded(&mut self, operand: Operand) -> Result<(), <Hexagon as Arch>::DecodeError> {
        let mut inst = &mut self.instructions[self.instruction_count as usize];
        inst.sources[inst.sources_count as usize] = operand;
        inst.sources_count += 1;
        Ok(())
    }
    fn on_dest_decoded(&mut self, operand: Operand) -> Result<(), <Hexagon as Arch>::DecodeError> {
        let mut inst = &mut self.instructions[self.instruction_count as usize];
        assert!(inst.dest.is_none());
        inst.dest = Some(operand);
        Ok(())
    }
    fn inst_predicated(&mut self, num: u8, negated: bool, pred_new: bool) -> Result<(), <Hexagon as Arch>::DecodeError> {
        let mut inst = &mut self.instructions[self.instruction_count as usize];
        assert!(inst.predicate.is_none());
        inst.predicate = Some(Predicate::reg(num).set_negated(negated).set_pred_new(pred_new));
        Ok(())
    }
    fn negate_result(&mut self) -> Result<(), <Hexagon as Arch>::DecodeError> {
        let mut inst = &mut self.instructions[self.instruction_count as usize];
        assert!(!inst.negated);
        inst.negated = true;
        Ok(())
    }
    fn branch_hint(&mut self, hint_taken: bool) -> Result<(), <Hexagon as Arch>::DecodeError> {
        let mut inst = &mut self.instructions[self.instruction_count as usize];
        assert!(inst.branch_hinted.is_none());
        if hint_taken {
            inst.branch_hinted = Some(BranchHint::Taken);
        } else {
            inst.branch_hinted = Some(BranchHint::NotTaken);
        }
        Ok(())
    }
    #[inline(always)]
    fn read_inst_word(&mut self, words: &mut T) -> Result<u32, <Hexagon as Arch>::DecodeError> {
        self.word_count += 1;
        self.read_u32(words)
    }
    fn on_word_read(&mut self, _word: <Hexagon as Arch>::Word) { }
    fn start_instruction(&mut self) { }
    fn end_instruction(&mut self) {
        self.instruction_count += 1;
    }
}

impl Decoder<Hexagon> for InstDecoder {
    fn decode_into<T: Reader<<Hexagon as Arch>::Address, <Hexagon as Arch>::Word>>(&self, packet: &mut InstructionPacket, words: &mut T) -> Result<(), <Hexagon as Arch>::DecodeError> {
        decode_packet(self, packet, words)
    }
}

fn reg_b0(inst: u32) -> u8 { (inst & 0b11111) as u8 }
fn reg_b8(inst: u32) -> u8 { ((inst >> 8) & 0b11111) as u8 }
fn reg_b16(inst: u32) -> u8 { ((inst >> 16) & 0b11111) as u8 }

fn decode_packet<
    T: Reader<<Hexagon as Arch>::Address, <Hexagon as Arch>::Word>,
    H: DecodeHandler<T>,
>(decoder: &<Hexagon as Arch>::Decoder, handler: &mut H, words: &mut T) -> Result<(), <Hexagon as Arch>::DecodeError> {
    handler.on_decode_start();

    let mut current_word = 0;

    // V73 Section 10.6:
    // > In addition to encoding the last instruction in a packet, the Parse field of the
    // > instruction word (Section 10.5) encodes the last packet in a hardware loop.
    //
    // accumulate Parse fields to comapre against V73 Table 10-7 once we've read the whole
    // packet.
    //
    // TODO: if the first instruction is a duplex, does that mean the packet cannot indicate
    // loop end?
    let mut loop_bits: u8 = 0b0000;

    // V74 Section 10.6:
    // > A constant extender is encoded as a 32-bit instruction with the 4-bit ICLASS field set to
    // > 0 and the 2-bit Parse field set to its usual value (Section 10.5). The remaining 26 bits in
    // > the instruction word store the data bits that are prepended to an operand as small as six
    // > bits to create a full 32-bit value.
    // > ...
    // > If the instruction operand to extend is longer than six bits, the overlapping bits in the
    // > base instruction must be encoded as zeros. The value in the constant extender always
    // > supplies the upper 26 bits.
    let mut extender: Option<u32> = None;

    // have we seen an end of packet?
    let mut end = false;

    while !end {
        if current_word >= 4 {
            panic!("TODO: instruction too large");
            // Err(DecodeError::InstructionTooLarge)
        }

        let inst: u32 = handler.read_inst_word(words)?;

        // V73 Section 10.5:
        // > Instruction packets are encoded using two bits of the instruction word (15:14), whic
        // > are referred to as the Parse field of the instruction word.
        let parse = (inst >> 14) & 0b11;

        if current_word == 0 {
            loop_bits |= parse as u8;
        } else if current_word == 1 {
            loop_bits |= (parse as u8) << 2;
        }

        // V73 Section 10.5:
        // > 11 indicates that an instruction is the last instruction in a packet
        // > 01 or 10 indicate that an instruction is not the last instruction in a packet
        // > 00 indicates a duplex
        match parse {
            0b00 => {
                /* duplex instruction */
                // see table 10-2
                // exactly how subinstructions are encoded is unclear...
                println!("duplex,");
            }
            0b01 | 0b10 => { /* nothing to do here */ }
            0b11 => {
                end = true;

                if loop_bits & 0b0111 == 0b0110 {
                    handler.on_loop_end(0);
                } else if loop_bits == 0b1001 {
                    handler.on_loop_end(1);
                } else if loop_bits == 0b1010 {
                    handler.on_loop_end(0);
                    handler.on_loop_end(1);
                }
            }
            _ => {
                unreachable!();
            }
        }

        let iclass = (inst >> 28) & 0b1111;

        if iclass == 0b0000 {
            extender = Some((inst & 0x3fff) | ((inst >> 2) & 0xfff));
        } else {
            handler.start_instruction();
            decode_instruction(decoder, handler, inst, extender)?;
            handler.end_instruction();
        }

        current_word += 1;
    }

    Ok(())
}

fn can_be_extended(iclass: u8, regclass: u8) -> bool {
    panic!("TODO: Table 10-10")
}

fn decode_instruction<
    T: Reader<<Hexagon as Arch>::Address, <Hexagon as Arch>::Word>,
    H: DecodeHandler<T>,
>(decoder: &<Hexagon as Arch>::Decoder, handler: &mut H, inst: u32, extender: Option<u32>) -> Result<(), <Hexagon as Arch>::DecodeError> {
    let iclass = (inst >> 28) & 0b1111;

    use Opcode::*;

    // V73 Section 10.9
    // > A constant extender must be positioned in a packet immediately before the
    // > instruction that it extends
    // > ...
    // > If a constant extender is encoded in a packet for an instruction that does not
    // > accept a constant extender, the execution result is undefined. The assembler
    // > normally ensures that only valid constant extenders are generated.
    if extender.is_some() {
        eprintln!("TODO: error; unconsumed extender");
    }

    // this is *called* "RegType" in the manual but it seem to more often describe
    // opcodes?
    let reg_type = (inst >> 24) & 0b1111;
    let min_op = (inst >> 21) & 0b111;

    match iclass {
        0b0010 => {
            if (inst >> 27) & 1 == 1 {
                // everything at
                // 0010 |1xxxxxxx.. is an undefined encoding
                return Err(DecodeError::InvalidOpcode);
            }

            let hint_taken = (inst >> 13) & 1 == 1;
            let op = (inst >> 22) & 0b11111;

            let r_lo = (inst >> 1) & 0b111_1111;
            let r_hi = (inst >> 20) & 0b11;
            let r = (((r_hi << 7) | r_lo) << 2) as i16;
            let r = r << 5 >> 5;

            let sss = ((inst >> 16) & 0b111) as u8;

            handler.on_dest_decoded(Operand::PCRel32 {
                rel: r as i32
            })?;

            handler.branch_hint(hint_taken)?;

            if op < 0b10000 {
                // these are `if([!]cmp.op(Ns.new,Rt)) jump:[n]t #r9:2`
                let ttttt = reg_b8(inst);

                if op < 0b0110 {
                    handler.on_source_decoded(Operand::GprNew { reg: sss })?;
                    handler.on_source_decoded(Operand::gpr(ttttt))?;
                } else {
                    handler.on_source_decoded(Operand::gpr(ttttt))?;
                    handler.on_source_decoded(Operand::GprNew { reg: sss })?;
                }

                static OPS: [Option<Opcode>; 16] = [
                    Some(JumpEq), Some(JumpNeq), Some(JumpGt), Some(JumpLe),
                    Some(JumpGtu), Some(JumpLeu), Some(JumpGt), Some(JumpLe),
                    Some(JumpGtu), Some(JumpLeu), None, None,
                    None, None, None, None,
                ];

                handler.on_opcode_decoded(decode_opcode!(OPS[op as usize]))?;
            } else {
                handler.on_source_decoded(Operand::GprNew { reg: sss })?;

                if op < 0b10110 {
                    let lllll = reg_b8(inst);
                    static OPS: &[Opcode; 6] = &[
                        JumpEq, JumpNeq, JumpGt, JumpLe,
                        JumpGtu, JumpLeu
                    ];
                    handler.on_source_decoded(Operand::imm_u32(lllll as u8 as u32))?;
                    handler.on_opcode_decoded(OPS[op as usize - 0b10000])?;
                } else if op < 0b11000 {
                    let opc = if op == 0b10110 {
                        JumpBitSet
                    } else {
                        JumpBitClear
                    };
                    handler.on_source_decoded(Operand::imm_u32(0))?;
                    handler.on_opcode_decoded(opc)?;
                } else if op < 0b11100 {
                    static OPS: &[Opcode; 4] = &[
                        JumpEq, JumpNeq, JumpGt, JumpLe,
                    ];
                    handler.on_source_decoded(Operand::imm_i32(-1))?;
                    handler.on_opcode_decoded(OPS[op as usize - 0b11000])?;
                } else {
                    return Err(DecodeError::InvalidOpcode);
                }
            }
        }
        0b0011 => {
            let upper = reg_type >> 2;
            match upper {
                0b00 => {
                    // 0011 | 00xxxxxxx
                    // everything under this is a predicated load
                    let nn = (inst >> 24) & 0b11;

                    let negated = nn & 1 == 1;
                    let pred_new = nn >> 1 == 1;

                    let ddddd = reg_b0(inst);
                    let vv = ((inst >> 5) & 0b11) as u8;
                    let i_lo = (inst >> 7) & 0b1;
                    let ttttt = reg_b8(inst);
                    let i_hi = ((inst >> 13) & 0b1) << 1;
                    let ii = (i_lo | i_hi) as u8;
                    let sssss = reg_b16(inst);
                    let op = (inst >> 21) & 0b111;

                    handler.inst_predicated(vv, negated, pred_new)?;

                    handler.on_source_decoded(Operand::RegShiftedReg { base: sssss, index: ttttt, shift: ii })?;
                    if op == 0b110 {
                        handler.on_dest_decoded(Operand::gprpair(ddddd)?)?;
                    } else {
                        handler.on_dest_decoded(Operand::gpr(ddddd))?;
                    }

                    use Opcode::*;
                    static OPCODES: [Option<Opcode>; 8] = [
                        Some(LoadMemb), Some(LoadMemub), Some(LoadMemh), Some(LoadMemuh),
                        Some(LoadMemw), None,        Some(LoadMemd), None,
                    ];
                    handler.on_opcode_decoded(OPCODES[op as usize].ok_or(DecodeError::InvalidOpcode)?)?;
                }
                0b01 => {
                    // 00011 | 01xxxxxxx
                    // everything under this is a predicated store
                    let nn = (inst >> 24) & 0b11;

                    let negated = nn & 1 == 1;
                    let pred_new = nn >> 1 == 1;

                    let ttttt = reg_b0(inst);
                    let vv = ((inst >> 5) & 0b11) as u8;
                    let i_lo = (inst >> 7) & 0b1;
                    let uuuuu = reg_b8(inst);
                    let i_hi = ((inst >> 13) & 0b1) << 1;
                    let ii = (i_lo | i_hi) as u8;
                    let sssss = reg_b16(inst);
                    let op = (inst >> 21) & 0b111;

                    handler.inst_predicated(vv, negated, pred_new)?;
                    handler.on_dest_decoded(Operand::RegShiftedReg { base: sssss, index: uuuuu, shift: ii })?;

                    if op == 0b101 {
                        // more complicated procedure, and registers are `Nt.new`
                        let op = (inst >> 3) & 0b11;
                        let ttt = inst & 0b111;
                        static OPCODES: [Option<Opcode>; 4] = [
                            Some(StoreMemb), Some(StoreMemh),
                            Some(StoreMemw), None,
                        ];
                        handler.on_opcode_decoded(decode_opcode!(OPCODES[op as usize]))?;
                        handler.on_source_decoded(Operand::gpr_new(ttt as u8))?;
                    } else {
                        static OPCODES: [Option<Opcode>; 8] = [
                            Some(StoreMemb), None, Some(StoreMemh), Some(StoreMemh),
                            Some(StoreMemw), None,        Some(StoreMemd), None,
                        ];
                        handler.on_opcode_decoded(decode_opcode!(OPCODES[op as usize]))?;
                        if op == 0b011 {
                            handler.on_source_decoded(Operand::GprHigh { reg: ttttt })?;
                        } else if op == 0b110 {
                            handler.on_source_decoded(Operand::gpr(ttttt))?;
                        } else {
                            handler.on_source_decoded(Operand::gprpair(ttttt)?)?;
                        }
                    }
                }
                0b10 => {
                    // 00011 | 10xxxxxxx
                    // some predicated stores, but also some predicated loads
                    let sssss = reg_b16(inst);

                    let predicate_store = (inst >> 25) & 0b1 == 0;
                    if predicate_store {
                        let nn = (inst >> 23) & 0b11;

                        let negated = nn & 1 == 1;
                        let pred_new = nn >> 1 == 1;

                        let l_lo = reg_b0(inst);
                        let vv = ((inst >> 5) & 0b11) as u8;
                        let iiiiii = (inst >> 7) & 0b11_1111;
                        let l_hi = (((inst >> 13) & 0b1) << 5) as u8;
                        let llllll = (l_lo | l_hi) as u8;
                        let op = (inst >> 21) & 0b11;

                        let negated = nn & 1 == 1;
                        let pred_new = nn >> 1 == 1;

                        handler.inst_predicated(vv, negated, pred_new)?;

                        match op {
                            0b00 => {
                                handler.on_opcode_decoded(Opcode::StoreMemb)?;
                                let offset = iiiiii;
                                let imm = (llllll as i8) << 2 >> 2;
                                handler.on_dest_decoded(Operand::RegOffset { base: sssss, offset })?;
                                handler.on_source_decoded(Operand::imm_i8(imm))?;
                            }
                            0b01 => {
                                handler.on_opcode_decoded(Opcode::StoreMemh)?;
                                let offset = iiiiii << 1;
                                let imm = (llllll as i16) << 10 >> 10;
                                handler.on_dest_decoded(Operand::RegOffset { base: sssss, offset })?;
                                handler.on_source_decoded(Operand::imm_i16(imm))?;
                            }
                            0b10 => {
                                handler.on_opcode_decoded(Opcode::StoreMemw)?;
                                let offset = iiiiii << 2;
                                let imm = (llllll as i32) << 26 >> 26;
                                handler.on_dest_decoded(Operand::RegOffset { base: sssss, offset })?;
                                handler.on_source_decoded(Operand::imm_i32(imm))?;
                            }
                            _ => {
                                return Err(DecodeError::InvalidOpcode);
                            }
                        }
                    } else {
                        // op determined by min_op
                        let store = (inst >> 24) & 1 == 1;

                        let reg_low = reg_b0(inst);
                        let reg_mid = reg_b8(inst);
                        let sssss = reg_b16(inst);

                        let i_lo = ((inst >> 7) & 1) as u8;
                        let i_hi = (((inst >> 13) & 0b1) << 1) as u8;
                        let ii = i_hi | i_lo;

                        if store {
                            // StoreMem*
                            handler.on_dest_decoded(Operand::RegShiftedReg { base: sssss, index: reg_mid, shift: ii })?;
                            match min_op {
                                0b010 => {
                                    handler.on_opcode_decoded(Opcode::StoreMemh)?;
                                    handler.on_source_decoded(Operand::gpr(reg_low))?;
                                },
                                0b011 => {
                                    handler.on_opcode_decoded(Opcode::StoreMemh)?;
                                    handler.on_source_decoded(Operand::GprHigh { reg: reg_low})?;
                                },
                                0b100 => {
                                    handler.on_opcode_decoded(Opcode::StoreMemb)?;
                                    handler.on_source_decoded(Operand::gprpair(reg_low)?)?;
                                }
                                0b101 => {
                                    // more complicated procedure, and registers are `Nt.new`
                                    let op = (inst >> 3) & 0b11;
                                    let ttt = inst & 0b111;
                                    static OPCODES: [Option<Opcode>; 4] = [
                                        Some(StoreMemb), Some(StoreMemh),
                                        Some(StoreMemw), None,
                                    ];
                                    handler.on_opcode_decoded(decode_opcode!(OPCODES[op as usize]))?;
                                    handler.on_source_decoded(Operand::gpr_new(ttt as u8))?;
                                },
                                0b0110 => {
                                    handler.on_opcode_decoded(Opcode::StoreMemd)?;
                                    handler.on_source_decoded(Operand::gprpair(reg_low)?)?;
                                }
                                _ => {
                                    return Err(DecodeError::InvalidOpcode);
                                }
                            }
                        } else {
                            // LoadMem*
                            static OPCODES: [Option<Opcode>; 8] = [
                                Some(LoadMemb), Some(LoadMemub), Some(LoadMemh), Some(LoadMemh),
                                Some(LoadMemw), None,        Some(LoadMemd), None,
                            ];

                            handler.on_opcode_decoded(decode_opcode!(OPCODES[min_op as usize]))?;
                            handler.on_source_decoded(Operand::RegShiftedReg { base: sssss, index: reg_mid, shift: ii })?;
                            if min_op == 0b001 || min_op == 0b011 || min_op == 0b110 {
                                handler.on_dest_decoded(Operand::gprpair(reg_low)?)?;
                            } else {
                                handler.on_dest_decoded(Operand::gpr(reg_low))?;
                            }
                        }
                    }
                }
                other => {
                    // 0b11, so bits are like 0011|11xxxx
                    // these are all stores to Rs+#u6:N, shift is determined by op size.
                    // the first few are stores of immediates, most others operate on registers.
                    panic!("TODO: other: {}", other);
                }
            }
        }
        0b0101 => {
            let majop = (inst >> 25) & 0b111;
            match majop {
                0b000 => {
                    // 0 1 0 1 | 0 0 0 ...
                    // call to register, may be predicated
                    panic!("TODO");
                },
                0b000 => {
                    // 0 1 0 1 | 0 0 1 ...
                    // jump to register, may be predicated
                    panic!("TODO");
                },
                0b100 => {
                    // V73 Jump to address
                    // 0 1 0 1 | 1 0 0 i...
                    handler.on_opcode_decoded(Opcode::Jump)?;
                    let imm = ((inst >> 1) & 0x7fff) | ((inst >> 3) & 0xff8000);
                    let imm = ((imm as i32) << 10) >> 10;
                    handler.on_source_decoded(Operand::PCRel32 { rel: imm & !0b11 })?;
                },
                0b101 => {
                    // V73 Call to address
                    // 0 1 0 1 | 1 0 1 i...
                    if inst & 0 != 0 {
                        // unlike jump, for some reason, the last bit is defined to be 0
                        return Err(DecodeError::InvalidOpcode);
                    }
                    handler.on_opcode_decoded(Opcode::Call)?;
                    let imm = ((inst >> 1) & 0x7fff) | ((inst >> 3) & 0xff8000);
                    let imm = ((imm as i32) << 10) >> 10;
                    handler.on_source_decoded(Operand::PCRel32 { rel: imm & !0b11 })?;
                },
                _ => {
                    // TODO: exhaustive
                }
            }
        },
        0b0110 => {
            let opbits = (inst >> 21) & 0b1111111;

            match opbits {
                0b0010001 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                    handler.on_dest_decoded(Operand::cr(ddddd))?;
                }
                0b0111000 => {
                    // not in V73! store to supervisor register?
                    let sssss = reg_b16(inst);
                    let ddddddd = inst & 0b111_1111;
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_dest_decoded(Operand::sr(ddddddd as u8))?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                }
                0b1101000 => {
                    // not in V73! store to supervisor register?
                    let sssss = reg_b16(inst);
                    let ddddddd = inst & 0b111_1111;
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_dest_decoded(Operand::srpair(ddddddd as u8)?)?;
                    handler.on_source_decoded(Operand::gprpair(sssss)?)?;
                }
                0b1110100 | 0b1110101 |
                0b1110110 | 0b1110111 => {
                    // not in V73! load supervisor register?
                    let sssss = reg_b0(inst);
                    let ddddddd = (inst >> 16) & 0b111_1111;
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::sr(ddddddd as u8))?;
                    handler.on_dest_decoded(Operand::gpr(sssss))?;
                }
                0b1111000 | 0b1111001 |
                0b1111010 | 0b1111011 => {
                    // not in V73! load supervisor register?
                    let sssss = reg_b0(inst);
                    let ddddddd = (inst >> 16) & 0b111_1111;
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::srpair(ddddddd as u8)?)?;
                    handler.on_dest_decoded(Operand::gprpair(sssss)?)?;
                }
                0b0011001 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::gprpair(sssss)?)?;
                    handler.on_dest_decoded(Operand::crpair(ddddd)?)?;
                }
                0b1000000 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::crpair(ddddd)?)?;
                    handler.on_dest_decoded(Operand::gprpair(sssss)?)?;
                }
                0b1010000 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::TransferRegister)?;
                    handler.on_source_decoded(Operand::cr(ddddd))?;
                    handler.on_dest_decoded(Operand::gpr(sssss))?;
                }
                0b1010010 => {
                    if (inst >> 16) & 0b11111 != 0b01001 {
                        // TODO: this is checking for register number 9 aka PC.
                        // if other register numbers are specified here, can you add from other
                        // control registers into a GPR?
                        return Err(DecodeError::InvalidOperand);
                    }

                    let iiiiii = (inst >> 7) & 0b111111;
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::Add)?;
                    handler.on_source_decoded(Operand::cr(9))?;
                    handler.on_source_decoded(Operand::imm_u8(iiiiii as u8))?;
                    handler.on_dest_decoded(Operand::gpr(ddddd))?;
                }
                0b1100000 => {
                    opcode_check!(inst & 0x2000 == 0);
                    let sssss = reg_b16(inst);
                    let ttttt = reg_b8(inst);
                    handler.on_opcode_decoded(Opcode::Tlbw)?;
                    handler.on_source_decoded(Operand::gprpair(sssss)?)?;
                    handler.on_source_decoded(Operand::gpr(ttttt))?;
                }
                0b1100010 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::Tlbr)?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                    handler.on_dest_decoded(Operand::gprpair(ddddd)?)?;
                }
                0b1100100 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::Tlbp)?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                    handler.on_dest_decoded(Operand::gpr(ddddd))?;
                }
                0b1100101 => {
                    let sssss = reg_b16(inst);
                    handler.on_opcode_decoded(Opcode::TlbInvAsid)?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                }
                0b1100110 => {
                    opcode_check!(inst & 0x2000 == 0);
                    let sssss = reg_b16(inst);
                    let ttttt = reg_b8(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::Ctlbw)?;
                    handler.on_source_decoded(Operand::gprpair(sssss)?)?;
                    handler.on_source_decoded(Operand::gpr(ttttt))?;
                    handler.on_dest_decoded(Operand::gpr(ddddd))?;
                }
                0b1100111 => {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    handler.on_opcode_decoded(Opcode::Tlboc)?;
                    handler.on_source_decoded(Operand::gprpair(sssss)?)?;
                    handler.on_dest_decoded(Operand::gpr(ddddd))?;
                }
                _ => {
                    eprintln!("!!! TODO: {:07b} !!!", opbits);
                }
            }
        }
        0b0111 => {
            match reg_type {
            0b0000 => {
                static OPS: [Option<Opcode>; 8] = [
                    Some(Opcode::Aslh), Some(Opcode::Asrh), None, Some(Opcode::TransferRegister),
                    Some(Opcode::Zxtb), Some(Opcode::Sxtb), Some(Opcode::Zxth), Some(Opcode::Sxth),
                ];

                let opcode = decode_opcode!(OPS[min_op as usize]);

                let ddddd = reg_b0(inst);
                let sssss = reg_b16(inst);
                let predicated = (inst >> 13) & 1 != 0;

                if opcode == Opcode::TransferRegister {
                    // no support for predicated register transfer..?
                    opcode_check!(!predicated);
                } else if opcode == Opcode::Zxtb {
                    // non-predicated zext is assembled as `Rd=and(Rs,#255)`
                    // really curious if hardware supports this instruction anyway...
                    opcode_check!(predicated);
                }

                handler.on_opcode_decoded(opcode)?;

                if predicated {
                    let pred_bits = (inst >> 10) & 0b11;
                    let negated = pred_bits >> 1 != 0;
                    let dotnew = pred_bits & 1 != 0;
                    let pred_number = (inst >> 8) & 0b11;

                    handler.inst_predicated(pred_number as u8, negated, dotnew)?;
                }

                handler.on_dest_decoded(Operand::Gpr { reg: ddddd })?;
                handler.on_source_decoded(Operand::Gpr { reg: sssss })?;
            }
            0b0001 => {
                let imm16 = inst & 0xffff;
                opcode_check!(min_op & 1 == 1);

                let i_high = ((inst >> 8) & 0xc000) as u16;
                let i_low = (inst & 0x3fff) as u16;
                let i = i_high | i_low;
                let xxxxx = reg_b16(inst);

                handler.on_opcode_decoded(Opcode::TransferImmediate)?;
                handler.on_dest_decoded(Operand::GprLow { reg: xxxxx })?;
                handler.on_source_decoded(Operand::ImmU16 { imm: i })?;
            }
            0b0010 => {
                let imm16 = inst & 0xffff;
                opcode_check!(min_op & 1 == 1);

                let i_high = ((inst >> 8) & 0xc000) as u16;
                let i_low = (inst & 0x3fff) as u16;
                let i = i_high | i_low;
                let xxxxx = reg_b16(inst);

                handler.on_opcode_decoded(Opcode::TransferImmediate)?;
                handler.on_dest_decoded(Operand::GprHigh { reg: xxxxx })?;
                handler.on_source_decoded(Operand::ImmU16 { imm: i })?;
            }
            0b0011 => {
                // in either case there is a #s8, but the exact operation is still TBD
                let i = ((inst >> 5) & 0xff) as i8;

                if inst & 0x2000 != 0 {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);

                    let op = if min_op & 0b010 == 0 {
                        handler.on_dest_decoded(Operand::gprpair(ddddd)?)?;
                        Opcode::Combine
                    } else {
                        handler.on_dest_decoded(Operand::gpr(ddddd))?;
                        Opcode::CmpEq
                    };
                    handler.on_opcode_decoded(op);

                    let min_low = min_op & 0b11;

                    if min_low == 0b11 {
                        handler.negate_result();
                    }

                    if min_low == 0b01 {
                        handler.on_source_decoded(Operand::imm_i8(i));
                        handler.on_source_decoded(Operand::gpr(sssss));
                    } else {
                        handler.on_source_decoded(Operand::gpr(sssss));
                        handler.on_source_decoded(Operand::imm_i8(i));
                    }
                } else {
                    let sssss = reg_b16(inst);
                    let ddddd = reg_b0(inst);
                    let uu = min_op as u8 & 0b11;

                    handler.on_opcode_decoded(Opcode::Mux)?;
                    handler.on_dest_decoded(Operand::gpr(ddddd))?;
                    handler.on_source_decoded(Operand::pred(uu))?;

                    let reg_first = min_op & 0b100 == 0;

                    if reg_first {
                        handler.on_source_decoded(Operand::gpr(sssss))?;
                        handler.on_source_decoded(Operand::imm_i8(i))?;
                    } else {
                        handler.on_source_decoded(Operand::imm_i8(i))?;
                        handler.on_source_decoded(Operand::gpr(sssss))?;
                    }
                }
            }
            0b0100 => {
                let sssss = reg_b16(inst);
                let ddddd = reg_b0(inst);

                handler.on_opcode_decoded(Opcode::Add);

                let negated = (min_op >> 2) & 1 == 1;
                let pred_number = min_op & 0b11;
                let dotnew = inst >> 13 & 1 == 1;
                let i = ((inst >> 5) & 0xff) as i8;

                handler.inst_predicated(pred_number as u8, negated, dotnew)?;
                handler.on_dest_decoded(Operand::gpr(ddddd))?;
                handler.on_source_decoded(Operand::gpr(sssss))?;
                handler.on_source_decoded(Operand::imm_i8(i))?;
            }
            0b0101 => {
                let sssss = reg_b16(inst);
                let ddddd = reg_b0(inst);
                let dd = ddddd & 0b11;
                operand_check!(ddddd & 0b01100 == 0);

                let i_hi = ((min_op & 0b1) as i16) << 15 >> 6;
                let i_lo = ((inst >> 5) as i16) & 0b1_1111_1111;
                let i = i_lo | i_hi;

                let op_bits = min_op >> 1;

                static OPS: [Option<Opcode>; 4] = [
                    Some(Opcode::CmpEq), Some(Opcode::CmpGt),
                    Some(Opcode::CmpGtu), None,
                ];

                let opcode = decode_opcode!(OPS[op_bits as usize]);

                let negated = ddddd & 0b10000 != 0;
                if negated {
                    handler.negate_result()?;
                }

                handler.on_opcode_decoded(opcode)?;
                handler.on_dest_decoded(Operand::pred(dd))?;
                handler.on_source_decoded(Operand::gpr(sssss))?;
                if opcode == Opcode::CmpGtu {
                    operand_check!(i >= 0);
                    handler.on_source_decoded(Operand::imm_u16(i as u16))?;
                } else {
                    handler.on_source_decoded(Operand::imm_i16(i))?;
                }
            }
            0b0110 => {
                let sssss = reg_b16(inst);
                let ddddd = reg_b0(inst);
                let dd = ddddd & 0b11;

                let i_hi = ((min_op & 0b1) as i16) << 15 >> 6;
                let i_lo = ((inst >> 5) as i16) & 0b1_1111_1111;
                let i = i_lo | i_hi;

                let op_bits = min_op >> 1;

                static OPS: [Option<Opcode>; 4] = [
                    Some(Opcode::And), Some(Opcode::Sub),
                    Some(Opcode::Or), None,
                ];

                let opcode = decode_opcode!(OPS[op_bits as usize]);

                handler.on_opcode_decoded(opcode)?;
                handler.on_dest_decoded(Operand::gpr(ddddd))?;
                if opcode == Opcode::Sub {
                    handler.on_source_decoded(Operand::imm_i16(i))?;
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                } else {
                    handler.on_source_decoded(Operand::gpr(sssss))?;
                    handler.on_source_decoded(Operand::imm_i16(i))?;
                }
            }
            0b0111 => {
                return Err(DecodeError::InvalidOpcode);
            }
            0b1000 => {
                let ddddd = reg_b0(inst);

                let i_hi = (min_op >> 1) as i16;
                let i_mid = reg_b16(inst) as i16;
                let i_lo = ((inst >> 5) as i16) & 0b1_1111_1111;
                let i = i_lo | (i_mid << 5) | (i_hi << 14);

                handler.on_opcode_decoded(Opcode::TransferImmediate)?;
                handler.on_dest_decoded(Operand::gpr(ddddd))?;
                handler.on_source_decoded(Operand::imm_i16(i))?;
            }
            0b1010 |
            0b1011 => {
                let ddddd = reg_b0(inst);

                let i = (inst >> 5) as i8;
                let uu = (inst >> 23) & 0b11;
                let l_lo = ((inst >> 13) & 0b01) as i8;
                let l_hi = ((inst >> 16) & 0x7f) as i8;
                let l = l_lo | (l_hi << 1);

                handler.on_opcode_decoded(Opcode::Mux)?;
                handler.on_dest_decoded(Operand::gpr(ddddd))?;
                handler.on_source_decoded(Operand::pred(uu as u8))?;
                handler.on_source_decoded(Operand::imm_i8(i))?;
                handler.on_source_decoded(Operand::imm_i8(l))?;
            }
            0b1100 => {
                let ddddd = reg_b0(inst);

                let i = (inst >> 5) as i8;
                let l_lo = ((inst >> 13) & 0b01) as i8;
                let l_hi = ((inst >> 16) & 0x7f) as i8;
                let l = l_lo | (l_hi << 1);

                handler.on_opcode_decoded(Opcode::Combine)?;
                handler.on_dest_decoded(Operand::gprpair(ddddd)?)?;
                handler.on_source_decoded(Operand::imm_i8(i))?;
                if min_op & 0b100 == 0 {
                    handler.on_source_decoded(Operand::imm_i8(l))?;
                } else {
                    // TODO: technically more restrictive than the manual - these bits are
                    // dontcare, not 0
                    let l = l as u8;
                    operand_check!(l & 0xc0 == 0);
                    handler.on_source_decoded(Operand::imm_u8(l))?;
                }
            }
            0b1101 => {
                return Err(DecodeError::InvalidOpcode);
            }
            0b1110 => {
                let iiii = reg_b16(inst) as i16;
                operand_check!(iiii & 0b1_0000 == 0);

                let ddddd = reg_b0(inst);

                handler.on_opcode_decoded(Opcode::TransferImmediate);

                let negated = (min_op >> 2) & 1 == 1;
                let pred_number = min_op & 0b11;
                let dotnew = inst >> 13 & 1 == 1;

                let i_lo = ((inst >> 5) & 0xff) as i16;
                let i_hi = iiii << 12 >> 4 as i16;
                let i = i_lo | i_hi;

                handler.inst_predicated(pred_number as u8, negated, dotnew)?;
                handler.on_dest_decoded(Operand::gpr(ddddd))?;
                handler.on_source_decoded(Operand::imm_i16(i))?;
            }
            0b1111 => {
                // 0b0111_1111_...
                handler.on_opcode_decoded(Opcode::Nop)?;
            }
            x => {
                unreachable!("impossible pattern");
            }
            }
        }
        0b1001 => {
            if (inst >> 27) & 1 != 0 {
                panic!("other mem op");
            }

            let ddddd = reg_b0(inst);
            let sssss = reg_b16(inst);
            let i_lo = (inst >> 5) & 0b1_1111_1111;
            let i_hi = (inst >> 25) & 0b11;
            let i = i_lo | (i_hi << 9);
            let op = (inst >> 21) & 0b1111;

            static SAMT: [u8; 16] = [
                0xff, 0x01, 0x00, 0x01,
                0x00, 0x02, 0xff, 0x02,
                0x03, 0x03, 0x03, 0x03,
                0x03, 0xff, 0x03, 0xff,
            ];

            handler.on_source_decoded(Operand::RegOffset { base: sssss, offset: (i as u32) << SAMT[op as usize] })?;
            handler.on_dest_decoded(Operand::Gpr { reg: ddddd })?;

            use Opcode::*;
            static OPCODES: [Option<Opcode>; 16] = [
                None,      Some(Membh), Some(MemhFifo), Some(Memubh),
                Some(MembFifo), Some(Memubh), None, Some(Membh),
                Some(Memb), Some(Memub), Some(Memh), Some(Memuh),
                Some(Memw), None, Some(Memd), None,
            ];
            handler.on_opcode_decoded(OPCODES[op as usize].ok_or(DecodeError::InvalidOpcode)?)?;
        }
        0b1010 => {
            eprintln!("TODO: 1010");
        }
        0b1011 => {
            handler.on_opcode_decoded(Opcode::Add)?;

            let ddddd = reg_b0(inst);
            let sssss = reg_b16(inst);

            let i_hi = (inst >> 21) & 0b111_1111;
            let i_lo = (inst >> 5) & 0b1_1111_1111;
            let i = (i_hi << 9) | i_lo;

            handler.on_dest_decoded(Operand::gpr(ddddd))?;
            handler.on_source_decoded(Operand::gpr(sssss))?;
            handler.on_source_decoded(Operand::imm_i16(i as i16))?;
        }
        _ => {
            eprintln!("iclass: {:04b}", iclass);
            // TODO: exhaustive
        }
    }

    Ok(())
}
