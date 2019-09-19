use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use support::{StorageMap, ensure};
use system::{ensure_signed};
use primitives::traits::As;

mod exec;
mod store;
mod tests;

use vm::types::transaction::SignedTransaction;
use canonical_serialization::{
    CanonicalSerialize, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use exec::Executor;
use mock::account::{Account, AccountData};

type Balance = Vec<u8>;
type Gas = u64;
type MoveModule = Vec<u8>;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Vmove {
		/// code storage
		HasGenesis get(has_genesis): bool;
		CodeStorage: map Vec<u8> => MoveModule;
		pub AccessStorage get(access_storage): map Vec<u8> => Option<Vec<u8>>;
		AccessBalance get(balance): map Vec<u8> => u64;
		AccessSequence get(sequence): map Vec<u8> => u64;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// code with storage hash
		CodeStored(Vec<u8>),

		/// An event from contract of account.
		Contract(AccountId, Vec<u8>),
	}
);

/// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

//		pub fn put_code(origin, code: Vec<u8>) -> Result {
//			println!("into put code");
//			Ok(())
//		}
//
//		pub fn call(origin, dest: Vec<u8>, value: Balance, gas_limit: Gas, data: Vec<u8>) -> Result {
//			println!("into call");
//			Ok(())
//		}
//
//		pub fn create(origin, code_hash: Hash, value: Balance, gas_limit: Gas, data: Vec<u8>) -> Result {
//			println!("into create");
//			Ok(())
//		}


		pub fn execute(origin, transaction: Vec<u8>) -> Result {
			let _sender = ensure_signed(origin)?;
			let mut executor = Self::get_executor();
			let txn = SimpleDeserializer::deserialize(&transaction);
			ensure!(txn.is_ok(), "unknown transaction");
			let output = executor.execute_transaction(txn.expect("unknown transaction"));
			println!("output {:?}", output);

			executor.apply_write_set(output.write_set());
			Ok(())
		}

		pub fn create_account(origin, pubkey: Vec<u8>, value: u64) -> Result {
			let sender = ensure_signed(origin)?;
			let mut executor = Self::get_executor();

			// TODO: check pubkey and sender.
			let expected_index = <system::Module<T>>::account_nonce(sender);
//			let account = AccountData::new_with_account(Account::mock(&pubkey), value, As::<u64>::as_(expected_index));
			let account = AccountData::new_with_account(Account::mock(&pubkey), value, 0);
			executor.add_account_data(&account);
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn get_executor() -> Executor<T> {
		let executor;
		if !<HasGenesis<T>>::get() {
			executor = Executor::from_genesis_file();
		} else {
			executor = Executor::no_genesis();
			<HasGenesis<T>>::put(true);
		}
		executor
	}

}