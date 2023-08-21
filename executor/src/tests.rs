use crate::trace::gen_storage_table;
use crate::Process;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use core::merkle_tree::tree::AccountTree;
use core::program::binary_program::BinaryProgram;
use core::program::instruction::{ImmediateOrRegName, Opcode};
use core::program::Program;
use core::trace::trace::TapeRow;
use core::types::account::Address;
use core::types::merkle_tree::tree_key_default;
use log::Level::Debug;
use log::{debug, LevelFilter};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;

fn executor_run_test_program(bin_file_path: &str, trace_name: &str, print_trace: bool) {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();
    let file = File::open(bin_file_path).unwrap();

    let reader = BufReader::new(file);

    let program: BinaryProgram = serde_json::from_reader(reader).unwrap();

    let instructions = program.bytecode.split("\n");
    let mut prophets = HashMap::new();
    for item in program.prophets {
        prophets.insert(item.host as u64, item);
    }

    let mut program: Program = Program {
        instructions: Vec::new(),
        trace: Default::default(),
        debug_info: program.debug_info,
    };

    for inst in instructions {
        program.instructions.push(inst.to_string());
    }
    let mut process = Process::new();
    process.ctx_registers_stack.push(Address::default());

    let mut tp_start = 0;
    if trace_name.eq("tape_trace.txt") {
        let tx_ctx = init_tx_context();
        tp_start = load_tx_context(&mut process, &tx_ctx);
    }
    process.tp = GoldilocksField::from_canonical_u64(tp_start as u64);

    let res = process.execute(
        &mut program,
        &mut Some(prophets),
        &mut AccountTree::new_test(),
    );

    println!("execute res:{:?}", res);
    if print_trace {
        println!("vm trace: {:?}", program.trace);
    }
    let trace_json_format = serde_json::to_string(&program.trace).unwrap();

    let mut file = File::create(trace_name).unwrap();
    file.write_all(trace_json_format.as_ref()).unwrap();
}

#[test]
fn memory_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/memory.json",
        "memory_trace.txt",
        true,
    );
}

#[test]
fn range_check_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/range_check.json",
        "range_check_trace.txt",
        true,
    );
}

#[test]
fn bitwise_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/bitwise.json",
        "bitwise_trace.txt",
        true,
    );
}

#[test]
fn comparison_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/comparison.json",
        "comparison_trace.txt",
        true,
    );
}

#[test]
fn call_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/call.json",
        "call_trace.txt",
        true,
    );
}

#[test]
fn fibo_use_loop_decode() {
    executor_run_test_program(
        "../assembler/test_data/bin/fibo_loop.json",
        "fib_loop_trace.txt",
        true,
    );
}

#[test]
fn fibo_recursive() {
    executor_run_test_program(
        "../assembler/test_data/bin/fibo_recursive.json",
        "fibo_recursive_trace.txt",
        true,
    );
}

#[test]
fn prophet_sqrt_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/prophet_sqrt.json",
        "prophet_sqrt_trace.txt",
        true,
    );
}

#[test]
fn sqrt_newton_iteration_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/sqrt.json",
        "sqrt_trace.txt",
        true,
    );
}

#[test]
fn storage_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/storage.json",
        "storage_trace.txt",
        false,
    );
}

#[test]
fn storage_multi_keys_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/storage_multi_keys.json",
        "storage_multi_keys_trace.txt",
        false,
    );
}

#[test]
fn poseidon_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/poseidon.json",
        "poseidon_trace.txt",
        false,
    );
}

#[test]
fn malloc_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/malloc.json",
        "malloc_trace.txt",
        false,
    );
}

#[test]
fn vote_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/vote.json",
        "vote_trace.txt",
        false,
    );
}

#[test]
fn mem_gep_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/mem_gep.json",
        "mem_gep_trace.txt",
        false,
    );
}

#[test]
fn mem_gep_vecotr_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/mem_gep_vector.json",
        "mem_gep_vector_trace.txt",
        false,
    );
}

#[test]
fn string_assert_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/string_assert.json",
        "string_assert_trace.txt",
        false,
    );
}

