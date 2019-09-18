use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use support::{StorageMap};
use system::ensure_signed;

mod exec;
mod store;

use vm::types::transaction::SignedTransaction;
use canonical_serialization::{
    CanonicalSerialize, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use exec::Executor;

type Balance = Vec<u8>;
type Gas = u64;
type Hash = Vec<u8>;
type MoveModule = Vec<u8>;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	// TODO: Add other types and constants required configure this module.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Vm {
		pub Something get(something) : Vec<u8>;
		/// code storage
		pub HasGenesis get(has_genesis): bool;
		pub CodeStorage: map Vec<u8> => MoveModule;
		AccessStorage: map Vec<u8> => Option<Vec<u8>>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// code with storage hash
		CodeStored(Hash),

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
			let transactor = ensure_signed(origin)?;
			let mut executor = Self::get_executor();
			let txn: SignedTransaction = SimpleDeserializer::deserialize(&transaction).unwrap();
			let output = executor.execute_block(vec![txn]);
			let txn_output = output.get(0).expect("must have a transaction output");
			println!("output {:?}", txn_output);

			executor.apply_write_set(txn_output.write_set());
			Ok(())
		}

		pub fn test(origin, key: Vec<u8>, value: Vec<u8>) -> Result {
			let _sender = ensure_signed(origin)?;
			<CodeStorage<T>>::insert(key, value);
			Ok(())
		}

		pub fn do_something(origin, something: Vec<u8>) -> Result {
			let who = ensure_signed(origin)?;

			<Something<T>>::put(something);
			println!("one");
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn get_executor() -> Executor<T> {
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