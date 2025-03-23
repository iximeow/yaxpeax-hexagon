use core::fmt;

use crate::{Instruction, InstructionPacket, Opcode, Operand};
use crate::{AssignMode, BranchHint, DomainHint};

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
        static REG_COMPARE_JUMPS: &[Opcode] = &[
            Opcode::JumpRegNz, Opcode::JumpRegGez,
            Opcode::JumpRegZ, Opcode::JumpRegLez,
        ];
        if REG_COMPARE_JUMPS.contains(&self.opcode) {
            return write!(f, "if ({}{}#0) jump{} {}",
                self.sources[0],
                match self.opcode {
                    Opcode::JumpRegZ => "==",
                    Opcode::JumpRegNz => "!=",
                    Opcode::JumpRegGez => ">=",
                    // JumpRegLez, the one other option
                    _ => "<=",
                },
                match self.flags.branch_hint {
                    Some(BranchHint::Taken) => { ":t" },
                    Some(BranchHint::NotTaken) => { ":nt" },
                    None => { "" },
                },
                self.dest.as_ref().unwrap()
            );
        }
        // handle cmp+jump first; this includes elements that would be misformatted below (like
        // predication)
        static COMPARE_JUMPS: &[Opcode] = &[
            Opcode::CmpEqJump, Opcode::CmpGtJump,
            Opcode::CmpGtuJump, Opcode::TestClrJump,
        ];
        if COMPARE_JUMPS.contains(&self.opcode) {
            let predicate = self.flags.predicate.as_ref().unwrap();
            let preg = Operand::pred(predicate.num());

            let hint_label = match self.flags.branch_hint.unwrap() {
                BranchHint::Taken => { "t" },
                BranchHint::NotTaken => { "nt" },
            };

            write!(f, "{} = {}({}, {}); if ({}{}.new) jump:{} {}",
                preg,
                self.opcode.cmp_str().unwrap(),
                self.sources[0],
                self.sources[1],
                if predicate.negated() { "!" } else { "" },
                preg,
                hint_label,
                self.dest.as_ref().unwrap(),
            )?;
            return Ok(());
        }

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
            Opcode::StoreMemb, Opcode::StoreMemh, Opcode::StoreMemw, Opcode::StoreMemd, Opcode::MemwRl, Opcode::MemdRl
        ];
        let mut needs_parens = self.sources_count > 0;
        if STORES.contains(&self.opcode) {
            let (assign_op, needs_parens) = match self.flags.assign_mode {
                None => ("=", false),
                Some(AssignMode::AddAssign) => ("+=", false),
                Some(AssignMode::SubAssign) => ("-=", false),
                Some(AssignMode::AndAssign) => ("&=", false),
                Some(AssignMode::OrAssign) => ("|=", false),
                Some(AssignMode::XorAssign) => ("^=", false),
                Some(AssignMode::ClrBit) => ("= clrbit", true),
                Some(AssignMode::SetBit) => ("= setbit", true),
            };
            write!(f, "{}({}){} {}{}{}{}",
                self.opcode, self.dest.expect("TODO: unreachable; store has a destination"),
                match self.flags.threads {
                    Some(DomainHint::Same) => { ":st" },
                    Some(DomainHint::All) => { ":at" },
                    None => { "" }
                },
                assign_op,
                if needs_parens { "(" } else { " " },
                self.sources[0],
                if needs_parens { ")" } else { "" },
            )?;
            return Ok(());
        }

        // TODO: do store conditionals have assign_merge?
        static SC_STORES: &[Opcode] = &[
            Opcode::MemwStoreCond, Opcode::MemdStoreCond,
        ];
        if SC_STORES.contains(&self.opcode) {
            write!(f, "{}({}, {}) = {}",
                self.opcode,
                self.dest.expect("TODO: unreachable; store has a destination"),
                self.alt_dest.expect("TODO: unreachable; store-conditional has a predicate reg"),
                self.sources[0]
            )?;
            return Ok(());
        }

        // TransferRegisterJump and TransferImmediateJump also have special display rules...
        if self.opcode == Opcode::TransferRegisterJump || self.opcode == Opcode::TransferImmediateJump {
            write!(f, "{} = {}; jump {}",
                self.alt_dest.as_ref().unwrap(),
                self.sources[0],
                self.dest.as_ref().unwrap(),
            )?;
            return Ok(());
        }

        static CONDITIONAL_JUMPS: &[Opcode] = &[
            Opcode::JumpEq, Opcode::JumpNeq, Opcode::JumpGt, Opcode::JumpLe,
            Opcode::JumpGtu, Opcode::JumpLeu, Opcode::JumpBitSet, Opcode::JumpBitClear,
        ];
        if CONDITIONAL_JUMPS.contains(&self.opcode) {
            let hint_label = match self.flags.branch_hint {
                Some(BranchHint::Taken) => { ":t" },
                Some(BranchHint::NotTaken) => { ":nt" },
                None => { "" },
            };
            write!(f, "if ({}({}, {})) jump{} {}",
                self.opcode.cmp_str().unwrap(), // TODO: unwrap_unchecked??
                self.sources[0],
                self.sources[1],
                hint_label,
                self.dest.unwrap(),
            )?;
            return Ok(());
        }
        static UNCONDITIONAL_BRANCHES: &[Opcode] = &[
            Opcode::Call, Opcode::Callr, Opcode::Callrh,
            Opcode::Jump, Opcode::Jumpr, Opcode::Jumprh,
        ];
        if UNCONDITIONAL_BRANCHES.contains(&self.opcode) {
            write!(f, "{}{} {}",
                self.opcode,
                match self.flags.branch_hint.as_ref() {
                    Some(BranchHint::Taken) => ":t",
                    Some(BranchHint::NotTaken) => ":nt",
                    None => "",
                },
                self.dest.unwrap())?;
            return Ok(());
        }

        if let Some(o) = self.dest.as_ref() {
            let (assign_op, p) = match self.flags.assign_mode {
                None => ("=", false),
                Some(AssignMode::AddAssign) => ("+=", false),
                Some(AssignMode::SubAssign) => ("-=", false),
                Some(AssignMode::AndAssign) => ("&=", false),
                Some(AssignMode::OrAssign) => ("|=", false),
                Some(AssignMode::XorAssign) => ("^=", false),
                Some(AssignMode::ClrBit) => ("=clrbit", true),
                Some(AssignMode::SetBit) => ("=setbit", true),
            };
            needs_parens |= p;
            write!(f, "{} {} ", o, assign_op)?;
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
        if needs_parens { f.write_str("(")?; }

        if self.opcode == Opcode::And_nRR || self.opcode == Opcode::Or_nRR {
            // while operand order does not matter here at all, the hexagon (v73) manual reverses
            // Rs and Rt for these specific instructions.
            write!(f, "{}, ~{}", self.sources[1], self.sources[0])?;
            if needs_parens { f.write_str(")")?; }
            return Ok(());
        }

        if self.opcode == Opcode::And_RnR || self.opcode == Opcode::Or_RnR {
            write!(f, "{}, ~{}", self.sources[0], self.sources[1])?;
            if needs_parens { f.write_str(")")?; }
            return Ok(());
        }

        if self.sources_count > 0 {
            write!(f, "{}", self.sources[0])?;
            for i in 1..self.sources_count {
                write!(f, ", {}", self.sources[i as usize])?;
            }
        }
        if needs_parens { f.write_str(")")?; }

        if let Some(mode) = self.flags.rounded {
            write!(f, "{}", mode.as_label())?;
        }
        if self.flags.chop {
            f.write_str(":chop")?;
        }
        if self.flags.saturate {
            f.write_str(":sat")?;
        }
        match self.flags.branch_hint {
            Some(BranchHint::Taken) => { f.write_str(":t")? },
            Some(BranchHint::NotTaken) => { f.write_str(":nt")? },
            None => {}
        }

        match self.flags.threads {
            Some(DomainHint::Same) => { f.write_str(":st")? },
            Some(DomainHint::All) => { f.write_str(":at")? },
            None => {}
        }

        // DeallocateFrame is shown with `:raw` as a suffix, but after the taken/not-taken hint
        // same for DeallocReturn
        if self.opcode == Opcode::AllocFrame || self.opcode == Opcode::DeallocFrame || self.opcode == Opcode::DeallocReturn {
            f.write_str(":raw")?;
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
            Opcode::Jumpr => { f.write_str("jumpr") },
            Opcode::Jumprh => { f.write_str("jumprh") },
            Opcode::Call => { f.write_str("call") },
            Opcode::Callr => { f.write_str("callr") },
            Opcode::Callrh => { f.write_str("callrh") },
            Opcode::Hintjr => { f.write_str("hintjr") },
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
            Opcode::And_nRR => { f.write_str("and") },
            Opcode::And_RnR => { f.write_str("and_RnR") },
            Opcode::Sub => { f.write_str("sub") },
            Opcode::Or => { f.write_str("or") },
            Opcode::Or_nRR => { f.write_str("or") },
            Opcode::Or_RnR => { f.write_str("or_RnR") },
            Opcode::Xor => { f.write_str("xor") },
            Opcode::Contains => { f.write_str("contains") },

            Opcode::JumpEq => { f.write_str("cmp.eq+jump") },
            Opcode::JumpNeq => { f.write_str("cmp.neq+jump") },
            Opcode::JumpGt => { f.write_str("cmp.gt+jump") },
            Opcode::JumpLe => { f.write_str("cmp.le+jump") },
            Opcode::JumpGtu => { f.write_str("cmp.gtu+jump") },
            Opcode::JumpLeu => { f.write_str("cmp.leu+jump") },
            Opcode::JumpBitSet => { f.write_str("tstbit+jump") },
            Opcode::JumpBitClear => { f.write_str("!tstbit+jump") },

            Opcode::CmpEqJump => { f.write_str("p=cmp.eq+if(p.new)jump") },
            Opcode::CmpGtJump => { f.write_str("p=cmp.gt+if(p.new)jump") },
            Opcode::CmpGtuJump => { f.write_str("p=cmp.gtu+if(p.new)jump") },
            Opcode::TestClrJump => { f.write_str("p=tstbit+if(p.new)jump") },

            Opcode::JumpRegZ => { f.write_str("if(rn==0)jump") },
            Opcode::JumpRegNz => { f.write_str("if(rn!=0)jump") },
            Opcode::JumpRegGez => { f.write_str("if(rn>=0)jump") },
            Opcode::JumpRegLez => { f.write_str("if(rn<=0)jump") },

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
            Opcode::Vlsrh => { f.write_str("vlsrh") },
            Opcode::Vaslw => { f.write_str("vaslw") },
            Opcode::Vaslh => { f.write_str("vaslh") },

            Opcode::Not => { f.write_str("not") },
            Opcode::Neg => { f.write_str("neg") },
            Opcode::Abs => { f.write_str("abs") },
            Opcode::Vconj => { f.write_str("vconj") },

            Opcode::Deinterleave => { f.write_str("deinterleave") },
            Opcode::Interleave => { f.write_str("interleave") },
            Opcode::Brev => { f.write_str("brev") },

            Opcode::ConvertDf2d => { f.write_str("convert_df2d") },
            Opcode::ConvertDf2ud => { f.write_str("convert_df2ud") },
            Opcode::ConvertUd2df => { f.write_str("convert_ud2df") },
            Opcode::ConvertD2df => { f.write_str("convert_d2df") },

            Opcode::Extractu => { f.write_str("extractu") },
            Opcode::Insert => { f.write_str("insert") },

            Opcode::TransferRegisterJump => { f.write_str("transferregisterjump") }
            Opcode::TransferImmediateJump => { f.write_str("transferimmediatejump") }

            Opcode::Trap0 => { f.write_str("trap0") }
            Opcode::Trap1 => { f.write_str("trap1") }
            Opcode::Pause => { f.write_str("pause") }
            Opcode::Icinva => { f.write_str("icinva") }
            Opcode::Isync => { f.write_str("isync") }
            Opcode::Unpause => { f.write_str("unpause") }

            Opcode::Vaddh => { f.write_str("vaddh") },
            Opcode::Vadduh => { f.write_str("vadduh") },
            Opcode::Vsubh => { f.write_str("vsubh") },
            Opcode::Vsubuh => { f.write_str("vsubuh") },
            Opcode::Vavgh => { f.write_str("vavgh") },
            Opcode::Vnavgh => { f.write_str("vnavgh") },
            Opcode::Packhl => { f.write_str("packhl") },

            Opcode::DcCleanA => { f.write_str("dccleana") },
            Opcode::DcInvA => { f.write_str("dcinva") },
            Opcode::DcCleanInvA => { f.write_str("dccleaninva") },
            Opcode::DcZeroA => { f.write_str("dczeroa") },
            Opcode::L2Fetch => { f.write_str("l2fetch") },
            Opcode::DmSyncHt => { f.write_str("dmsyncht") },
            Opcode::SyncHt => { f.write_str("syncht") },

            Opcode::Release => { f.write_str("release") },
            Opcode::Barrier => { f.write_str("barrier") },
            Opcode::AllocFrame => { f.write_str("allocframe") },
            Opcode::MemwRl => { f.write_str("memw_rl") },
            Opcode::MemdRl => { f.write_str("memd_rl") },

            Opcode::DeallocFrame => { f.write_str("deallocframe") },
            Opcode::DeallocReturn => { f.write_str("dealloc_return") },
            Opcode::Dcfetch => { f.write_str("dcfetch") },

            // LL/SC ops are distinguished by where they are in an instruction, not by their
            // textual representation
            Opcode::MemwLockedLoad => { f.write_str("memw_locked") },
            Opcode::MemwStoreCond => { f.write_str("memw_locked") },
            Opcode::MemwAq => { f.write_str("memw_aq") },
            // LL/SC ops are distinguished by where they are in an instruction, not by their
            // textual representation
            Opcode::MemdLockedLoad => { f.write_str("memd_locked") },
            Opcode::MemdStoreCond => { f.write_str("memd_locked") },
            Opcode::MemdAq => { f.write_str("memd_aq") },
            Opcode::Pmemcpy => { f.write_str("pmemcpy") },
            Opcode::Linecpy => { f.write_str("linecpy") },

            Opcode::Loop0 => { f.write_str("loop0") },
            Opcode::Loop1 => { f.write_str("loop1") },
            Opcode::Sp1Loop0 => { f.write_str("sp1loop0") },
            Opcode::Sp2Loop0 => { f.write_str("sp2loop0") },
            Opcode::Sp3Loop0 => { f.write_str("sp3loop0") },
            Opcode::Trace => { f.write_str("trace") },
            Opcode::Diag => { f.write_str("diag") },
            Opcode::Diag0 => { f.write_str("diag0") },
            Opcode::Diag1 => { f.write_str("diag1") },

            Opcode::Movlen => { f.write_str("movlen") },
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
                write!(f, "$+#{}", rel)
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
            Operand::RegShiftOffset { base, shift, offset } => {
                write!(f, "R{}<<{} + {:#x}", base, shift, offset)
            }
            Operand::RegOffsetInc { base, offset } => {
                write!(f, "R{}++#{:#x}", base, offset)
            }
            Operand::RegOffsetCirc { base, offset, mu } => {
                write!(f, "R{}++#{:#x}:circ(M{})", base, offset, mu)
            }
            Operand::RegCirc { base, mu } => {
                write!(f, "R{}++I:circ(M{})", base, mu)
            }
            Operand::RegMemIndexed { base, mu } => {
                write!(f, "R{}++M{}", base, mu)
            }
            Operand::RegMemIndexedBrev { base, mu } => {
                write!(f, "R{}++M{}:brev", base, mu)
            }
            Operand::RegStoreAssign { base, addr } => {
                write!(f, "R{}=#{:#x}", base, addr)
            }
            Operand::Absolute { addr } => {
                write!(f, "#{:#x}", addr)
            }
            Operand::GpOffset { offset } => {
                write!(f, "gp+#{:#x}", offset)
            }
        }
    }
}
