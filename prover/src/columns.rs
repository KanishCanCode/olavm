// The trace for starky should be like:
// ┌───────┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬────────┐
// │  inst │  clk  │  pc   │ flag  │ reg_0 │ reg_1 │ reg_2 │  ...  │ reg_15 |
// ├───────┼───────┼───────┼───────┼───────┼───────┼───────┼───────┼────────|
// │  add  │  10   │  123  │   0   │   0   │   1   │   2   │       │        |
// └───────┴───────┴───────┴───────┴───────┴───────┴───────┴───────┴────────┘

// columns bumber of trace.
pub(crate) const COL_INST: usize = 0;
pub(crate) const COL_CLK: usize = COL_INST + 1;
pub(crate) const COL_PC: usize = COL_CLK + 1;
pub(crate) const COL_FLAG: usize = COL_PC + 1;

// start column of register array.
pub(crate) const COL_REG: usize = COL_FLAG + 1;

// For arithmetic operation, we need `NUM_ARITH_COLS` columns for stark trace.
pub(crate) const NUM_ARITH_COLS: usize = COL_REG + 3;

// For flow operation, we need `NUM_FLOW_COLS` columns for stark trace.
pub(crate) const NUM_FLOW_COLS: usize = COL_REG + 2;

// arithmetic register columns.
// reg_0 for output.
pub(crate) const COL_ARITH_OUTPUT: usize = COL_REG;
// reg_1 for input0.
pub(crate) const COL_ARITH_INPUT0: usize = COL_ARITH_OUTPUT + 1;
// reg_2 for input1.
pub(crate) const COL_ARITH_INPUT1: usize = COL_ARITH_INPUT0 + 1;

// flow register columns.
// reg_0 for dst.
pub(crate) const COL_FLOW_DST: usize = COL_REG;
// reg_1 for src.
pub(crate) const COL_FLOW_SRC: usize = COL_FLOW_DST + 1;

// usize type of instruction.
pub(crate) const ADD_ID: usize = 1;
pub(crate) const MUL_ID: usize = 2;
pub(crate) const EQ_ID: usize = 3;
pub(crate) const MOV_ID: usize = 4;
pub(crate) const JMP_ID: usize = 5;
pub(crate) const CJMP_ID: usize = 6;
pub(crate) const CALL_ID: usize = 7;
pub(crate) const RET_ID: usize = 8;
pub(crate) const MLOAD_ID: usize = 9;
pub(crate) const MSTORE_ID: usize = 10;
