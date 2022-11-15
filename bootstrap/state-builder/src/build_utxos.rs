//! A script for building the Bitcoin canister's state and storing it into a file.
//!
//! Example run:
//!
//! FIXME
//! cargo run --bin state-builder --release -- \
//!     --state-path data.bin \
//!     --network testnet \
//!     --blocks-path /path/to/data/testnet3 \
//!     --tip 000000002ce019cc4a8f2af62b3ecf7c30a19d29828b25268a0194dbac3cac50
use bitcoin::{
    consensus::Decodable, Address, Block as BitcoinBlock, BlockHash, BlockHeader,
    Txid as BitcoinTxid,
};
use clap::Parser;
use ic_btc_canister::{
    types::{Address as OurAddress, AddressUtxo, Config, Network, OutPoint, TxOut, Txid},
    with_state_mut,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, Memory, StableBTreeMap,
};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    str::FromStr,
};

const WASM_PAGE_SIZE: u64 = 65536;

#[derive(Parser, Debug)]
struct Args {
    /// The path of the UTXOs dump.
    #[clap(long, value_hint = clap::ValueHint::DirPath)]
    utxos_dump_path: PathBuf,

    /// The path to store the state in.
    #[clap(long, value_hint = clap::ValueHint::DirPath)]
    output: PathBuf,

    /// The bitcoin network.
    #[clap(long)]
    network: Network,
}

fn write_mem_to_file(path: &PathBuf, memory_id: MemoryId) {
    let canister_mem = ic_btc_canister::get_memory().with(|m| m.clone());
    let memory_manager = MemoryManager::init(canister_mem);

    let memory = memory_manager.get(memory_id);

    let mut memory_vec = vec![0; (memory.size() * WASM_PAGE_SIZE).try_into().unwrap()];

    memory.read(0, &mut memory_vec);

    let mut file = match File::create(path) {
        Err(err) => panic!("couldn't create {}: {}", path.display(), err),
        Ok(file) => file,
    };

    match file.write_all(&memory_vec) {
        Err(err) => panic!("couldn't write to {}: {}", path.display(), err),
        Ok(_) => println!("successfully wrote to {}", path.display()),
    };
}

fn main() {
    let args = Args::parse();

    // Read the UTXOs from the UTXOs dump.
    let utxos_file = File::open(args.utxos_dump_path).unwrap();
    let reader = BufReader::new(utxos_file);

    ic_btc_canister::init(Config {
        network: args.network,
        ..Config::default()
    });

    with_state_mut(|s| {
        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            let parts: Vec<_> = line.split(',').collect();

            let txid = Txid::from(BitcoinTxid::from_str(parts[1]).unwrap().to_vec());
            let vout: u32 = parts[2].parse().unwrap();
            let amount: u64 = parts[3].parse().unwrap();
            let address_str = parts[5];
            let script = parts[6];
            let height: u32 = parts[9].parse().unwrap();
            let mut script = hex::decode(script).unwrap();

            if i % 100_000 == 0 {
                println!("Processed {}", i);
            }

            if let Ok(address) = Address::from_str(parts[5]) {
                // update the script.
                script = address.script_pubkey().as_bytes().to_vec();
            }

            // Insert the UTXO
            let outpoint = OutPoint { txid, vout };

            if !bitcoin::Script::from(script.clone()).is_provably_unspendable() {
                let txout = TxOut {
                    value: amount,
                    script_pubkey: script,
                };

                let x = s.utxos.utxos.insert(outpoint, (txout, height));
                assert!(!x); // not seen this utxo before.
            }
        }
    });

    let mut p = args.output.clone();
    p.push("small_utxos");
    write_mem_to_file(&p, MemoryId::new(2));
    let mut p = args.output.clone();
    p.push("medium_utxos");
    write_mem_to_file(&p, MemoryId::new(3));

    // TODO: also save large UTXOs
}
