#[cfg(test)]
mod tests {
    use crate::{
        batch_exe_manager::BlockExeInfo,
        config::*,
        ola_storage::{DiskStorageWriter, OlaCachedStorage},
        tx_exe_manager::{OlaTapeInitInfo, TxExeManager},
    };
    use anyhow::Ok;
    use core::{
        program::{
            binary_program::{BinaryInstruction, BinaryProgram},
            decoder::decode_binary_program_to_instructions,
        },
        vm::{
            hardware::{ContractAddress, OlaStorage},
            types::Event,
        },
    };
    use interpreter::sema::symbol;

    use ola_lang_abi::{Abi, FixedArray4, FixedArray8, Value};
    use std::{
        collections::HashMap,
        fs::File,
        io::BufReader,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn test_program() {
        let mut path = get_test_dir();
        path.push("contracts/vote_simple_bin.json");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let program: BinaryProgram = serde_json::from_reader(reader).unwrap();
        let instructions = decode_binary_program_to_instructions(program).unwrap();
        let mut instruction_map: HashMap<u64, BinaryInstruction> = HashMap::new();
        let mut index: u64 = 0;
        instructions.iter().for_each(|instruction| {
            instruction_map.insert(index, instruction.clone());
            index += instruction.binary_length() as u64;
        });
        // print instructions ordered by keys asend
        let mut keys: Vec<u64> = instruction_map.keys().cloned().collect();
        keys.sort();
        keys.iter().for_each(|key| {
            let instruction = instruction_map.get(key).unwrap();
            println!("{}: {:?}", key, instruction);
        });
    }

    #[test]
    fn test_u256() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 43981];
        deploy(&mut writer, "contracts/u256_basic_bin.json", address).unwrap();
        let calldata = vec![0u64, 2590488802];
        let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
        println!("events: {:?}", events)
    }

    #[test]
    fn test_simple_vote() {
        let mut writer = get_writer().unwrap();
        let address = [991, 992, 993, 994];
        deploy(&mut writer, "contracts/vote_simple_bin.json", address).unwrap();
        let init_calldata = vec![7, 1, 2, 3, 4, 5, 6, 7, 8, 3826510503];
        let _ = invoke(&mut writer, address, init_calldata, Some(0), None, None).unwrap();
        let vote_calldata = vec![4, 1, 597976998];
        let _ = invoke(&mut writer, address, vote_calldata, Some(1), None, None).unwrap();
        let check_calldata = vec![0, 1621094845];
        let result = call(address, check_calldata, None).unwrap();
        println!("result: {:?}", result);
    }

    #[test]
    fn test_storage_u256() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 123456];
        deploy(&mut writer, "contracts/storage_u256_bin.json", address).unwrap();
        let abi_path = "contracts-abi/storage_u256_abi.json";
        let mut path = get_test_dir();
        path.push(abi_path);
        let abi: Abi = {
            let file = File::open(path).expect("failed to open ABI file");

            serde_json::from_reader(file).expect("failed to parse ABI")
        };
        {
            let func = abi.functions[0].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[1].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[2].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[3].clone();
            // encode input and function selector
            let param = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 77]));
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[4].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[5].clone();
            // encode input and function selector
            let param = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 88]));
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[6].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[7].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[8].clone();
            // encode input and function selector
            let param_0 = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 99]));
            let param_1 = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 110]));
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param_0, param_1])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[9].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[10].clone();
            // encode input and function selector
            let param_0 = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 130]));
            let param_1 = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 140]));
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param_0, param_1])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[11].clone();
            let param_0 = Value::U256(FixedArray8([0, 0, 0, 0, 0, 0, 0, 130]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param_0])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
    }

    #[test]
    fn test_u32_storage() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 123456];
        deploy(&mut writer, "contracts/storage_u32_bin.json", address).unwrap();
        let abi_path = "contracts-abi/storage_u32_abi.json";
        let mut path = get_test_dir();
        path.push(abi_path);
        let abi: Abi = {
            let file = File::open(path).expect("failed to open ABI file");

            serde_json::from_reader(file).expect("failed to parse ABI")
        };
        {
            let func = abi.functions[1].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
    }

    #[test]
    fn test_book_event() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 1234599];
        deploy(&mut writer, "contracts/books_bin.json", address).unwrap();
        let abi_path = "contracts-abi/books_abi.json";
        let mut path = get_test_dir();
        path.push(abi_path);
        let abi: Abi = {
            let file = File::open(path).expect("failed to open ABI file");

            serde_json::from_reader(file).expect("failed to parse ABI")
        };
        {
            let func = abi.functions[0].clone();
            let param_0 = Value::U32(12);
            let param_1 = Value::String("hello".to_string());
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[param_0, param_1])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
    }

    #[test]
    fn test_erc20() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 1234588];
        deploy(&mut writer, "contracts/erc20_bin.json", address).unwrap();
        let abi_path = "contracts-abi/erc20_abi.json";
        let mut path = get_test_dir();
        path.push(abi_path);
        let abi: Abi = {
            let file = File::open(path).expect("failed to open ABI file");
            serde_json::from_reader(file).expect("failed to parse ABI")
        };
        {
            let func = abi.functions[0].clone();
            let name = Value::String("OlaToken".to_string());
            let symbol = Value::String("OLA".to_string());
            let decimal = Value::U32(2);
            let total_supply = Value::U32(1000000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(
                    func.signature().as_str(),
                    &[name, symbol, decimal, total_supply],
                )
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        {
            let func = abi.functions[1].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        {
            let func = abi.functions[2].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        {
            let func = abi.functions[3].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        {
            let func = abi.functions[4].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        {
            let func = abi.functions[5].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("owner result: {:?}", result)
        }
        {
            let func = abi.functions[6].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        // mint
        {
            let func = abi.functions[7].clone();
            let to = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let value = Value::U32(1000000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[to, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        // burn
        {
            let func = abi.functions[8].clone();
            let from = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let value = Value::U32(1000000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[from, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        // transfer
        {
            let func = abi.functions[9].clone();
            let to = Value::Address(FixedArray4([2001, 2002, 2003, 2005]));
            let value = Value::U32(300000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[to, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
        // balanceOf Owner
        {
            let func = abi.functions[6].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        // balanceOf to
        {
            let func = abi.functions[6].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2005]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }
        // owner approve to spender
        {
            let func = abi.functions[10].clone();
            let spender = Value::Address(FixedArray4([2001, 2002, 2003, 2006]));
            let value = Value::U32(200000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[spender, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }

        // allowance
        {
            let func = abi.functions[11].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let spender = Value::Address(FixedArray4([2001, 2002, 2003, 2006]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner, spender])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }

        // approve to burn
        {
            let func = abi.functions[8].clone();
            let from = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let caller = Some([2001, 2002, 2003, 2006]);
            let value = Value::U32(100000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[from, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), caller, None).unwrap();
            println!("events: {:?}", events)
        }

        // approve to transferFrom
        {
            let func = abi.functions[12].clone();
            let from = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let caller = Some([2001, 2002, 2003, 2006]);
            let to = Value::Address(FixedArray4([2001, 2002, 2003, 2005]));
            let value = Value::U32(100000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[from, to, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), caller, None).unwrap();
            println!("events: {:?}", events)
        }

        // allowance
        {
            let func = abi.functions[11].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let spender = Value::Address(FixedArray4([2001, 2002, 2003, 2006]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner, spender])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }

        // balanceOf Owner
        {
            let func = abi.functions[6].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }

        // balanceOf to
        {
            let func = abi.functions[6].clone();
            let owner = Value::Address(FixedArray4([2001, 2002, 2003, 2005]));
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[owner])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }

        // totalSupply
        {
            let func = abi.functions[4].clone();
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[])
                .unwrap();
            println!("input: {:?}", calldata);
            let result = call(address, calldata, None).unwrap();
            println!("result: {:?}", result)
        }

    }

    #[test]
    fn test_erc20_revert() {
        let mut writer = get_writer().unwrap();
        let address = [0, 0, 0, 1234588];
        deploy(&mut writer, "contracts/erc20_bin.json", address).unwrap();
        let abi_path = "contracts-abi/erc20_abi.json";
        let mut path = get_test_dir();
        path.push(abi_path);
        let abi: Abi = {
            let file = File::open(path).expect("failed to open ABI file");

            serde_json::from_reader(file).expect("failed to parse ABI")
        };
        // mint
        {
            let func = abi.functions[7].clone();
            let to = Value::Address(FixedArray4([2001, 2002, 2003, 2004]));
            let value = Value::U32(100000000000000);
            // encode input and function selector
            let calldata = abi
                .encode_input_with_signature(func.signature().as_str(), &[to, value])
                .unwrap();
            println!("input: {:?}", calldata);
            let events = invoke(&mut writer, address, calldata, Some(0), None, None).unwrap();
            println!("events: {:?}", events)
        }
    }

    fn call(
        address: ContractAddress,
        calldata: Vec<u64>,
        block: Option<BlockExeInfo>,
    ) -> anyhow::Result<Vec<u64>> {
        let mut storage = get_storage().unwrap();
        let block_info = match block {
            Some(block) => block,
            None => BlockExeInfo {
                block_number: 0,
                block_timestamp: 0,
                sequencer_address: [1001, 1002, 1003, 1004],
                chain_id: 1027,
            },
        };
        let tx = OlaTapeInitInfo {
            version: 0,
            origin_address: [0, 0, 0, 0],
            calldata,
            nonce: None,
            signature_r: None,
            signature_s: None,
            tx_hash: None,
        };
        let mut tx_exe_manager: TxExeManager =
            TxExeManager::new(ExecuteMode::Debug, block_info, tx, &mut storage, address, 0);
        tx_exe_manager.call()
    }

    fn invoke(
        writer: &mut DiskStorageWriter,
        address: ContractAddress,
        calldata: Vec<u64>,
        nonce: Option<u64>,
        caller: Option<ContractAddress>,
        block: Option<BlockExeInfo>,
    ) -> anyhow::Result<Vec<Event>> {
        let mut storage = get_storage().unwrap();

        let block_info = match block {
            Some(block) => block,
            None => BlockExeInfo {
                block_number: 0,
                block_timestamp: 0,
                sequencer_address: [1001, 1002, 1003, 1004],
                chain_id: 1027,
            },
        };
        let tx = OlaTapeInitInfo {
            version: 0,
            origin_address: caller.unwrap_or([2001, 2002, 2003, 2004]),
            calldata,
            nonce,
            signature_r: None,
            signature_s: None,
            tx_hash: None,
        };
        let mut tx_exe_manager: TxExeManager =
            TxExeManager::new(ExecuteMode::Debug, block_info, tx, &mut storage, address, 0);
        let result = tx_exe_manager.invoke()?;
        storage.on_tx_success();
        let cached = storage.get_cached_modification();
        for (key, value) in cached {
            writer.save(key, value)?;
        }
        Ok(result.events)
    }

    fn deploy_system_contracts(writer: &mut DiskStorageWriter) -> anyhow::Result<()> {
        [
            (ADDR_U64_ENTRYPOINT, "system/Entrypoint.json"),
            (ADDR_U64_CODE_STORAGE, "system/AccountCodeStorage.json"),
            (ADDR_U64_NONCE_HOLDER, "system/NonceHolder.json"),
            (
                ADDR_U64_KNOWN_CODES_STORAGE,
                "system/KnownCodesStorage.json",
            ),
            (ADDR_U64_CONTRACT_DEPLOYER, "system/ContractDeployer.json"),
            (ADDR_U64_DEFAULT_ACCOUNT, "system/DefaultAccount.json"),
            (ADDR_U64_SYSTEM_CONTEXT, "system/SystemContext.json"),
        ]
        .into_iter()
        .for_each(|(addr, relative_path)| {
            println!("start deploy {}", relative_path);
            deploy(writer, relative_path, addr).unwrap();
        });
        Ok(())
    }

    fn deploy(
        writer: &mut DiskStorageWriter,
        relative_path: &str,
        address: ContractAddress,
    ) -> anyhow::Result<()> {
        let mut path = get_test_dir();
        path.push(relative_path);
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let program: BinaryProgram = serde_json::from_reader(reader)?;
        writer.save_program(program, address)
    }

    fn get_storage() -> anyhow::Result<OlaCachedStorage> {
        let block_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;
        let storage = OlaCachedStorage::new(get_db_path(), Some(block_timestamp))?;
        Ok(storage)
    }

    fn get_writer() -> anyhow::Result<DiskStorageWriter> {
        let writer = DiskStorageWriter::new(get_db_path())?;
        Ok(writer)
    }

    fn get_db_path() -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("db_test");
        path.into_os_string().into_string().unwrap()
    }

    fn get_test_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test");
        path
    }
}