#[test]
fn tape_test() {
    executor_run_test_program(
        "../assembler/test_data/bin/tape.json",
        "tape_trace.txt",
        false,
    );
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TxCtxInfo {
    pub block_number: GoldilocksField,
    pub block_timestamp: GoldilocksField,
    pub sequencer_address: [GoldilocksField; 4],
    pub version: GoldilocksField,
    pub chain_id: GoldilocksField,
    pub caller_address: [GoldilocksField; 4],
    pub nonce: GoldilocksField,
    pub signature: [GoldilocksField; 4],
    pub tx_hash: [GoldilocksField; 4],
}

fn init_tx_context() -> TxCtxInfo {
    TxCtxInfo {
        block_number: GoldilocksField::from_canonical_u64(3),
        block_timestamp: GoldilocksField::from_canonical_u64(1692846754),
        sequencer_address: [
            GoldilocksField::from_canonical_u64(1),
            GoldilocksField::from_canonical_u64(2),
            GoldilocksField::from_canonical_u64(3),
            GoldilocksField::from_canonical_u64(4),
        ],
        version: GoldilocksField::from_canonical_u64(3),
        chain_id: GoldilocksField::from_canonical_u64(1),
        caller_address: [
            GoldilocksField::from_canonical_u64(5),
            GoldilocksField::from_canonical_u64(6),
            GoldilocksField::from_canonical_u64(7),
            GoldilocksField::from_canonical_u64(8),
        ],
        nonce: GoldilocksField::from_canonical_u64(25),
        signature: [
            GoldilocksField::from_canonical_u64(rand::random()),
            GoldilocksField::from_canonical_u64(rand::random()),
            GoldilocksField::from_canonical_u64(rand::random()),
            GoldilocksField::from_canonical_u64(rand::random()),
        ],
        tx_hash: [GoldilocksField::from_canonical_u64(rand::random()); 4],
    }
}

fn load_tx_context(process: &mut Process, tx_ctx: &TxCtxInfo) -> usize {
    let mut serd = bincode::serialize(tx_ctx).expect("Serialization failed");

    serd.chunks(8).enumerate().for_each(|(addr, mut e)| {
        let value = e
            .read_u64::<LittleEndian>()
            .expect("failed to deserialize value");
        process.tape.write(
            addr as u64,
            0,
            GoldilocksField::from_canonical_u64(0),
            GoldilocksField::ONE,
            GoldilocksField::ZERO,
            GoldilocksField::from_canonical_u64(value),
        );
    });

    serd.len() / 8
}

#[test]
fn gen_storage_table_test() {
    let mut program: Program = Program {
        instructions: Vec::new(),
        trace: Default::default(),
        debug_info: Default::default(),
    };
    let mut hash = Vec::new();
    let mut process = Process::new();

    let mut store_addr = [
        GoldilocksField::from_canonical_u64(8),
        GoldilocksField::from_canonical_u64(9),
        GoldilocksField::from_canonical_u64(10),
        GoldilocksField::from_canonical_u64(11),
    ];

    let mut store_val = [
        GoldilocksField::from_canonical_u64(1),
        GoldilocksField::from_canonical_u64(2),
        GoldilocksField::from_canonical_u64(3),
        GoldilocksField::from_canonical_u64(4),
    ];

    process.storage.write(
        1,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        store_val,
        tree_key_default(),
    );
    hash.push(tree_key_default());
    store_val[3] = GoldilocksField::from_canonical_u64(5);
    process.storage.write(
        3,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        store_val,
        tree_key_default(),
    );
    hash.push(tree_key_default());

    process.storage.read(
        7,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        tree_key_default(),
        tree_key_default(),
    );
    hash.push(tree_key_default());

    process.storage.read(
        6,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        tree_key_default(),
        tree_key_default(),
    );
    hash.push(tree_key_default());

    store_val[3] = GoldilocksField::from_canonical_u64(8);
    store_addr[3] = GoldilocksField::from_canonical_u64(6);

    process.storage.write(
        5,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        store_val,
        tree_key_default(),
    );
    hash.push(tree_key_default());

    store_val[3] = GoldilocksField::from_canonical_u64(9);
    process.storage.write(
        2,
        GoldilocksField::from_canonical_u64(1 << Opcode::SSTORE as u64),
        store_addr,
        store_val,
        tree_key_default(),
    );
    hash.push(tree_key_default());

    process.storage.read(
        9,
        GoldilocksField::from_canonical_u64(1 << Opcode::SLOAD as u64),
        store_addr,
        tree_key_default(),
        tree_key_default(),
    );
    hash.push(tree_key_default());

    gen_storage_table(&mut process, &mut program, hash);
}
