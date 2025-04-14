use core::fmt;

use crate::{Instruction, InstructionPacket, Opcode, Operand};
use crate::{AssignMode, BranchHint, DomainHint, RawMode};

fn special_display_rules(op: &Opcode) -> bool {
    *op as u16 & 0x8000 == 0x8000
}

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

fn assign_mode_label(mode: Option<AssignMode>) -> &'static str {
    match mode {
        None => "=",
        Some(AssignMode::AddAssign) => "+=",
        Some(AssignMode::SubAssign) => "-=",
        Some(AssignMode::AndAssign) => "&=",
        Some(AssignMode::OrAssign) => "|=",
        Some(AssignMode::XorAssign) => "^=",
        Some(AssignMode::ClrBit) => "BUG",
        Some(AssignMode::SetBit) => "BUG",
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if special_display_rules(&self.opcode) {
            match self.opcode {
                Opcode::AndAnd => {
                    return write!(f, "{} = and({}, and({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::AndOr => {
                    return write!(f, "{} = and({}, or({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::OrAnd => {
                    return write!(f, "{} = or({}, and({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::AndNot => {
                    // not entirely sure if these should be the same opcode, but "not" as in
                    // "bitswise negation" as in "! for boolean predicates and ~ for integer
                    // registers" seems... ok?
                    if let Some(dest @ Operand::PredicateReg { .. }) = self.dest.as_ref() {
                        return write!(f, "{} {} and({}, !{})", dest,
                            assign_mode_label(self.flags.assign_mode),
                            self.sources[0], self.sources[1]);
                    } else {
                        return write!(f, "{} {} and({}, ~{})", self.dest.as_ref().unwrap(),
                            assign_mode_label(self.flags.assign_mode),
                            self.sources[0], self.sources[1]);
                    }
                }
                Opcode::OrOr => {
                    return write!(f, "{} = or({}, or({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::AndAndNot => {
                    return write!(f, "{} = and({}, and({}, !{}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::AndOrNot => {
                    return write!(f, "{} = and({}, or({}, !{}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::OrAndNot => {
                    return write!(f, "{} = or({}, and({}, !{}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::OrNot => {
                    return write!(f, "{} {} or({}, !{})", self.dest.as_ref().unwrap(),
                        assign_mode_label(self.flags.assign_mode),
                        self.sources[0], self.sources[1]);
                }
                Opcode::OrOrNot => {
                    return write!(f, "{} = or({}, or({}, !{}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                }
                Opcode::AndLsr => {
                    return write!(f, "{} = and({}, lsr({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::OrLsr => {
                    return write!(f, "{} = or({}, lsr({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddLsr => {
                    return write!(f, "{} = add({}, lsr({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::SubLsr => {
                    return write!(f, "{} = sub({}, lsr({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddLsl => {
                    return write!(f, "{} = add({}, lsl({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddAsl => {
                    return write!(f, "{} = add({}, asl({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::SubAsl => {
                    return write!(f, "{} = sub({}, asl({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AndAsl => {
                    return write!(f, "{} = and({}, asl({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::OrAsl => {
                    return write!(f, "{} = or({}, asl({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddAdd => {
                    return write!(f, "{} = add({}, add({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddSub => {
                    return write!(f, "{} = add({}, sub({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::AddMpyi => {
                    return write!(f, "{} = add({}, mpyi({}, {}))", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1], self.sources[2]);
                },
                Opcode::MpyiPos => {
                    return write!(f, "{} = +mpyi({}, {})", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1]);
                },
                Opcode::MpyiNeg => {
                    return write!(f, "{} = -mpyi({}, {})", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1]);
                },
                Opcode::AddClb => {
                    return write!(f, "{} = add(clb({}), {})", self.dest.as_ref().unwrap(),
                        self.sources[0], self.sources[1]);
                }
                Opcode::Vacsh => {
                    return write!(f, "{}, {} = {}({}, {})",
                        self.dest.as_ref().unwrap(), self.alt_dest.as_ref().unwrap(),
                        self.opcode, self.sources[0], self.sources[1]);
                }
                Opcode::Vminub => {
                    if let Some(alt_dest) = self.alt_dest.as_ref() {
                        return write!(f, "{}, {} = {}({}, {})",
                            self.dest.as_ref().unwrap(), alt_dest,
                            self.opcode, self.sources[0], self.sources[1]);
                    } else {
                        return write!(f, "{} = {}({}, {})",
                            self.dest.as_ref().unwrap(),
                            self.opcode, self.sources[0], self.sources[1]);
                    }
                }
                Opcode::SfRecipa => {
                    return write!(f, "{}, {} = {}({}, {})",
                        self.dest.as_ref().unwrap(), self.alt_dest.as_ref().unwrap(),
                        self.opcode, self.sources[0], self.sources[1]);
                }
                Opcode::SfInvsqrta => {
                    return write!(f, "{}, {} = {}({})",
                        self.dest.as_ref().unwrap(), self.alt_dest.as_ref().unwrap(),
                        self.opcode, self.sources[0]);
                }
                Opcode::Any8VcmpbEq => {
                    return write!(f, "{} = {}any8(vcmpb.eq({}, {}))",
                        self.dest.as_ref().unwrap(),
                        if self.flags.negated { "!" } else { "" },
                        self.sources[0], self.sources[1]);
                }
                _ => {
                    unreachable!("should be exhaustive for opcodes with special display rules");
                }
            }
        }
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
            write!(f, "if ({}p{}{}) ",
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
                self.opcode, self.dest.expect("unreachable; store has a destination"),
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

        static SC_STORES: &[Opcode] = &[
            Opcode::MemwStoreCond, Opcode::MemdStoreCond,
        ];
        if SC_STORES.contains(&self.opcode) {
            write!(f, "{}({}, {}) = {}",
                self.opcode,
                self.dest.expect("unreachable; store has a destination"),
                self.alt_dest.expect("unreachable; store-conditional has a predicate reg"),
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
                self.opcode.cmp_str().unwrap(), // (obvious, but) decoder bug if this fails
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

        // ... vxaddsubh has the right shift after round, but cmpyiwh and friends have the left
        // shift before round. cmpyiwh and add/sub `:sat:<<16` forms have the shift in different
        // positions. awful awful awful.
        if self.opcode == Opcode::Add || self.opcode == Opcode::Sub {
            if let Some(mode) = self.flags.rounded {
                write!(f, "{}", mode.as_label())?;
            }
            if self.flags.chop {
                f.write_str(":chop")?;
            }
            if self.flags.carry {
                f.write_str(":carry")?;
            }
            if self.flags.saturate {
                f.write_str(":sat")?;
            }
            if let Some(shift) = self.flags.shift_right {
                write!(f, ":>>{}", shift)?;
            }
            if let Some(shift) = self.flags.shift_left {
                write!(f, ":<<{}", shift)?;
            }
        } else {
            if let Some(shift) = self.flags.shift_left {
                write!(f, ":<<{}", shift)?;
            }
            if let Some(mode) = self.flags.rounded {
                write!(f, "{}", mode.as_label())?;
            }
            if self.flags.chop {
                f.write_str(":chop")?;
            }
            if let Some(shift) = self.flags.shift_right {
                write!(f, ":>>{}", shift)?;
            }
            if self.flags.carry {
                f.write_str(":carry")?;
            }
            if self.flags.saturate {
                f.write_str(":sat")?;
            }
        }
        if self.flags.scale {
            f.write_str(":scale")?;
        }
        if self.flags.library {
            f.write_str(":lib")?;
        }
        if self.flags.deprecated {
            f.write_str(":deprecated")?;
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

        match self.flags.raw_mode {
            Some(RawMode::Lo) => { f.write_str(":raw:lo")? },
            Some(RawMode::Hi) => { f.write_str(":raw:hi")? },
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
            Opcode::CmpbEq => { f.write_str("cmpb.eq") },
            Opcode::CmphEq => { f.write_str("cmph.eq") },
            Opcode::CmpGt => { f.write_str("cmp.gt") },
            Opcode::CmpbGt => { f.write_str("cmpb.gt") },
            Opcode::CmphGt => { f.write_str("cmph.gt") },
            Opcode::CmpGtu => { f.write_str("cmp.gtu") },
            Opcode::CmpbGtu => { f.write_str("cmpb.gtu") },
            Opcode::CmphGtu => { f.write_str("cmph.gtu") },
            Opcode::Add => { f.write_str("add") },
            Opcode::And => { f.write_str("and") },
            Opcode::And_nRR => { f.write_str("and") },
            Opcode::And_RnR => { f.write_str("and") },
            Opcode::Sub => { f.write_str("sub") },
            Opcode::Or => { f.write_str("or") },
            Opcode::Or_nRR => { f.write_str("or") },
            Opcode::Or_RnR => { f.write_str("or") },
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
            Opcode::Lsl => { f.write_str("lsl") },
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
            Opcode::Vlslw => { f.write_str("vlslw") },
            Opcode::Vlslh => { f.write_str("vlslh") },

            Opcode::Not => { f.write_str("not") },
            Opcode::Neg => { f.write_str("neg") },
            Opcode::Abs => { f.write_str("abs") },
            Opcode::Vconj => { f.write_str("vconj") },

            Opcode::Deinterleave => { f.write_str("deinterleave") },
            Opcode::Interleave => { f.write_str("interleave") },
            Opcode::Brev => { f.write_str("brev") },

            Opcode::Extractu => { f.write_str("extractu") },
            Opcode::Extract => { f.write_str("extract") },
            Opcode::Insert => { f.write_str("insert") },

            Opcode::TransferRegisterJump => { f.write_str("transferregisterjump") }
            Opcode::TransferImmediateJump => { f.write_str("transferimmediatejump") }

            Opcode::Trap0 => { f.write_str("trap0") }
            Opcode::Trap1 => { f.write_str("trap1") }
            Opcode::Pause => { f.write_str("pause") }
            Opcode::Icinva => { f.write_str("icinva") }
            Opcode::Isync => { f.write_str("isync") }
            Opcode::Unpause => { f.write_str("unpause") }

            Opcode::SfAdd => { f.write_str("sfadd") },
            Opcode::SfSub => { f.write_str("sfsub") },
            Opcode::SfMpy => { f.write_str("sfmpy") },
            Opcode::SfMax => { f.write_str("sfmax") },
            Opcode::SfMin => { f.write_str("sfmin") },

            Opcode::DfAdd => { f.write_str("dfadd") },
            Opcode::DfSub => { f.write_str("dfsub") },
            Opcode::DfMax => { f.write_str("dfmax") },
            Opcode::DfMin => { f.write_str("dfmin") },

            Opcode::SfCmpEq => { f.write_str("sfcmp.eq") },
            Opcode::SfCmpGt => { f.write_str("sfcmp.gt") },
            Opcode::SfCmpGe => { f.write_str("sfcmp.ge") },
            Opcode::SfCmpUo => { f.write_str("sfcmp.uo") },

            Opcode::DfCmpEq => { f.write_str("dfcmp.eq") },
            Opcode::DfCmpGt => { f.write_str("dfcmp.gt") },
            Opcode::DfCmpGe => { f.write_str("dfcmp.ge") },
            Opcode::DfCmpUo => { f.write_str("dfcmp.uo") },
            Opcode::DfMpyll => { f.write_str("dfmpyll") },
            Opcode::DfMpylh => { f.write_str("dfmpylh") },
            Opcode::DfMpyhh => { f.write_str("dfmpyhh") },
            Opcode::DfMpyfix => { f.write_str("dfmpyfix") },

            Opcode::Cmpy => { f.write_str("cmpy") },
            Opcode::Cmpyiw => { f.write_str("cmpyiw") },
            Opcode::Cmpyrw => { f.write_str("cmpyrw") },
            Opcode::Cmpyiwh => { f.write_str("cmpyiwh") },
            Opcode::Cmpyrwh => { f.write_str("cmpyrwh") },
            Opcode::Cmpyi => { f.write_str("cmpyi") },
            Opcode::Cmpyr => { f.write_str("cmpyr") },
            Opcode::Vacsh => { f.write_str("vacsh") },
            Opcode::Vcmpyi => { f.write_str("vcmpyi") },
            Opcode::Vcmpyr => { f.write_str("vcmpyr") },
            Opcode::Vmpybu => { f.write_str("vmpybu") },
            Opcode::Vmpyh => { f.write_str("vmpyh") },
            Opcode::Vmpyhsu => { f.write_str("vmpyhsu") },
            Opcode::Vmpybsu => { f.write_str("vmpybsu") },
            Opcode::Vrmpybsu => { f.write_str("vrmpybsu") },
            Opcode::Vdmpybsu => { f.write_str("vdmpybsu") },
            Opcode::Vrmpybu => { f.write_str("vrmpybu") },
            Opcode::Vrmpyh => { f.write_str("vrmpyh") },
            Opcode::Vraddub => { f.write_str("vraddub") },
            Opcode::Vraddh => { f.write_str("vraddh") },
            Opcode::Vradduh => { f.write_str("vradduh") },
            Opcode::Vrsadub => { f.write_str("vrsadub") },
            Opcode::Vaddub => { f.write_str("vaddub") },
            Opcode::Vsubub => { f.write_str("vsubub") },
            Opcode::Vaddhub => { f.write_str("vaddhub") },
            Opcode::Vaddw => { f.write_str("vaddw") },
            Opcode::Vsubw => { f.write_str("vsubw") },
            Opcode::Vavgub => { f.write_str("vavgub") },
            Opcode::Vavgw => { f.write_str("vavgw") },
            Opcode::Vavguw => { f.write_str("vavguw") },
            Opcode::Vavguh => { f.write_str("vavguh") },
            Opcode::Max => { f.write_str("max") },
            Opcode::Maxu => { f.write_str("maxu") },
            Opcode::Min => { f.write_str("min") },
            Opcode::Minu => { f.write_str("minu") },
            Opcode::Vcrotate => { f.write_str("vcrotate") },
            Opcode::Vtrunowh => { f.write_str("vtrunowh") },
            Opcode::Vtrunewh => { f.write_str("vtrunewh") },
            Opcode::Vmaxb => { f.write_str("vmaxb") },
            Opcode::Vmaxub => { f.write_str("vmaxub") },
            Opcode::Vminb => { f.write_str("vminb") },
            Opcode::Vminub => { f.write_str("vminub") },
            Opcode::Vminuw => { f.write_str("vminuw") },
            Opcode::Vminh => { f.write_str("vminh") },
            Opcode::Vminuh => { f.write_str("vminuh") },
            Opcode::Vminw => { f.write_str("vminw") },
            Opcode::Vmaxw => { f.write_str("vmaxw") },
            Opcode::Vmaxuw => { f.write_str("vmaxuw") },
            Opcode::Vmaxh => { f.write_str("vmaxh") },
            Opcode::Vmaxuh => { f.write_str("vmaxuh") },
            Opcode::Vnegh => { f.write_str("vnegh") },
            Opcode::Vcnegh => { f.write_str("vcnegh") },

            Opcode::Pmpyw => { f.write_str("pmpyw") },
            Opcode::Vpmpyh => { f.write_str("vpmpyh") },
            Opcode::Lfs => { f.write_str("lfs") },

            Opcode:: Vxaddsubh => { f.write_str("vxaddsubh") },
            Opcode::Vxaddsubw => { f.write_str("vxaddsubw") },
            Opcode::Vxsubaddh => { f.write_str("vxsubaddh") },
            Opcode::Vxsubaddw => { f.write_str("vxsubaddw") },

            Opcode::Vaddh => { f.write_str("vaddh") },
            Opcode::Vadduh => { f.write_str("vadduh") },
            Opcode::Vsubh => { f.write_str("vsubh") },
            Opcode::Vsubuh => { f.write_str("vsubuh") },
            Opcode::Vavgh => { f.write_str("vavgh") },
            Opcode::Vnavgh => { f.write_str("vnavgh") },
            Opcode::Vnavgw => { f.write_str("vnavgw") },
            Opcode::Packhl => { f.write_str("packhl") },

            Opcode::DcCleanA => { f.write_str("dccleana") },
            Opcode::DcInvA => { f.write_str("dcinva") },
            Opcode::DcCleanInvA => { f.write_str("dccleaninva") },
            Opcode::DcZeroA => { f.write_str("dczeroa") },
            Opcode::DcKill => { f.write_str("dckill") },
            Opcode::IcKill => { f.write_str("ickill") },
            Opcode::L2Fetch => { f.write_str("l2fetch") },
            Opcode::L2Kill => { f.write_str("l2kill") },
            Opcode::L2Gunlock => { f.write_str("l2gunlock") },
            Opcode::L2Gclean => { f.write_str("l2gclean") },
            Opcode::L2Gcleaninv => { f.write_str("l2gcleaninv") },
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
            Opcode::Fastcorner9 => { f.write_str("fastcorner9") },
            Opcode::AndAnd => { f.write_str("andand") },
            Opcode::AndOr => { f.write_str("andor") },
            Opcode::OrAnd => { f.write_str("orand") },
            Opcode::AndNot => { f.write_str("andnot") },
            Opcode::OrOr => { f.write_str("oror") },
            Opcode::AndAndNot => { f.write_str("andandnot") },
            Opcode::AndOrNot => { f.write_str("andornot") },
            Opcode::OrAndNot => { f.write_str("orandnot") },
            Opcode::OrNot => { f.write_str("ornot") },
            Opcode::OrOrNot => { f.write_str("orornot") },
            Opcode::AndLsr => { f.write_str("andlsr") },
            Opcode::OrLsr => { f.write_str("orlsr") },
            Opcode::OrAsl => { f.write_str("orasl") },
            Opcode::AddLsr => { f.write_str("addlsr") },
            Opcode::SubLsr => { f.write_str("sublsr") },
            Opcode::AddLsl => { f.write_str("addlsl") },
            Opcode::AddAsl => { f.write_str("addasl") },
            // this is the form shown literally as `addasl`, in contrast to the form above which
            // shouldn't ever really be printed as `addasl` (instruction display has more complex
            // rules here.
            Opcode::AddAslRegReg => { f.write_str("addasl") },
            Opcode::Swi => { f.write_str("swi") },
            Opcode::Cswi => { f.write_str("cswi") },
            Opcode::Ciad => { f.write_str("ciad") },
            Opcode::Wait => { f.write_str("wait") },
            Opcode::Resume => { f.write_str("resume") },
            Opcode::Stop => { f.write_str("stop") },
            Opcode::Start => { f.write_str("start") },
            Opcode::Nmi => { f.write_str("nmi") },
            Opcode::Setimask => { f.write_str("setimask") },
            Opcode::Siad => { f.write_str("siad") },
            Opcode::Brkpt => { f.write_str("brkpt") },
            Opcode::TlbLock => { f.write_str("tlblock") },
            Opcode::K0Lock => { f.write_str("k0lock") },
            Opcode::Crswap => { f.write_str("crswap") },
            Opcode::Getimask => { f.write_str("getimask") },
            Opcode::Iassignr => { f.write_str("iassignr") },
            Opcode::Icdatar => { f.write_str("icdatar") },
            Opcode::Ictagr => { f.write_str("ictagr") },
            Opcode::Ictagw => { f.write_str("ictagw") },
            Opcode::Icinvidx => { f.write_str("icinvidx") },

            Opcode::SubAsl => { f.write_str("subasl") },
            Opcode::AndAsl => { f.write_str("andasl") },
            Opcode::AddClb => { f.write_str("addclb") },
            Opcode::AddAdd => { f.write_str("addadd") },
            Opcode::AddSub => { f.write_str("addsub") },
            Opcode::AddMpyi => { f.write_str("addmpyi") },
            Opcode::Any8 => { f.write_str("any8") },
            Opcode::All8 => { f.write_str("all8") },
            Opcode::Valignb => { f.write_str("valignb") },
            Opcode::Vspliceb => { f.write_str("vspliceb") },
            Opcode::Vsxtbh => { f.write_str("vsxtbh") },
            Opcode::Vzxtbh => { f.write_str("vzxtbh") },
            Opcode::Vsxthw => { f.write_str("vsxthw") },
            Opcode::Vzxthw => { f.write_str("vzxthw") },
            Opcode::Vsplatb => { f.write_str("vsplatb") },
            Opcode::Vsplath => { f.write_str("vsplath") },
            Opcode::Vrcrotate => { f.write_str("vrcrotate") },
            Opcode::Vrmaxh => { f.write_str("vrmaxh") },
            Opcode::Vrmaxw => { f.write_str("vrmaxw") },
            Opcode::Vrminh => { f.write_str("vrminh") },
            Opcode::Vrminw => { f.write_str("vrminw") },
            Opcode::Vrmaxuh => { f.write_str("vrmaxuh") },
            Opcode::Vrmaxuw => { f.write_str("vrmaxuw") },
            Opcode::Vrminuh => { f.write_str("vrminuh") },
            Opcode::Vrminuw => { f.write_str("vrminuw") },
            Opcode::Vrcnegh => { f.write_str("vrcnegh") },
            Opcode::Vabsdiffb => { f.write_str("vabsdiffb") },
            Opcode::Vabsdiffub => { f.write_str("vabsdiffub") },
            Opcode::Vabsdiffh => { f.write_str("vabsdiffh") },
            Opcode::Vabsdiffw => { f.write_str("vabsdiffw") },
            Opcode::ConvertDf2D => { f.write_str("convert_df2d") },
            Opcode::ConvertDf2Ud => { f.write_str("convert_df2ud") },
            Opcode::ConvertUd2Df => { f.write_str("convert_ud2df") },
            Opcode::ConvertD2Df => { f.write_str("convert_d2df") },
            Opcode::ConvertD2Sf => { f.write_str("convert_d2sf") },
            Opcode::ConvertSf2Df => { f.write_str("convert_sf2df") },
            Opcode::ConvertDf2Sf => { f.write_str("convert_df2sf") },
            Opcode::ConvertUw2Sf => { f.write_str("convert_uw2sf") },
            Opcode::ConvertUw2Df => { f.write_str("convert_uw2df") },
            Opcode::ConvertUd2Sf => { f.write_str("convert_ud2sf") },
            Opcode::ConvertW2Sf => { f.write_str("convert_w2sf") },
            Opcode::ConvertW2Df => { f.write_str("convert_w2df") },
            Opcode::ConvertSf2Uw => { f.write_str("convert_sf2uw") },
            Opcode::ConvertSf2Ud => { f.write_str("convert_sf2ud") },
            Opcode::ConvertSf2W => { f.write_str("convert_sf2w") },
            Opcode::ConvertSf2D => { f.write_str("convert_sf2d") },
            Opcode::Mask => { f.write_str("mask") },
            Opcode::Setbit => { f.write_str("setbit") },
            Opcode::Clrbit => { f.write_str("clrbit") },
            Opcode::Tstbit => { f.write_str("tstbit") },
            Opcode::Togglebit => { f.write_str("togglebit") },
            Opcode::Bitsclr => { f.write_str("bitsclr") },
            Opcode::Bitsset => { f.write_str("bitsset") },
            Opcode::Modwrap => { f.write_str("modwrap") },
            Opcode::SfClass => { f.write_str("sfclass") },
            Opcode::DfClass => { f.write_str("dfclass") },
            Opcode::SfMake => { f.write_str("sfmake") },
            Opcode::DfMake => { f.write_str("dfmake") },
            Opcode::Tableidxb => { f.write_str("tableidxb") },
            Opcode::Tableidxh => { f.write_str("tableidxh") },
            Opcode::Tableidxw => { f.write_str("tableidxw") },
            Opcode::Tableidxd => { f.write_str("tableidxd") },
            Opcode::Vasrhub => { f.write_str("vasrhub") },
            Opcode::Vrndwh => { f.write_str("vrndwh") },
            Opcode::Vtrunohb => { f.write_str("vtrunohb") },
            Opcode::Vtrunehb => { f.write_str("vtrunehb") },
            Opcode::Normamt => { f.write_str("normamt") },
            Opcode::Popcount => { f.write_str("popcount") },
            Opcode::Sat => { f.write_str("sat") },
            Opcode::Satb => { f.write_str("satb") },
            Opcode::Sath => { f.write_str("sath") },
            Opcode::Satub => { f.write_str("satub") },
            Opcode::Satuh => { f.write_str("satuh") },
            Opcode::Round => { f.write_str("round") },
            Opcode::Cround => { f.write_str("cround") },
            Opcode::Bitsplit => { f.write_str("bitsplit") },
            Opcode::Clip => { f.write_str("clip") },
            Opcode::Vclip => { f.write_str("vclip") },
            Opcode::Clb => { f.write_str("clb") },
            Opcode::Cl0 => { f.write_str("cl0") },
            Opcode::Cl1 => { f.write_str("cl1") },
            Opcode::Ct0 => { f.write_str("ct0") },
            Opcode::Ct1 => { f.write_str("ct1") },
            Opcode::Vitpack => { f.write_str("vitpack") },
            Opcode::SfFixupr => { f.write_str("sffixupr") },
            Opcode::SfFixupn => { f.write_str("sffixupn") },
            Opcode::SfFixupd => { f.write_str("sffixupd") },
            Opcode::SfRecipa => { f.write_str("sfrecipa") },
            Opcode::Swiz => { f.write_str("swiz") },
            Opcode::Shuffeb => { f.write_str("shuffeb") },
            Opcode::Shuffob => { f.write_str("shuffob") },
            Opcode::Shuffeh => { f.write_str("shuffeh") },
            Opcode::Shuffoh => { f.write_str("shuffoh") },
            Opcode::Decbin => { f.write_str("decbin") },
            Opcode::SfInvsqrta => { f.write_str("sfinvsqrta") },
            Opcode::Parity => { f.write_str("parity") },
            Opcode::Tlbmatch => { f.write_str("tlbmatch") },
            Opcode::Boundscheck => { f.write_str("boundscheck") },
            Opcode::Any8VcmpbEq => { f.write_str("any8vcmpbeq") },
            Opcode::Vmux => { f.write_str("vmux") },
            Opcode::VcmpwEq => { f.write_str("vcmpw.eq") },
            Opcode::VcmpwGt => { f.write_str("vcmpw.gt") },
            Opcode::VcmpwGtu => { f.write_str("vcmpw.gtu") },
            Opcode::VcmphEq => { f.write_str("vcmph.eq") },
            Opcode::VcmphGt => { f.write_str("vcmph.gt") },
            Opcode::VcmphGtu => { f.write_str("vcmph.gtu") },
            Opcode::VcmpbEq => { f.write_str("vcmpb.eq") },
            Opcode::VcmpbGt => { f.write_str("vcmpb.gt") },
            Opcode::VcmpbGtu => { f.write_str("vcmpb.gtu") },
            Opcode::Mpy => { f.write_str("mpy") },
            Opcode::Mpyu => { f.write_str("mpyu") },
            Opcode::Mpyi => { f.write_str("mpyi") },
            Opcode::MpyiNeg => { f.write_str("mpyineg") },
            Opcode::MpyiPos => { f.write_str("mpyipos") },
            Opcode::Mpysu => { f.write_str("mpysu") },
            Opcode::Vcmpyh => { f.write_str("vcmpyh") },
            Opcode::Vrcmpys => { f.write_str("vrcmpys") },
            Opcode::Vdmpy => { f.write_str("vdmpy") },
            Opcode::Vmpyeh => { f.write_str("vmpyeh") },
            Opcode::Vmpyweh => { f.write_str("vmpyweh") },
            Opcode::Vmpywoh => { f.write_str("vmpywoh") },
            Opcode::Vrmpyweh => { f.write_str("vrmpyweh") },
            Opcode::Vrmpywoh => { f.write_str("vrmpywoh") },
            Opcode::Vrmpyu => { f.write_str("vrmpyu") },
            Opcode::Vrmpysu => { f.write_str("vrmpysu") },
            Opcode::Vmpyweuh => { f.write_str("vmpyweuh") },
            Opcode::Vmpywouh => { f.write_str("vmpywouh") },
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
                if *rel >= 0 {
                    write!(f, "$+{:#x}", *rel)
                } else {
                    write!(f, "$-{:#x}", -*rel)
                }
            }
            Operand::Gpr { reg } => {
                const NAMES: [&'static str; 32] = [
                    "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7",
                    "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
                    "r16", "r17", "r18", "r19", "r20", "r21", "r22", "r23",
                    "r24", "r25", "r26", "r27",
                    // the three r29 through r31 general registers support subroutines and the Software
                    // Stack. ... they have symbol aliases that indicate when these registers are accessed
                    // as subroutine and stack registers (V73 Section 2.1)
                    "r28", "sp", "fp", "lr",
                ];

                f.write_str(NAMES[*reg as usize])
            }
            Operand::Cr { reg } => {
                // V69 Table 2-2 Aliased control registers
                static CR_NAMES: [&'static str; 32] = [
                    "sa0", "lc0", "sa1", "lc1",
                    "p3:0", "c5", "m0", "m1",
                    "usr", "pc", "ugp", "gp",
                    "cs0", "cs1", "upcyclelo", "upcyclehi",
                    "framelimit", "framekey", "pktcountlo", "pktcounthi",
                    "c20", "c21", "c22", "c23",
                    "c24", "c25", "c26", "c27",
                    "c28", "c29", "utimerlo", "utimerhi",
                ];
                f.write_str(CR_NAMES[*reg as usize])
            }
            Operand::Sr { reg } => {
                // V62 "System control register transfer" includes this table
                static SR_NAMES: [&'static str; 64] = [
                    "sgp0", "sgp1", "stid", "elr",
                    "badva0", "badva1", "ssr", "ccr",
                    "htid", "badva", "imask", "S11",
                    "S12", "S13", "S14", "S15",
                    "evb", "modectl", "syscfg", "S19",
                    "ipend", "vid", "iad", "S23",
                    "iel", "S25", "iahl", "cfgbase",
                    "diag", "rev", "pcyclelo", "pcyclehi",
                    "isdbst", "isdbcfg0", "isdbcfg1", "S35",
                    "brkptpc0", "brkptcfg0", "brkptpc1", "brkptcfg1",
                    "isdbmbxin", "isdbmbxout", "isdben", "isdbgpr",
                    "S44", "S45", "S46", "S47",
                    "pmunct0", "pmucnt1", "pmucnt2", "pmucnt3",
                    "pmuevtcfg", "pmucfg", "s54", "s55",
                    "s56", "s57", "s58", "s59",
                    "s60", "s61", "s62", "s63",
                ];
                f.write_str(SR_NAMES[*reg as usize])
            }
            Operand::GprNew { reg } => {
                write!(f, "r{}.new", reg)
            }
            Operand::GprLow { reg } => {
                write!(f, "r{}.l", reg)
            }
            Operand::GprHigh { reg } => {
                write!(f, "r{}.h", reg)
            }
            Operand::GprConjugate { reg } => {
                write!(f, "r{}*", reg)
            }
            Operand::Gpr64b { reg_low } => {
                write!(f, "r{}:{}", reg_low + 1, reg_low)
            }
            Operand::Gpr64bConjugate { reg_low } => {
                write!(f, "r{}:{}*", reg_low + 1, reg_low)
            }
            Operand::Cr64b { reg_low } => {
                if *reg_low == 14 {
                    f.write_str("upcycle")
                } else if *reg_low == 18 {
                    f.write_str("pktcount")
                } else if *reg_low == 30 {
                    f.write_str("utimer")
                } else {
                    write!(f, "c{}:{}", reg_low + 1, reg_low)
                }
            }
            Operand::Sr64b { reg_low } => {
                if *reg_low == 0 {
                    f.write_str("sgp1:0")
                } else {
                    write!(f, "s{}:{}", reg_low + 1, reg_low)
                }
            }
            Operand::PredicateReg { reg } => {
                write!(f, "p{}", reg)
            }
            Operand::RegOffset { base, offset } => {
                write!(f, "r{}+#{}", base, offset)
            }
            Operand::RegShiftedReg { base, index, shift } => {
                write!(f, "r{} + r{}<<{}", base, index, shift)
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
            Operand::Immext { imm } => {
                write!(f, "##{:#x}", imm)
            }
            Operand::RegShiftOffset { base, shift, offset } => {
                write!(f, "r{}<<{} + {:#x}", base, shift, offset)
            }
            Operand::RegOffsetInc { base, offset } => {
                write!(f, "r{}++#{:#x}", base, offset)
            }
            Operand::RegOffsetCirc { base, offset, mu } => {
                write!(f, "r{}++#{:#x}:circ(m{})", base, offset, mu)
            }
            Operand::RegCirc { base, mu } => {
                write!(f, "r{}++I:circ(m{})", base, mu)
            }
            Operand::RegMemIndexed { base, mu } => {
                write!(f, "r{}++m{}", base, mu)
            }
            Operand::RegMemIndexedBrev { base, mu } => {
                write!(f, "r{}++m{}:brev", base, mu)
            }
            Operand::RegStoreAssign { base, addr } => {
                write!(f, "r{}=#{:#x}", base, addr)
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
