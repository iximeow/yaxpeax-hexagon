use core::fmt;

use crate::{Instruction, InstructionPacket, Opcode, Operand};

impl fmt::Display for InstructionPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{ ")?;
        write!(f, "{}", self.instructions[0])?;
        for i in 1..self.instruction_count {
            write!(f, "; {}", self.instructions[i as usize])?;
        }

        f.write_str(" }")?;
        if self.loop_effect.end_0() {
            f.write_str(":endloop0")?;
        }
        if self.loop_effect.end_1() {
            f.write_str(":endloop1")?;
        }

        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(predication) = self.flags.predicate {
            write!(f, "if ({}P{}{}) ",
                if predication.negated() { "!" } else { "" },
                predication.num(),
                if predication.pred_new() { ".new" } else { "" },
            )?;
        }

        // V73 Section 10.11
        // > The assembler encodes some Hexagon processor instructions as variants of other
        // > instructions. The encoding as a variant done for Operations that are functionally
        // > equivalent to other instructions, but are still defined as separate instructions because
        // > of their programming utility as common operations.
        // ...
        // | Instruction  | Mapping          |
        // |--------------|------------------|
        // | Rd = not(Rs) | Rd = sub(#-1,Rs) |
        // | Rd = neg(Rs) | Rd = sub(#0,Rs)  |
        // | Rdd = Rss    | Rdd = combine(Rss.H32, Rss.L32) |

        // stores put the mnemonic on LHS
        static STORES: &[Opcode] = &[
            Opcode::StoreMemb, Opcode::StoreMemh, Opcode::StoreMemw, Opcode::StoreMemd
        ];
        if STORES.contains(&self.opcode) {
            write!(f, "{}({}) = {}",
                self.opcode, self.dest.expect("TODO: unreachable; store has a destination"),
                self.sources[0]
            )?;
            return Ok(());
        }

        static JUMPS: &[Opcode] = &[
            Opcode::JumpEq, Opcode::JumpNeq, Opcode::JumpGt, Opcode::JumpLe,
            Opcode::JumpGtu, Opcode::JumpLeu, Opcode::JumpBitSet, Opcode::JumpBitClear,
        ];
        if JUMPS.contains(&self.opcode) {
            use crate::BranchHint;
            let hint_label = match self.flags.branch_hint.unwrap() {
                BranchHint::Taken => { "t" },
                BranchHint::NotTaken => { "nt" },
            };
            write!(f, "if ({}({}, {})) jump:{} {}",
                self.opcode.cmp_str().unwrap(), // TODO: unwrap_unchecked??
                self.sources[0],
                self.sources[1],
                hint_label,
                self.dest.unwrap(),
            )?;
            return Ok(());
        }
        if let Some(o) = self.dest.as_ref() {
            write!(f, "{} = ", o)?;
        }

        // TransferRegister and TransferImmediate display the source operand atypically.
        if self.opcode == Opcode::TransferRegister || self.opcode == Opcode::TransferImmediate {
            write!(f, "{}", self.sources[0])?;
            return Ok(());
        }

        if self.flags.negated {
            f.write_str("!")?;
        }
        write!(f, "{}", self.opcode)?;

        if self.sources_count > 0 {
            f.write_str("(")?;
            write!(f, "{}", self.sources[0])?;
            for i in 1..self.sources_count {
                write!(f, ", {}", self.sources[i as usize])?;
            }
            f.write_str(")")?;
        }

        if let Some(mode) = self.flags.rounded {
            write!(f, "{}", mode.as_label())?;
        }
        if self.flags.saturate {
            f.write_str(":sat")?;
        }
        Ok(())
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Opcode::BUG => { f.write_str("BUG") },
            Opcode::Nop => { f.write_str("nop") },
            Opcode::Jump => { f.write_str("jump") },
            Opcode::Call => { f.write_str("call") },
            Opcode::Memb => { f.write_str("memb") },
            Opcode::Memub => { f.write_str("memub") },
            Opcode::Memh => { f.write_str("memh") },
            Opcode::Memuh => { f.write_str("memuh") },
            Opcode::Memw => { f.write_str("memw") },
            Opcode::Memd => { f.write_str("memd") },

            Opcode::LoadMemb => { f.write_str("memb") },
            Opcode::LoadMemub => { f.write_str("memub") },
            Opcode::LoadMemh => { f.write_str("memh") },
            Opcode::LoadMemuh => { f.write_str("memuh") },
            Opcode::LoadMemw => { f.write_str("memw") },
            Opcode::LoadMemd => { f.write_str("memd") },

            Opcode::StoreMemb => { f.write_str("memb") },
            Opcode::StoreMemh => { f.write_str("memh") },
            Opcode::StoreMemw => { f.write_str("memw") },
            Opcode::StoreMemd => { f.write_str("memd") },

            Opcode::Membh => { f.write_str("membh") },
            Opcode::MemhFifo => { f.write_str("memh_fifo") },
            Opcode::Memubh => { f.write_str("memubh") },
            Opcode::MembFifo => { f.write_str("memb_fifo") },

