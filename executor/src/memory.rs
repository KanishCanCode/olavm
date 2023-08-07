use crate::error::ProcessorError;
use crate::GoldilocksField;
use plonky2::field::types::Field;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MemoryCell {
    pub clk: u32,
    pub is_rw: GoldilocksField,
    pub op: GoldilocksField,
    pub is_write: GoldilocksField,
    pub filter_looked_for_main: GoldilocksField,
    pub region_heap: GoldilocksField,
    pub region_prophet: GoldilocksField,
    pub value: GoldilocksField,
}

#[derive(Debug, Default)]
pub struct MemoryTree {
    // visit by memory address, MemoryCell vector store memory trace value， the last one is the
    // current status
    pub trace: BTreeMap<u64, Vec<MemoryCell>>,
}

impl MemoryTree {
    pub fn read(
        &mut self,
        addr: u64,
        clk: u32,
        op: GoldilocksField,
        is_rw: GoldilocksField,
        is_write: GoldilocksField,
        filter_looked_for_main: GoldilocksField,
        region_prophet: GoldilocksField,
        region_heap: GoldilocksField,
    ) -> Result<GoldilocksField, ProcessorError> {
        // look up the previous value in the appropriate address trace and add (clk,
        // prev_value) to it; if this is the first time we access this address,
        // return MemVistInv error because memory must be inited first.
        // Return the last value in the address trace.
        let mut read_mem_res = self.trace.get_mut(&addr);
        if let Some(mut mem_data) = read_mem_res {
            let last_value = mem_data.last().expect("empty address trace").value;
            let new_value = MemoryCell {
                is_rw,
                clk,
                op,
                is_write,
                filter_looked_for_main,
                region_prophet,
                region_heap,
                value: last_value,
            };
            mem_data.push(new_value);
            Ok(last_value)
        } else {
            Err(ProcessorError::MemVistInv(format!(
                "read not init memory:{}",
                addr
            )))
        }
    }

    pub fn write(
        &mut self,
        addr: u64,
        clk: u32,
        op: GoldilocksField,
        is_rw: GoldilocksField,
        is_write: GoldilocksField,
        filter_looked_for_main: GoldilocksField,
        region_prophet: GoldilocksField,
        region_heap: GoldilocksField,
        value: GoldilocksField,
    ) {
        // add a memory access to the appropriate address trace; if this is the first
        // time we access this address, initialize address trace.
        let new_cell = MemoryCell {
            is_rw,
            clk,
            op,
            is_write,
            filter_looked_for_main,
            region_prophet,
            region_heap,
            value,
        };
        self.trace
            .entry(addr)
            .and_modify(|addr_trace| addr_trace.push(new_cell))
            .or_insert_with(|| vec![new_cell]);
    }
}
