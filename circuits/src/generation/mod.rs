//use std::collections::HashMap;

use core::program::Program;
use core::types::merkle_tree::decode_addr;
use std::collections::HashMap;

use std::sync::mpsc::channel;
use std::thread;

use eth_trie_utils::partial_trie::HashedPartialTrie;
use ethereum_types::{Address, H256};

//use eth_trie_utils::partial_trie::PartialTrie;
use plonky2::field::extension::Extendable;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::hash::hash_types::RichField;
use serde::{Deserialize, Serialize};

use crate::stark::ola_stark::{OlaStark, NUM_TABLES};
use crate::stark::proof::{BlockMetadata, PublicValues, TrieRoots};
use crate::stark::util::trace_to_poly_values;

use self::builtin::{generate_bitwise_trace, generate_cmp_trace, generate_rc_trace};
use self::cpu::generate_cpu_trace;
use self::memory::generate_memory_trace;
use self::poseidon::generate_poseidon_trace;
use self::poseidon_chunk::generate_poseidon_chunk_trace;
use self::sccall::generate_sccall_trace;
use self::storage::generate_storage_access_trace;
use self::tape::generate_tape_trace;

pub mod builtin;
pub mod cpu;
mod ctl_test;
pub mod memory;
pub mod poseidon;
pub mod poseidon_chunk;
pub mod prog;
pub mod sccall;
pub mod storage;
pub mod tape;

