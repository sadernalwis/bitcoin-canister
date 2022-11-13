use bitcoin::{consensus::Decodable, Address, Block as BitcoinBlock, BlockHash, BlockHeader, Txid};
use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use ic_btc_canister::{
    heartbeat, memory, pre_upgrade, runtime,
    state::main_chain_height,
    types::{
        self, Address as OurAddress, Block, Config, GetSuccessorsCompleteResponse,
        GetSuccessorsResponse, Network,
    },
    unstable_blocks, with_state, with_state_mut, UnstableBlocks,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, FileMemory, Memory, StableBTreeMap,
};
use rusty_leveldb::{Options, DB};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

const WASM_PAGE_SIZE: u64 = 65536;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, value_hint = clap::ValueHint::DirPath)]
    memories_dir: PathBuf,

    #[clap(long, value_hint = clap::ValueHint::DirPath)]
    output: PathBuf,
}

fn write_memory(memory_manager: &MemoryManager<FileMemory>, memory_id: u8, memory: &PathBuf) {
    let dst = memory_manager.get(MemoryId::new(memory_id));

    let src = FileMemory::new(File::open(memory).unwrap());
    dst.grow(src.size());

    let mut buf = vec![0; (src.size() * WASM_PAGE_SIZE) as usize];
    src.read(0, &mut buf);
    dst.write(0, &buf);
}

fn main() {
    let args = Args::parse();
    let f = FileMemory::new(File::create(args.output).unwrap());

    let memory_manager = MemoryManager::init(f);

    let mut p = args.memories_dir.clone();
    p.push("./upgrade");
    write_memory(&memory_manager, 0, &p);

    let mut p = args.memories_dir.clone();
    p.push("./address_utxos");
    write_memory(&memory_manager, 1, &p);

    let mut p = args.memories_dir.clone();
    p.push("./small_utxos");
    write_memory(&memory_manager, 2, &p);

    let mut p = args.memories_dir.clone();
    p.push("./medium_utxos");
    write_memory(&memory_manager, 3, &p);

    let mut p = args.memories_dir.clone();
    p.push("./balances");
    write_memory(&memory_manager, 4, &p);
}