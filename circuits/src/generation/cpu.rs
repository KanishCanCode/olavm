use core::{
    program::{CTX_REGISTER_NUM, REGISTER_NUM},
    trace::trace::Step,
    vm::opcodes::OlaOpcode,
};
use std::collections::HashMap;

use crate::cpu::columns::{self as cpu, COL_IS_ENTRY_SC};
use plonky2::hash::hash_types::RichField;

pub fn generate_cpu_trace<F: RichField>(steps: &Vec<Step>) -> [Vec<F>; cpu::NUM_CPU_COLS] {
    let trace_len = steps.len();

    let ext_trace_len = if !trace_len.is_power_of_two() {
        trace_len.next_power_of_two()
    } else {
        trace_len
    };
    let mut trace: Vec<Vec<F>> = vec![vec![F::ZERO; ext_trace_len]; cpu::NUM_CPU_COLS];
    let mut opcode_to_selector = HashMap::new();
    opcode_to_selector.insert(
        OlaOpcode::ADD.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::MUL.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::EQ.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::ASSERT.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(OlaOpcode::MOV.binary_bit_mask(), cpu::COL_S_MOV);
    opcode_to_selector.insert(OlaOpcode::JMP.binary_bit_mask(), cpu::COL_S_JMP);
    opcode_to_selector.insert(OlaOpcode::CJMP.binary_bit_mask(), cpu::COL_S_CJMP);
    opcode_to_selector.insert(OlaOpcode::CALL.binary_bit_mask(), cpu::COL_S_CALL);
    opcode_to_selector.insert(OlaOpcode::RET.binary_bit_mask(), cpu::COL_S_RET);
    opcode_to_selector.insert(OlaOpcode::MLOAD.binary_bit_mask(), cpu::COL_S_MLOAD);
    opcode_to_selector.insert(OlaOpcode::MSTORE.binary_bit_mask(), cpu::COL_S_MSTORE);
    opcode_to_selector.insert(OlaOpcode::END.binary_bit_mask(), cpu::COL_S_END);
    opcode_to_selector.insert(OlaOpcode::RC.binary_bit_mask(), cpu::COL_S_RC);
    opcode_to_selector.insert(OlaOpcode::AND.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::OR.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::XOR.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::NOT.binary_bit_mask(), cpu::COL_S_NOT);
    opcode_to_selector.insert(
        OlaOpcode::NEQ.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(OlaOpcode::GTE.binary_bit_mask(), cpu::COL_S_GTE);
    opcode_to_selector.insert(OlaOpcode::POSEIDON.binary_bit_mask(), cpu::COL_S_PSDN);
    opcode_to_selector.insert(OlaOpcode::SLOAD.binary_bit_mask(), cpu::COL_S_SLOAD);
    opcode_to_selector.insert(OlaOpcode::SSTORE.binary_bit_mask(), cpu::COL_S_SSTORE);
    opcode_to_selector.insert(OlaOpcode::TLOAD.binary_bit_mask(), cpu::COL_S_TLOAD);
    opcode_to_selector.insert(OlaOpcode::TSTORE.binary_bit_mask(), cpu::COL_S_TSTORE);
    opcode_to_selector.insert(OlaOpcode::SCCALL.binary_bit_mask(), cpu::COL_S_CALL_SC);

    for (i, s) in steps.iter().enumerate() {
        // env related columns.
        trace[cpu::COL_TX_IDX][i] = F::ZERO;
        trace[cpu::COL_ENV_IDX][i] = F::from_canonical_u64(s.env_idx.0);
        trace[cpu::COL_CALL_SC_CNT][i] = F::from_canonical_u64(s.call_sc_cnt.0);

        // Context related columns.
        for j in 0..CTX_REGISTER_NUM {
            trace[cpu::COL_ADDR_STORAGE_RANGE.start + j][i] =
                F::from_canonical_u64(s.addr_storage[j].0);
        }
        for j in 0..CTX_REGISTER_NUM {
            trace[cpu::COL_ADDR_CODE_RANGE.start + j][i] = F::from_canonical_u64(s.addr_code[j].0);
        }
        trace[cpu::COL_TP][i] = F::from_canonical_u64(s.tp.0);
        trace[cpu::COL_CLK][i] = F::from_canonical_u32(s.clk);
        trace[cpu::COL_PC][i] = F::from_canonical_u64(s.pc);
        trace[cpu::COL_IS_EXT_LINE][i] = F::from_canonical_u64(s.is_ext_line.0);
        trace[cpu::COL_EXT_CNT][i] = F::from_canonical_u64(s.ext_cnt.0);
        for j in 0..REGISTER_NUM {
            trace[cpu::COL_START_REG + j][i] = F::from_canonical_u64(s.regs[j].0);
        }
        // Instruction related columns.
        trace[cpu::COL_INST][i] = F::from_canonical_u64(s.instruction.0);
        trace[cpu::COL_OP1_IMM][i] = F::from_canonical_u64(s.op1_imm.0);
        trace[cpu::COL_OPCODE][i] = F::from_canonical_u64(s.opcode.0);
        trace[cpu::COL_IMM_VAL][i] = F::from_canonical_u64(s.immediate_data.0);

        // Selectors of register related columns.
        trace[cpu::COL_OP0][i] = F::from_canonical_u64(s.register_selector.op0.0);
        trace[cpu::COL_OP1][i] = F::from_canonical_u64(s.register_selector.op1.0);
        trace[cpu::COL_DST][i] = F::from_canonical_u64(s.register_selector.dst.0);
        trace[cpu::COL_AUX0][i] = F::from_canonical_u64(s.register_selector.aux0.0);
        trace[cpu::COL_AUX1][i] = F::from_canonical_u64(s.register_selector.aux1.0);
        trace[cpu::COL_IDX_STORAGE][i] = F::from_canonical_u64(s.storage_access_idx.0);

        for j in 0..REGISTER_NUM {
            trace[cpu::COL_S_OP0_START + j][i] =
                F::from_canonical_u64(s.register_selector.op0_reg_sel[j].0);
            trace[cpu::COL_S_OP1_START + j][i] =
                F::from_canonical_u64(s.register_selector.op1_reg_sel[j].0);
            trace[cpu::COL_S_DST_START + j][i] =
                F::from_canonical_u64(s.register_selector.dst_reg_sel[j].0);
        }

        // Selectors of opcode related columns.
        match opcode_to_selector.get(&s.opcode.0) {
            Some(selector) => trace[selector.clone()][i] = F::from_canonical_u64(1),
            None => (),
        }

        trace[COL_IS_ENTRY_SC][i] = if trace[cpu::COL_ENV_IDX][i].is_zero() {
            F::ONE
        } else {
            F::ZERO
        };

        let ext_length = if s.opcode.0 == OlaOpcode::SLOAD.binary_bit_mask()
            || s.opcode.0 == OlaOpcode::SSTORE.binary_bit_mask()
            || s.opcode.0 == OlaOpcode::SCCALL.binary_bit_mask()
            || (s.opcode.0 == OlaOpcode::END.binary_bit_mask()
                && !trace[cpu::COL_ENV_IDX][i].is_zero())
        {
            1
        } else if s.opcode.0 == OlaOpcode::TLOAD.binary_bit_mask() {
            s.register_selector.op0.0 * s.register_selector.op1.0 + (1 - s.register_selector.op0.0)
        } else if s.opcode.0 == OlaOpcode::TSTORE.binary_bit_mask() {
            s.register_selector.op1.0
        } else {
            0
        };

        trace[cpu::COL_IS_NEXT_LINE_DIFF_INST][i] = if ext_length == s.ext_cnt.0 {
            F::ONE
        } else {
            F::ZERO
        };
        trace[cpu::COL_IS_NEXT_LINE_SAME_TX][i] = if trace[cpu::COL_ENV_IDX][i].is_zero()
            && s.opcode.0 == OlaOpcode::END.binary_bit_mask()
        {
            F::ZERO
        } else {
            F::ONE
        };
        trace[cpu::COL_FILTER_TAPE_LOOKING][i] = F::from_canonical_u64(s.filter_tape_looking.0);
        trace[cpu::IS_SCCALL_EXT_LINE][i] =
            if s.opcode.0 == OlaOpcode::SCCALL.binary_bit_mask() && s.ext_cnt.0 == 1 {
                F::ONE
            } else {
                F::ZERO
            };
        trace[cpu::COL_IS_STORAGE_EXT_LINE][i] = if (s.opcode.0
            == OlaOpcode::SLOAD.binary_bit_mask()
            || s.opcode.0 == OlaOpcode::SSTORE.binary_bit_mask())
            && s.is_ext_line.0 == 1
        {
            F::ONE
        } else {
            F::ZERO
        };
        trace[cpu::COL_FILTER_SCCALL_END][i] =
            if s.opcode.0 == OlaOpcode::END.binary_bit_mask() && s.is_ext_line.0 == 1 {
                F::ONE
            } else {
                F::ZERO
            };
        trace[cpu::COL_FILTER_LOOKING_PROG_IMM][i] = if s.is_ext_line.0 == 1 {
            F::ZERO
        } else if s.opcode.0 == OlaOpcode::MLOAD.binary_bit_mask()
            || s.opcode.0 == OlaOpcode::MSTORE.binary_bit_mask()
        {
            F::ONE
        } else if s.op1_imm.0 == 1 {
            F::ONE
        } else {
            F::ZERO
        };
    }
    // fill in padding.
    let inst_end = if trace_len == 0 {
        F::from_canonical_u64(1048576)
    } else {
        trace[cpu::COL_INST][trace_len - 1]
    };
    let last_tx_id = if trace_len == 0 {
        F::ZERO
    } else {
        trace[cpu::COL_TX_IDX][trace_len - 1]
    };
    let last_idx_storage = if trace_len == 0 {
        F::ZERO
    } else {
        trace[cpu::COL_IDX_STORAGE][trace_len - 1]
    };

    if trace_len != ext_trace_len {
        trace[cpu::COL_TX_IDX][trace_len..].fill(last_tx_id);
        trace[cpu::COL_INST][trace_len..].fill(inst_end);
        trace[cpu::COL_OPCODE][trace_len..]
            .fill(F::from_canonical_u64(OlaOpcode::END.binary_bit_mask()));
        trace[cpu::COL_IDX_STORAGE][trace_len..].fill(last_idx_storage);
        trace[cpu::COL_S_END][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_ENTRY_SC][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_NEXT_LINE_DIFF_INST][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_NEXT_LINE_SAME_TX][trace_len..].fill(F::ZERO);
        trace[cpu::COL_IS_PADDING][trace_len..].fill(F::ONE);
    }

    let trace_row_vecs = trace.try_into().unwrap_or_else(|v: Vec<Vec<F>>| {
        panic!(
            "Expected a Vec of length {} but it was {}",
            cpu::NUM_CPU_COLS,
            v.len()
        )
    });
    trace_row_vecs
}

pub fn gen_cpu_trace<F: RichField>(batch_steps: &Vec<Vec<Step>>) -> [Vec<F>; cpu::NUM_CPU_COLS] {
    let trace_len: usize = batch_steps.iter().map(|inner_vec| inner_vec.len()).sum();
    let ext_trace_len = if !trace_len.is_power_of_two() {
        trace_len.next_power_of_two()
    } else {
        trace_len
    };
    let mut trace: Vec<Vec<F>> = vec![vec![F::ZERO; ext_trace_len]; cpu::NUM_CPU_COLS];
    let mut opcode_to_selector = HashMap::new();
    opcode_to_selector.insert(
        OlaOpcode::ADD.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::MUL.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::EQ.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(
        OlaOpcode::ASSERT.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(OlaOpcode::MOV.binary_bit_mask(), cpu::COL_S_MOV);
    opcode_to_selector.insert(OlaOpcode::JMP.binary_bit_mask(), cpu::COL_S_JMP);
    opcode_to_selector.insert(OlaOpcode::CJMP.binary_bit_mask(), cpu::COL_S_CJMP);
    opcode_to_selector.insert(OlaOpcode::CALL.binary_bit_mask(), cpu::COL_S_CALL);
    opcode_to_selector.insert(OlaOpcode::RET.binary_bit_mask(), cpu::COL_S_RET);
    opcode_to_selector.insert(OlaOpcode::MLOAD.binary_bit_mask(), cpu::COL_S_MLOAD);
    opcode_to_selector.insert(OlaOpcode::MSTORE.binary_bit_mask(), cpu::COL_S_MSTORE);
    opcode_to_selector.insert(OlaOpcode::END.binary_bit_mask(), cpu::COL_S_END);
    opcode_to_selector.insert(OlaOpcode::RC.binary_bit_mask(), cpu::COL_S_RC);
    opcode_to_selector.insert(OlaOpcode::AND.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::OR.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::XOR.binary_bit_mask(), cpu::COL_S_BITWISE);
    opcode_to_selector.insert(OlaOpcode::NOT.binary_bit_mask(), cpu::COL_S_NOT);
    opcode_to_selector.insert(
        OlaOpcode::NEQ.binary_bit_mask(),
        cpu::COL_S_SIMPLE_ARITHMATIC_OP,
    );
    opcode_to_selector.insert(OlaOpcode::GTE.binary_bit_mask(), cpu::COL_S_GTE);
    opcode_to_selector.insert(OlaOpcode::POSEIDON.binary_bit_mask(), cpu::COL_S_PSDN);
    opcode_to_selector.insert(OlaOpcode::SLOAD.binary_bit_mask(), cpu::COL_S_SLOAD);
    opcode_to_selector.insert(OlaOpcode::SSTORE.binary_bit_mask(), cpu::COL_S_SSTORE);
    opcode_to_selector.insert(OlaOpcode::TLOAD.binary_bit_mask(), cpu::COL_S_TLOAD);
    opcode_to_selector.insert(OlaOpcode::TSTORE.binary_bit_mask(), cpu::COL_S_TSTORE);
    opcode_to_selector.insert(OlaOpcode::SCCALL.binary_bit_mask(), cpu::COL_S_CALL_SC);

    let mut i: usize = 0;
    for (tx_idx, tx_steps) in batch_steps.iter().enumerate() {
        for s in tx_steps.iter() {
            // env related columns.
            trace[cpu::COL_TX_IDX][i] = F::from_canonical_usize(tx_idx);
            trace[cpu::COL_ENV_IDX][i] = F::from_canonical_u64(s.env_idx.0);
            trace[cpu::COL_CALL_SC_CNT][i] = F::from_canonical_u64(s.call_sc_cnt.0);

            // Context related columns.
            for j in 0..CTX_REGISTER_NUM {
                trace[cpu::COL_ADDR_STORAGE_RANGE.start + j][i] =
                    F::from_canonical_u64(s.addr_storage[j].0);
            }
            for j in 0..CTX_REGISTER_NUM {
                trace[cpu::COL_ADDR_CODE_RANGE.start + j][i] =
                    F::from_canonical_u64(s.addr_code[j].0);
            }
            trace[cpu::COL_TP][i] = F::from_canonical_u64(s.tp.0);
            trace[cpu::COL_CLK][i] = F::from_canonical_u32(s.clk);
            trace[cpu::COL_PC][i] = F::from_canonical_u64(s.pc);
            trace[cpu::COL_IS_EXT_LINE][i] = F::from_canonical_u64(s.is_ext_line.0);
            trace[cpu::COL_EXT_CNT][i] = F::from_canonical_u64(s.ext_cnt.0);
            for j in 0..REGISTER_NUM {
                trace[cpu::COL_START_REG + j][i] = F::from_canonical_u64(s.regs[j].0);
            }
            // Instruction related columns.
            trace[cpu::COL_INST][i] = F::from_canonical_u64(s.instruction.0);
            trace[cpu::COL_OP1_IMM][i] = F::from_canonical_u64(s.op1_imm.0);
            trace[cpu::COL_OPCODE][i] = F::from_canonical_u64(s.opcode.0);
            trace[cpu::COL_IMM_VAL][i] = F::from_canonical_u64(s.immediate_data.0);

            // Selectors of register related columns.
            trace[cpu::COL_OP0][i] = F::from_canonical_u64(s.register_selector.op0.0);
            trace[cpu::COL_OP1][i] = F::from_canonical_u64(s.register_selector.op1.0);
            trace[cpu::COL_DST][i] = F::from_canonical_u64(s.register_selector.dst.0);
            trace[cpu::COL_AUX0][i] = F::from_canonical_u64(s.register_selector.aux0.0);
            trace[cpu::COL_AUX1][i] = F::from_canonical_u64(s.register_selector.aux1.0);
            trace[cpu::COL_IDX_STORAGE][i] = F::from_canonical_u64(s.storage_access_idx.0);

            for j in 0..REGISTER_NUM {
                trace[cpu::COL_S_OP0_START + j][i] =
                    F::from_canonical_u64(s.register_selector.op0_reg_sel[j].0);
                trace[cpu::COL_S_OP1_START + j][i] =
                    F::from_canonical_u64(s.register_selector.op1_reg_sel[j].0);
                trace[cpu::COL_S_DST_START + j][i] =
                    F::from_canonical_u64(s.register_selector.dst_reg_sel[j].0);
            }

            // Selectors of opcode related columns.
            match opcode_to_selector.get(&s.opcode.0) {
                Some(selector) => trace[selector.clone()][i] = F::from_canonical_u64(1),
                None => (),
            }

            trace[COL_IS_ENTRY_SC][i] = if trace[cpu::COL_ENV_IDX][i].is_zero() {
                F::ONE
            } else {
                F::ZERO
            };

            let ext_length = if s.opcode.0 == OlaOpcode::SLOAD.binary_bit_mask()
                || s.opcode.0 == OlaOpcode::SSTORE.binary_bit_mask()
                || s.opcode.0 == OlaOpcode::SCCALL.binary_bit_mask()
                || (s.opcode.0 == OlaOpcode::END.binary_bit_mask()
                    && !trace[cpu::COL_ENV_IDX][i].is_zero())
            {
                1
            } else if s.opcode.0 == OlaOpcode::TLOAD.binary_bit_mask() {
                s.register_selector.op0.0 * s.register_selector.op1.0
                    + (1 - s.register_selector.op0.0)
            } else if s.opcode.0 == OlaOpcode::TSTORE.binary_bit_mask() {
                s.register_selector.op1.0
            } else {
                0
            };

            trace[cpu::COL_IS_NEXT_LINE_DIFF_INST][i] = if ext_length == s.ext_cnt.0 {
                F::ONE
            } else {
                F::ZERO
            };
            trace[cpu::COL_IS_NEXT_LINE_SAME_TX][i] = if trace[cpu::COL_ENV_IDX][i].is_zero()
                && s.opcode.0 == OlaOpcode::END.binary_bit_mask()
            {
                F::ZERO
            } else {
                F::ONE
            };
            trace[cpu::COL_FILTER_TAPE_LOOKING][i] = F::from_canonical_u64(s.filter_tape_looking.0);
            trace[cpu::IS_SCCALL_EXT_LINE][i] =
                if s.opcode.0 == OlaOpcode::SCCALL.binary_bit_mask() && s.ext_cnt.0 == 1 {
                    F::ONE
                } else {
                    F::ZERO
                };
            trace[cpu::COL_IS_STORAGE_EXT_LINE][i] = if (s.opcode.0
                == OlaOpcode::SLOAD.binary_bit_mask()
                || s.opcode.0 == OlaOpcode::SSTORE.binary_bit_mask())
                && s.is_ext_line.0 == 1
            {
                F::ONE
            } else {
                F::ZERO
            };
            trace[cpu::COL_FILTER_SCCALL_END][i] =
                if s.opcode.0 == OlaOpcode::END.binary_bit_mask() && s.is_ext_line.0 == 1 {
                    F::ONE
                } else {
                    F::ZERO
                };
            trace[cpu::COL_FILTER_LOOKING_PROG_IMM][i] = if s.is_ext_line.0 == 1 {
                F::ZERO
            } else if s.opcode.0 == OlaOpcode::MLOAD.binary_bit_mask()
                || s.opcode.0 == OlaOpcode::MSTORE.binary_bit_mask()
            {
                F::ONE
            } else if s.op1_imm.0 == 1 {
                F::ONE
            } else {
                F::ZERO
            };
            i += 1;
        }
    }

    // fill in padding.
    let inst_end = if trace_len == 0 {
        F::from_canonical_u64(1048576)
    } else {
        trace[cpu::COL_INST][trace_len - 1]
    };
    let last_tx_id = if trace_len == 0 {
        F::ZERO
    } else {
        trace[cpu::COL_TX_IDX][trace_len - 1]
    };
    let last_idx_storage = if trace_len == 0 {
        F::ZERO
    } else {
        trace[cpu::COL_IDX_STORAGE][trace_len - 1]
    };

    if trace_len != ext_trace_len {
        trace[cpu::COL_TX_IDX][trace_len..].fill(last_tx_id);
        trace[cpu::COL_INST][trace_len..].fill(inst_end);
        trace[cpu::COL_OPCODE][trace_len..]
            .fill(F::from_canonical_u64(OlaOpcode::END.binary_bit_mask()));
        trace[cpu::COL_IDX_STORAGE][trace_len..].fill(last_idx_storage);
        trace[cpu::COL_S_END][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_ENTRY_SC][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_NEXT_LINE_DIFF_INST][trace_len..].fill(F::ONE);
        trace[cpu::COL_IS_NEXT_LINE_SAME_TX][trace_len..].fill(F::ZERO);
        trace[cpu::COL_IS_PADDING][trace_len..].fill(F::ONE);
    }

    let trace_row_vecs = trace.try_into().unwrap_or_else(|v: Vec<Vec<F>>| {
        panic!(
            "Expected a Vec of length {} but it was {}",
            cpu::NUM_CPU_COLS,
            v.len()
        )
    });
    trace_row_vecs
}
