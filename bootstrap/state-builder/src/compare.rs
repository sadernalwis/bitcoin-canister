use bitcoin::OutPoint as BitcoinOutPoint;
use ic_btc_canister::types::{Address, AddressUtxo, OutPoint, Storable as _, TxOut};
use ic_btc_types::Height;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl, FileMemory, StableBTreeMap,
};
use std::fs::File;

fn main() {
    let canister_mem = FileMemory::new(File::open("./testnet_100k_reference.bin").unwrap());
    let memory_manager = MemoryManager::init(canister_mem);

    let canister_mem_2 = FileMemory::new(File::open("./full_canister_100k_combined").unwrap());
    let memory_manager_2 = MemoryManager::init(canister_mem_2);

    /*println!(
        "memory sizes: {:?}",
        memory_manager_2.inner.borrow().memory_sizes_in_pages
    );
    println!(
        "buckets: {:?}",
        memory_manager_2.inner.borrow().memory_buckets
    );*/

//    let balances_script_mem =
//        FileMemory::new(File::open("./canister_testnet_100k/balances").unwrap());
    let balances_script_mem = memory_manager_2.get(MemoryId::new(4));

    let mut buf = vec![1; 1000];
    balances_script_mem.read(0, &mut buf);
    println!("balances first few bytes: {:?}", buf);

    let mut balances: StableBTreeMap<_, Address, u64> =
        StableBTreeMap::init(balances_script_mem, 90, 8);

    println!("# balances {}", balances.len());

    let balance_mem_reference = memory_manager.get(MemoryId::new(4));

    let mut balances_reference: StableBTreeMap<_, Address, u64> =
        StableBTreeMap::init(balance_mem_reference, 90, 8);

    println!("# balances in ref {}", balances_reference.len());

    assert_eq!(balances_reference.len(), balances.len());

    for ((k1, v1), (k2, v2)) in std::iter::zip(balances.iter(), balances_reference.iter()) {
        assert_eq!(k1, k2);
        assert_eq!(v1, v2);
    }

    println!("balances match perfectly");

    let address_utxos_reference = memory_manager.get(MemoryId::new(1));
    use ic_stable_structures::Memory;

    let address_utxos_reference: StableBTreeMap<_, AddressUtxo, ()> = StableBTreeMap::init(
        address_utxos_reference,
        90 + 36, // max outpoint size.
        0,       // No values are stored in the map.
    );

//    let address_utxos =
  //      FileMemory::new(File::open("./canister_testnet_100k/address_utxos").unwrap());
    let address_utxos = memory_manager_2.get(MemoryId::new(1));
    let mut buf = vec![1; 10];
    address_utxos.read(0, &mut buf);

    //println!("address utxos first few bytes: {:?}", buf);

    let mut address_utxos: StableBTreeMap<_, AddressUtxo, ()> =
        StableBTreeMap::init(address_utxos, 90, 8);

    println!("# address utxos: {}", address_utxos.len());
    println!(
        "# address utxos reference: {}",
        address_utxos_reference.len()
    );

    for ((k1, v1), (k2, v2)) in std::iter::zip(address_utxos.iter(), address_utxos_reference.iter())
    {
        assert_eq!(k1, k2);
        assert_eq!(v1, v2);
    }

    println!("address utxos match perfectly.");

//    let small_utxos = FileMemory::new(File::open("./canister_testnet_100k/small_utxos").unwrap());
    let small_utxos = memory_manager_2.get(MemoryId::new(2));

    let mut small_utxos: StableBTreeMap<_, Vec<u8>, Vec<u8>> =
        StableBTreeMap::init(small_utxos, 0, 0);

    let small_utxos_reference = memory_manager.get(MemoryId::new(2));
    let mut small_utxos_reference: StableBTreeMap<_, Vec<u8>, Vec<u8>> =
        StableBTreeMap::init(small_utxos_reference, 0, 0);

    println!("# small utxos: {}", small_utxos.len());
    println!("# small utxos referenced: {}", small_utxos_reference.len());

    for (i, ((k1, v1), (k2, v2))) in
        std::iter::zip(small_utxos.iter(), small_utxos_reference.iter()).enumerate()
    {
        if k1 != k2 {
            let k1 = OutPoint::from_bytes(k1.clone());
            let k2 = OutPoint::from_bytes(k2.clone());
            println!("reference: {:?}", k1);
            println!("script: {:?}", k2);
        }
        assert_eq!(k1, k2);
        if v1 != v2 {
            let v1 = <(TxOut, Height)>::from_bytes(v1.clone());
            let v2 = <(TxOut, Height)>::from_bytes(v2.clone());
            println!("reference: {:?}", v2);
            println!("script: {:?}", v1);
        }
        assert_eq!(v1, v2);
    }

    //let medium_utxos = memory_manager_2.get(MemoryId::new(3));
    let medium_utxos = memory_manager_2.get(MemoryId::new(3));
    let mut medium_utxos: StableBTreeMap<_, Vec<u8>, Vec<u8>> =
        StableBTreeMap::init(medium_utxos, 0, 0);

    let medium_utxos_reference = memory_manager.get(MemoryId::new(3));
    let mut medium_utxos_reference: StableBTreeMap<_, Vec<u8>, Vec<u8>> =
        StableBTreeMap::init(medium_utxos_reference, 0, 0);

    println!("# medium utxos: {}", medium_utxos.len());
    println!(
        "# medium utxos referenced: {}",
        medium_utxos_reference.len()
    );

    for (i, ((k1, v1), (k2, v2))) in
        std::iter::zip(medium_utxos.iter(), medium_utxos_reference.iter()).enumerate()
    {
        let k1 = OutPoint::from_bytes(k1);
        let k2 = OutPoint::from_bytes(k2);
        if k1 != k2 {
            println!("{:?}, {:?}", k1, k2);

            let v1 = <(TxOut, Height)>::from_bytes(v1.clone());
            println!(
                "script: {:?}",
                bitcoin::Script::from(v1.clone().0.script_pubkey)
            );
            println!(
                "is provably unspendable? {:?}",
                bitcoin::Script::from(v1.clone().0.script_pubkey).is_provably_unspendable()
            );

            /*let k1: BitcoinOutPoint = BitcoinOutPoint {
                txid: bitcoin::Txid::from_hash(
                    bitcoin::hashes::Hash::from_slice(k1.txid.as_bytes())
                        .expect("txid must be valid"),
                ),
                vout: k1.vout,
            };
            let k2: BitcoinOutPoint = BitcoinOutPoint {
                txid: bitcoin::Txid::from_hash(
                    bitcoin::hashes::Hash::from_slice(k2.txid.as_bytes())
                        .expect("txid must be valid"),
                ),
                vout: k2.vout,
            };

            println!(
                "is provably unspendable: {:?}",
                k1.txid.script_pubkey.is_provably_unspendable()
            );*/
        }
        assert_eq!(k1, k2);
        /*if v1 != v2 {
            println!("{:?}, {:?}", k1, k2);
            println!("{:?}, {:?}", v1, v2);
        }*/

        let v1 = <(TxOut, Height)>::from_bytes(v1.clone());
        let v2 = <(TxOut, Height)>::from_bytes(v2.clone());
        assert_eq!(v1.0.value, v2.0.value);
        assert_eq!(v1.1, v2.1);
        // ignoring script discepancies for now.
    }
}
