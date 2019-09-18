use crate::account::{Account, AccountData, AccountResource, ALICE, BOB, GENESIS_KEYPAIR};
use crypto::{signing, PrivateKey, PublicKey, signing::KeyPair};
use stdlib::{
    stdlib_modules,
    transaction_scripts::{
        CREATE_ACCOUNT_TXN_BODY, MINT_TXN_BODY, PEER_TO_PEER_TRANSFER_TXN_BODY,
        ROTATE_AUTHENTICATION_KEY_TXN_BODY,
    },
};
use vm_cache_map::Arena;
use crate::data_store::FakeDataStore;
use vm::{
    types::{
        AccessPath,
        AccountAddress,
        account_config,
        ByteArray,
        transaction::{
            Program, RawTransaction, SignatureCheckedTransaction, TransactionArgument,
        },
        SCRIPT_HASH_LENGTH,
        write_set::{WriteOp, WriteSet}
    },
    def::{access::ModuleAccess, transaction_metadata::TransactionMetadata},
    vm_runtime::{
        code_cache::{
            module_adapter::FakeFetcher,
            module_cache::{BlockModuleCache, VMModuleCache},
        },
        data_cache::BlockDataCache,
        txn_executor::{TransactionExecutor, ACCOUNT_MODULE, COIN_MODULE},
    },
    vm_runtime::vm_runtime_types::value::Local,
};

pub fn create_genesis_write_set(
    private_key: &PrivateKey,
    public_key: PublicKey,
) -> WriteSet {
    // TODO: Currently validator set is unused because MoveVM doesn't support collections for now.
    //       Fix it later when we have collections.

    const INIT_BALANCE: u64 = 1_000_000_000;
    // Compile the needed stdlib modules.
    let modules = stdlib_modules();
    let arena = Arena::new();
    let state_view = FakeDataStore::default();
    let vm_cache = VMModuleCache::new(&arena);
    let genesis_addr = account_config::association_address();
    let genesis_auth_key = ByteArray::new(AccountAddress::from(public_key).to_vec());

    let genesis_write_set = {
        let fake_fetcher = FakeFetcher::new(modules.iter().map(|m| m.as_inner().clone()).collect());
        let data_cache = BlockDataCache::new(&state_view);
        let block_cache = BlockModuleCache::new(&vm_cache, fake_fetcher);
        {
            let mut txn_data = TransactionMetadata::default();
            txn_data.sender = genesis_addr;

            let mut txn_executor = TransactionExecutor::new(&block_cache, &data_cache, txn_data);
            txn_executor.create_account(genesis_addr).unwrap().unwrap();
            txn_executor
                .execute_function(&COIN_MODULE, "initialize", vec![])
                .unwrap()
                .unwrap();

            txn_executor
                .execute_function(
                    &ACCOUNT_MODULE,
                    "mint_to_address",
                    vec![Local::address(genesis_addr), Local::u64(INIT_BALANCE)],
                )
                .unwrap()
                .unwrap();

            txn_executor
                .execute_function(
                    &ACCOUNT_MODULE,
                    "rotate_authentication_key",
                    vec![Local::bytearray(genesis_auth_key)],
                )
                .unwrap()
                .unwrap();

            let stdlib_modules = modules
                .iter()
                .map(|m| {
                    let mut module_vec = vec![];
                    m.serialize(&mut module_vec).unwrap();
                    (m.self_id(), module_vec)
                })
                .collect();

            txn_executor
                .make_write_set(stdlib_modules, Ok(Ok(())))
                .unwrap()
                .write_set()
                .clone()
                .into_mut()
        }
    };
    genesis_write_set.freeze().unwrap()
}