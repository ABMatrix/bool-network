use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use support::{StorageMap};
use system::ensure_signed;

#[macro_use]
extern crate mirai_annotations;
#[macro_use]
extern crate rental;

pub mod def;
pub mod vm_runtime;
pub mod types;
pub mod state_view;
pub mod bytecode_verifier;

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
		/// code storage
		pub CodeStorage: map Hash => Option<MoveModule>;
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

		pub fn put_code(origin, code: Vec<u8>) -> Result {
			println!("into put code");
			Ok(())
		}

		pub fn call(origin, dest: Vec<u8>, value: Balance, gas_limit: Gas, data: Vec<u8>) -> Result {
			println!("into call");
			Ok(())
		}

		pub fn create(origin, code_hash: Hash, value: Balance, gas_limit: Gas, data: Vec<u8>) -> Result {
			println!("into create");
			Ok(())
		}

		pub fn execute(origin, transaction: Vec<u8>) -> Result {
			println!("execute transaction");
			Ok(())
		}
	}
}