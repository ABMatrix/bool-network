use failure;
use vm::{
	state_view::StateView,
	types::{
	AccessPath,
	ModuleId,
	transaction::{SignedTransaction, TransactionPayload},
	write_set::{WriteOp, WriteSet},
	},
	def::{errors::*, file_format::CompiledModule},
	vm_runtime::data_cache::RemoteCache,
};

use mock::account::{Account, AccountData, AccountResource};
use crate::{Trait, AccessStorage, AccessBalance, AccessSequence};
use support::{StorageMap};
use std::marker::PhantomData;
use crate::exec::get_account_struct_def;

#[derive(Debug)]
pub struct AccessStore<T> {
	p: PhantomData<T>,
}

impl<T: Trait> Default for AccessStore<T> {
	fn default() -> AccessStore<T> {
		AccessStore{
			p: PhantomData
		}
	}
}

impl<T: Trait> AccessStore<T> {
	/// Adds a [`WriteSet`] to this data store.
	pub fn add_write_set(&self, write_set: &WriteSet) {
		for (access_path, write_op) in write_set {
			match write_op {
				WriteOp::Value(blob) => {
					self.set(access_path.clone(), blob.clone());
				}
				WriteOp::Deletion => {
					self.remove(access_path);
				}
			}
		}
	}

	pub fn set(&self, access_path: AccessPath, data_blob: Vec<u8>) {
		// serialize
		let bytes = bincode::serialize(&access_path).expect("serialization failed");
		if access_path.is_resource_path() {
			let account_type = get_account_struct_def();
			match Account::read_account_resource(&data_blob, account_type) {
				Some(value) => {
					<AccessBalance<T>>::insert(access_path.address.to_vec() ,AccountResource::read_balance(&value));
					<AccessSequence<T>>::insert(access_path.address.to_vec() ,AccountResource::read_sequence_number(&value));
				},
				None => {},
			}

		}
		<AccessStorage<T>>::insert(bytes, data_blob);
	}

	pub fn get_(&self, access_path: &AccessPath) -> failure::Result<Option<Vec<u8>>> {
		// Since the data is in-memory, it can't fail.
		let bytes = bincode::serialize(&access_path).expect("serialization failed");

		match <AccessStorage<T>>::get(bytes) {
			None => Ok(None),
			Some(blob) => Ok(Some(blob.clone())),
		}
	}

	pub fn remove(&self, access_path: &AccessPath) {
		let bytes = bincode::serialize(&access_path).expect("serialization failed");
		<AccessStorage<T>>::remove(bytes);
	}

	pub fn add_account_data(&mut self, account_data: &AccountData) {
		match account_data.to_resource().simple_serialize() {
			Some(blob) => {
				self.set(account_data.make_access_path(), blob);
			}
			None => panic!("can't create Account data"),
		}
	}

	pub fn add_module(&mut self, module_id: &ModuleId, module: &CompiledModule) {
		let access_path = AccessPath::from(module_id);
		let mut blob = vec![];
		module
			.serialize(&mut blob)
			.expect("serializing this module should work");
		self.set(access_path, blob);
	}
}

impl<T: Trait> StateView for AccessStore<T> {
	fn get(&self, access_path: &AccessPath) -> failure::Result<Option<Vec<u8>>> {
		self.get_(access_path)
	}

	fn multi_get(&self, _access_paths: &[AccessPath]) -> failure::Result<Vec<Option<Vec<u8>>>> {
		unimplemented!();
	}

	fn is_genesis(&self) -> bool {
		true
	}
}

// This is used by the `process_transaction` API.
impl<T: Trait> RemoteCache for AccessStore<T> {
	fn get(
		&self,
		access_path: &AccessPath,
	) -> ::std::result::Result<Option<Vec<u8>>, VMInvariantViolation> {
		Ok(StateView::get(self, access_path).expect("it should not error"))
	}
}
