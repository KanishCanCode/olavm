extern crate clap;

use assembler::encoder::encode_asm_from_json_file;
use circuits::generation::GenerationInputs;
use circuits::stark::config::StarkConfig;
use circuits::stark::ola_stark::OlaStark;
use circuits::stark::prover::prove;
use circuits::stark::serialization::Buffer;
use circuits::stark::verifier::verify_proof;
use clap::{arg, Command};
use core::merkle_tree::tree::AccountTree;
use core::program::binary_program::BinaryProgram;
use core::program::Program;
use core::trace::trace::Trace;
use core::types::{Field, GoldilocksField};
use core::vm::transaction::init_tx_context;
use core::vm::vm_state::Address;
use executor::load_tx::init_tape;
use executor::Process;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::util::timing::TimingTree;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, metadata, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Instant;

#[allow(dead_code)]
const D: usize = 2;
#[allow(dead_code)]
type C = PoseidonGoldilocksConfig;
#[allow(dead_code)]
type F = <C as GenericConfig<D>>::F;

fn main() {
    let matches = Command::new("olavm")
        .about("Olavm cli")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("asm")
                .about("Run assembler to generate executable instruction code")
                .args(&[
                    arg!(-i --input <INPUT> "Must set a input file for Ola-lang assemble language"),
                    arg!(-o --output <OUTPUT> "Must set a output file for OlaVM executable instruction code"),
                ])
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("run")
                .about("Run an program from an input code file")
                .args(&[
                    arg!(-i --input <INPUT> "Must set a input file for OlaVM executing"),
                    arg!(-a --args <INPUT> "Must set a input file for OlaVM executing"),
                    arg!(-o --output <OUTPUT> "Must set a output file for OlaVM executing"),
                ])
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("prove")
                .about("generate proof from executed program")
                .args(&[
                    arg!(-i --input <Trace> "Must set a trace file generated by OlaVM executor"),
                    arg!(-o --output <Proof> "Must set a file for save proofs"),
                ])
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("verify")
                .about("verifiy generated proof")
                .args(&[arg!(-i --input <Trace> "Must set a proof file generated by OlaVM prover")])
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("asm", sub_matches)) => {
            let input_path = sub_matches.get_one::<String>("input").expect("required");
            println!("Input assemble file path: {}", input_path);
            let program = encode_asm_from_json_file(input_path.clone()).unwrap();
            let output_path = sub_matches.get_one::<String>("output").expect("required");
            println!("Output OlaVM raw codes file path: {}", output_path);
            let pretty = serde_json::to_string_pretty(&program).unwrap();
            fs::write(output_path, pretty).unwrap();
            println!("Asm done!");
        }
        Some(("run", sub_matches)) => {
            let path = sub_matches.get_one::<String>("input").expect("required");
            println!("Input program file path: {}", path);

            let args_path = sub_matches.get_one::<String>("args").expect("required");
            println!("Args file path: {}", args_path);
            let args_file = File::open(&args_path).unwrap();
            let args_reader = BufReader::new(args_file);
            let mut args = Vec::new();
            for line_result in args_reader.lines() {
                let line = line_result.unwrap();
                let number = line.trim().parse::<u64>().unwrap();
                args.push(number);
            }
            if args.len() < 2 {
                panic!("args length must larger than 2");
            }
            let first_two_inv: Vec<u64> = vec![args[1], args[0]];
            let rest: Vec<u64> = args.iter().skip(2).cloned().collect();
            let combined: Vec<u64> = [rest.as_slice(), first_two_inv.as_slice()].concat();
            let calldata: Vec<GoldilocksField> = combined
                .iter()
                .map(|v| GoldilocksField::from_canonical_u64(*v))
                .collect();

            let program = encode_asm_from_json_file(path.clone()).unwrap();
            let instructions = program.bytecode.split("\n");
            let mut prophets = HashMap::new();
            for item in program.prophets {
                prophets.insert(item.host as u64, item);
            }

            let mut program: Program = Program {
                instructions: Vec::new(),
                trace: Default::default(),
                debug_info: BTreeMap::new(),
            };

            for inst in instructions {
                program.instructions.push(inst.to_string());
            }
            let now = Instant::now();
            let mut process = Process::new();
            process.addr_storage = Address::default();
            process.tp = GoldilocksField::ZERO;
            init_tape(
                &mut process,
                calldata,
                Address::default(),
                Address::default(),
                Address::default(),
                &init_tx_context(),
            );

            process
                .execute(
                    &mut program,
                    &mut Some(prophets),
                    &mut AccountTree::new_db_test("./db_test".to_string()),
                )
                .expect("OlaVM execute fail");
            println!("exec time:{}", now.elapsed().as_millis());

            let now = Instant::now();

            let path = sub_matches.get_one::<String>("output").expect("required");
            println!("Output trace file path: {}", path);
            let file = File::create(path).unwrap();
            serde_json::to_writer(file, &program.trace).unwrap();
            println!("write time:{}", now.elapsed().as_millis());

            println!("Run done!");
        }
        Some(("prove", sub_matches)) => {
            let path = sub_matches.get_one::<String>("input").expect("required");
            println!("Input trace file path: {}", path);

            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);

            let trace: Trace = serde_json::from_reader(reader).unwrap();
            let program: Program = Program {
                instructions: trace.raw_binary_instructions.clone(),
                trace,
                debug_info: BTreeMap::new(),
            };

            let inputs = GenerationInputs::default();

            let mut ola_stark = OlaStark::<F, D>::default();
            let config = StarkConfig::standard_fast_config();
            let proof = prove::<F, C, D>(
                &program,
                &mut ola_stark,
                inputs,
                &config,
                &mut TimingTree::default(),
            )
            .unwrap();

            let path = sub_matches.get_one::<String>("output").expect("required");
            println!("Output proof file path: {}", path);
            let mut file = File::create(path).unwrap();
            let mut buffer = Buffer::new(Vec::new());
            buffer.write_all_proof(&proof).unwrap();
            let se_proof = buffer.bytes();
            file.write_all(&se_proof).unwrap();

            println!("Proof size: {} bytes", se_proof.len());
            println!("Prove done!");
        }
        Some(("verify", sub_matches)) => {
            println!("Loading proof...");
            let path = sub_matches.get_one::<String>("input").expect("required");
            println!("Input file path: {}", path);

            let mut file = File::open(path).unwrap();
            let metadata = metadata(&path).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            file.read(&mut buffer).expect("buffer overflow");

            let mut de_buffer = Buffer::new(buffer);
            let de_proof = de_buffer.read_all_proof::<F, C, D>();
            if de_proof.is_err() {
                println!("Deserialize proof failed!");
                return;
            }
            let de_proof = de_proof.unwrap();

            let ola_stark = OlaStark::<F, D>::default();
            let config = StarkConfig::standard_fast_config();
            match verify_proof(ola_stark, de_proof, &config) {
                Err(error) => println!("Verify failed due to: {error}"),
                _ => println!("Verify succeed!"),
            }
        }
        _ => unreachable!(),
    }
}