            Opcode::Aslh => { f.write_str("aslh") },
            Opcode::Asrh => { f.write_str("asrh") },
            Opcode::TransferRegister => { f.write_str("transfer_register") },
            Opcode::TransferImmediate => { f.write_str("transfer_immediate") },
            Opcode::Zxtb => { f.write_str("zxtb") },
            Opcode::Sxtb => { f.write_str("sxtb") },
            Opcode::Zxth => { f.write_str("zxth") },
            Opcode::Sxth => { f.write_str("sxth") },
            Opcode::Mux => { f.write_str("mux") },
            Opcode::Combine => { f.write_str("combine") },
            Opcode::CmpEq => { f.write_str("cmp.eq") },
            Opcode::CmpGt => { f.write_str("cmp.gt") },
            Opcode::CmpGtu => { f.write_str("cmp.gtu") },
            Opcode::Add => { f.write_str("add") },
            Opcode::And => { f.write_str("and") },
            Opcode::Sub => { f.write_str("sub") },
            Opcode::Or => { f.write_str("or") },

            Opcode::JumpEq => { f.write_str("cmp.eq+jump") },
            Opcode::JumpNeq => { f.write_str("cmp.neq+jump") },
            Opcode::JumpGt => { f.write_str("cmp.gt+jump") },
            Opcode::JumpLe => { f.write_str("cmp.le+jump") },
            Opcode::JumpGtu => { f.write_str("cmp.gtu+jump") },
            Opcode::JumpLeu => { f.write_str("cmp.leu+jump") },
            Opcode::JumpBitSet => { f.write_str("tstbit+jump") },
            Opcode::JumpBitClear => { f.write_str("!tstbit+jump") },

            Opcode::Tlbw => { f.write_str("tlbw") },
            Opcode::Tlbr => { f.write_str("tlbr") },
            Opcode::Tlbp => { f.write_str("tlbp") },
            Opcode::TlbInvAsid => { f.write_str("tlbinvasid") },
            Opcode::Ctlbw => { f.write_str("ctlbw") },
            Opcode::Tlboc => { f.write_str("tlboc") },

            Opcode::Asr => { f.write_str("asr") },
            Opcode::Lsr => { f.write_str("lsr") },
            Opcode::Asl => { f.write_str("asl") },
            Opcode::Rol => { f.write_str("rol") },
            Opcode::Vsathub => { f.write_str("vsathub") },
            Opcode::Vsatwuh => { f.write_str("vsatwuh") },
            Opcode::Vsatwh => { f.write_str("vsatwh") },
            Opcode::Vsathb => { f.write_str("vsathb") },
            Opcode::Vasrh => { f.write_str("vasrh") },

            Opcode::Vabsh => { f.write_str("vabsh") },
            Opcode::Vabsw => { f.write_str("vabsw") },
            Opcode::Vasrw => { f.write_str("vasrw") },
            Opcode::Vlsrw => { f.write_str("vlsrw") },
            Opcode::Vaslw => { f.write_str("vaslw") },
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Nothing => {
                f.write_str("BUG (operand)")
            }
            Operand::PCRel32 { rel } => {
                write!(f, "#{}", rel)
            }
            Operand::Gpr { reg } => {
                write!(f, "R{}", reg)
            }
            Operand::Cr { reg } => {
                match reg {
                    9 => {
                        f.write_str("pc")
                    }
                    reg => {
                        write!(f, "C{}", reg)
                    }
                }
            }
            Operand::Sr { reg } => {
                write!(f, "S{}", reg)
            }
            Operand::GprNew { reg } => {
                write!(f, "R{}.new", reg)
            }
            Operand::GprLow { reg } => {
                write!(f, "R{}.L", reg)
            }
            Operand::GprHigh { reg } => {
                write!(f, "R{}.H", reg)
            }
            Operand::Gpr64b { reg_low } => {
                write!(f, "R{}:{}", reg_low + 1, reg_low)
            }
            Operand::Cr64b { reg_low } => {
                write!(f, "C{}:{}", reg_low + 1, reg_low)
            }
            Operand::Sr64b { reg_low } => {
                write!(f, "S{}:{}", reg_low + 1, reg_low)
            }
            Operand::PredicateReg { reg } => {
                write!(f, "P{}", reg)
            }
            Operand::RegOffset { base, offset } => {
                write!(f, "R{}+#{}", base, offset)
            }
            Operand::RegShiftedReg { base, index, shift } => {
                write!(f, "R{} + R{}<<{}", base, index, shift)
            }
            Operand::ImmU8 { imm } => {
                write!(f, "#0x{:x}", imm)
            }
            Operand::ImmU16 { imm } => {
                write!(f, "#0x{:x}", imm)
            }
            Operand::ImmI8 { imm } => {
                write!(f, "#{:}", imm)
            }
            Operand::ImmI16 { imm } => {
                write!(f, "#{:}", imm)
            }
            Operand::ImmI32 { imm } => {
                write!(f, "#{:}", imm)
            }
            Operand::ImmU32 { imm } => {
                write!(f, "#{:}", imm)
            }
        }
    }
}