mod pre_process;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
/// Inputs needed for trace generation.
pub struct GenerationInputs {
    pub signed_txns: Vec<Vec<u8>>,
    pub tries: TrieInputs,
    pub trie_roots_after: TrieRoots,
    pub contract_code: HashMap<H256, Vec<u8>>,
    pub block_metadata: BlockMetadata,
    pub addresses: Vec<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TrieInputs {
    /// A partial version of the state trie prior to these transactions. It
    /// should include all nodes that will be accessed by these
    /// transactions.
    pub state_trie: HashedPartialTrie,

    /// A partial version of the transaction trie prior to these transactions.
    /// It should include all nodes that will be accessed by these
    /// transactions.
    pub transactions_trie: HashedPartialTrie,

    /// A partial version of the receipt trie prior to these transactions. It
    /// should include all nodes that will be accessed by these
    /// transactions.
    pub receipts_trie: HashedPartialTrie,

    /// A partial version of each storage trie prior to these transactions. It
    /// should include all storage tries, and nodes therein, that will be
    /// accessed by these transactions.
    pub storage_tries: Vec<(H256, HashedPartialTrie)>,
}

// #[derive(Debug, Clone)]
// pub struct TxExeTrace {
//     pub programs: Vec<(ContractAddress, Vec<u64>)>, // contract address to
// bytecode     pub cpu: Vec<(u64, u64, ExeContext, Vec<CpuExePiece>)>, /*
// call_sc_cnt, env_idx, context, trace.
//                                                      * Sorted by execution
//                                                        env. */
//     pub env_mem: HashMap<u64, Vec<MemExePiece>>, // env_id to mem, mem not
// sorted yet.     pub rc: Vec<RcExePiece>,                     /* rc only
// triggered by range_check
//                                                   * opcode. */
//     pub bitwise: Vec<BitwiseExePiece>,
//     pub cmp: Vec<CmpExePiece>,
//     pub poseidon: Vec<PoseidonPiece>, // poseidon only triggered by poseidon
// opcode.     pub storage: Vec<StorageExePiece>,
//     pub tape: Vec<TapeExePiece>,
// }

pub(crate) struct CpuSnapshot {}

pub(crate) struct BatchTxSnapshot {}

pub fn generate_traces<F: RichField + Extendable<D>, const D: usize>(
    mut program: Program,
    ola_stark: &mut OlaStark<F, D>,
    inputs: GenerationInputs,
) -> ([Vec<PolynomialValues<F>>; NUM_TABLES], PublicValues) {
    let (cpu_tx, cpu_rx) = channel();
    let exec = std::mem::replace(&mut program.trace.exec, Vec::new());
    let exec_for_cpu = exec.clone();
    thread::spawn(move || {
        let cpu_rows = generate_cpu_trace::<F>(&exec_for_cpu);
        let _ = cpu_tx.send(trace_to_poly_values(cpu_rows));
    });

    let (memory_tx, memory_rx) = channel();
    let memory = std::mem::replace(&mut program.trace.memory, Vec::new());
    thread::spawn(move || {
        let memory_rows = generate_memory_trace::<F>(&memory);
        let _ = memory_tx.send(trace_to_poly_values(memory_rows));
    });

    let (bitwise_tx, bitwise_rx) = channel();
    let builtin_bitwise_combined =
        std::mem::replace(&mut program.trace.builtin_bitwise_combined, Vec::new());
    thread::spawn(move || {
        let (bitwise_rows, bitwise_beta) = generate_bitwise_trace::<F>(&builtin_bitwise_combined);
        let _ = bitwise_tx.send((trace_to_poly_values(bitwise_rows), bitwise_beta));
    });

    let (cmp_tx, cmp_rx) = channel();
    let builtin_cmp = std::mem::replace(&mut program.trace.builtin_cmp, Vec::new());
    thread::spawn(move || {
        let cmp_rows = generate_cmp_trace(&builtin_cmp);
        let _ = cmp_tx.send(trace_to_poly_values(cmp_rows));
    });

    let (rc_tx, rc_rx) = channel();
    let builtin_rangecheck = std::mem::replace(&mut program.trace.builtin_rangecheck, Vec::new());
    thread::spawn(move || {
        let rc_rows = generate_rc_trace(&builtin_rangecheck);
        let _ = rc_tx.send(trace_to_poly_values(rc_rows));
    });

    let (poseidon_tx, poseidon_rx) = channel();
    let builtin_poseidon = std::mem::replace(&mut program.trace.builtin_poseidon, Vec::new());
    thread::spawn(move || {
        let poseidon_rows = generate_poseidon_trace(&builtin_poseidon);
        let _ = poseidon_tx.send(trace_to_poly_values(poseidon_rows));
    });

    let (poseidon_chunk_tx, poseidon_chunk_rx) = channel();
    let builtin_poseidon_chunk =
        std::mem::replace(&mut program.trace.builtin_poseidon_chunk, Vec::new());
    thread::spawn(move || {
        let poseidon_chunk_rows: [Vec<F>; 53] =
            generate_poseidon_chunk_trace(&builtin_poseidon_chunk);
        let _ = poseidon_chunk_tx.send(trace_to_poly_values(poseidon_chunk_rows));
    });

    let (storage_tx, storage_rx) = channel();
    let builtin_storage_hash =
        std::mem::replace(&mut program.trace.builtin_storage_hash, Vec::new());
    let builtin_program_hash =
        std::mem::replace(&mut program.trace.builtin_program_hash, Vec::new());
    thread::spawn(move || {
        let storage_access_rows =
            generate_storage_access_trace(&builtin_storage_hash, &builtin_program_hash);
        let _ = storage_tx.send(trace_to_poly_values(storage_access_rows));
    });

    let (tape_tx, tape_rx) = channel();
    let tape = std::mem::replace(&mut program.trace.tape, Vec::new());
    thread::spawn(move || {
        let tape_rows = generate_tape_trace(&tape);
        let _ = tape_tx.send(trace_to_poly_values(tape_rows));
    });

    let (sccall_tx, sccall_rx) = channel();
    let sc_call = std::mem::replace(&mut program.trace.sc_call, Vec::new());
    thread::spawn(move || {
        let sccall_rows = generate_sccall_trace(&sc_call);
        let _ = sccall_tx.send(trace_to_poly_values(sccall_rows));
    });

    let (program_tx, program_rx) = channel();
    let progs = program
        .trace
        .addr_program_hash
        .into_iter()
        .map(|(addr, hash)| (decode_addr(addr), hash))
        .collect::<Vec<_>>();
    let progs_for_program = progs.clone();
    thread::spawn(move || {
        let (program_rows, program_beta) =
            prog::generate_prog_trace::<F>(&exec, progs_for_program, program.trace.start_end_roots);
        let _ = program_tx.send((trace_to_poly_values(program_rows), program_beta));
    });

    let (prog_chunk_tx, prog_chunk_rx) = channel();
    thread::spawn(move || {
        let prog_chunk_rows = prog::generate_prog_chunk_trace::<F>(progs);
        let _ = prog_chunk_tx.send(trace_to_poly_values(prog_chunk_rows));
    });

    let (bitwise_trace, bitwise_beta) = bitwise_rx.recv().unwrap();
    ola_stark
        .bitwise_stark
        .set_compress_challenge(bitwise_beta)
        .unwrap();
    let (program_trace, program_beta) = program_rx.recv().unwrap();
    ola_stark
        .program_stark
        .set_compress_challenge(program_beta)
        .unwrap();

    let traces = [
        cpu_rx.recv().unwrap(),
        memory_rx.recv().unwrap(),
        bitwise_trace,
        cmp_rx.recv().unwrap(),
        rc_rx.recv().unwrap(),
        poseidon_rx.recv().unwrap(),
        poseidon_chunk_rx.recv().unwrap(),
        storage_rx.recv().unwrap(),
        tape_rx.recv().unwrap(),
        sccall_rx.recv().unwrap(),
        program_trace,
        prog_chunk_rx.recv().unwrap(),
    ];

    // TODO: update trie_roots_before & trie_roots_after
    let public_values = PublicValues {
        trie_roots_before: TrieRoots::default(),
        trie_roots_after: TrieRoots::default(),
        block_metadata: inputs.block_metadata,
    };
    (traces, public_values)
}
