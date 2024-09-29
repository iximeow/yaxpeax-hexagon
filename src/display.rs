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
        if let Some(predication) = self.predicate {
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
        if let Some(o) = self.dest.as_ref() {
            write!(f, "{} = ", o)?;
        }

        // TransferRegister and TransferImmediate display the source operand atypically.
        if self.opcode == Opcode::TransferRegister || self.opcode == Opcode::TransferImmediate {
            write!(f, "{}", self.sources[0])?;
            return Ok(());
        }

        if self.negated {
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

        Ok(())
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Opcode::BUG => { f.write_str("BUG") },
            Opcode::Nop => { f.write_str("nop") },
            Opcode::Jump => { f.write_str("jump") },
            Opcode::Memb => { f.write_str("memb") },
            Opcode::Memub => { f.write_str("memub") },
            Opcode::Memh => { f.write_str("memh") },
            Opcode::Memuh => { f.write_str("memuh") },
            Opcode::Memw => { f.write_str("memw") },
            Opcode::Memd => { f.write_str("memd") },
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
                f.write_str("idk!")
            }
            Operand::Gpr { reg } => {
                write!(f, "R{}", reg)
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
            Operand::PredicateReg { reg } => {
                write!(f, "P{}", reg)
            }
            Operand::RegOffset { base, offset } => {
                write!(f, "R{}+#{}", base, offset)
            }
            Operand::RegShiftedReg { base, index, shift } => {
                write!(f, "R{} + R{} << {}", base, index, shift)
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
        }
    }
}
