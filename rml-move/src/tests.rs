#![cfg(test)]

use crate::{Trait};

use runtime_io::with_externalities;
use substrate_primitives::{H256, Blake2Hasher};
use support::{impl_outer_origin, assert_ok};
use primitives::{
	BuildStorage,
	traits::{BlakeTwo256, IdentityLookup},
	testing::{Digest, DigestItem, Header}
};
use canonical_serialization::{
    SimpleDeserializer, SimpleSerializer,
};
use mock::*;
use mock::account::{Account, AccountData, AccountResource, ALICE, BOB, GENESIS_KEYPAIR};
use mock::common::*;
use mock::compile::*;
use lazy_static::lazy_static;
use crate::{Module, AccessStorage};
use crate::exec::Executor;

impl_outer_origin! {
	pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

impl system::Trait for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Digest = Digest;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type Log = DigestItem;
}
impl Trait for Test {
	type Event = ();
}
type ExecutorModule = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
}


lazy_static! {
	pub static ref GENESIS_ACCOUNT: Account = { Account::with_keypair(GENESIS_KEYPAIR.0.clone(), GENESIS_KEYPAIR.1.clone()) };
	pub static ref ALICE_ACCOUNT: Account = { Account::with_keypair(ALICE.0.clone(), ALICE.1.clone()) };
	pub static ref BOB_ACCOUNT: Account = { Account::with_keypair(BOB.0.clone(), BOB.1.clone()) };
}


#[test]
fn test_transfer() {
	with_externalities(&mut new_test_ext(), || {
		// make transaction

		let mut executor = ExecutorModule::get_executor();
		let sender = AccountData::new_with_account(ALICE_ACCOUNT.clone(), 2_000_000, 0);
		let receiver = AccountData::new_with_account(BOB_ACCOUNT.clone(), 0, 0);
		executor.add_account_data(&sender);
		let value = executor.read_account_resource(sender.account()).unwrap();
		assert_eq!(AccountResource::read_balance(&value), 2_000_000);


		// transfer
		let transfer_amount = 1_000;
		let txn = peer_to_peer_txn(sender.account(), receiver.account(), 0, transfer_amount);

		let tx_bytes = SimpleSerializer::<Vec<u8>>::serialize(&txn).expect("should serialize ok.");
		assert_ok!(ExecutorModule::execute(Origin::signed(1), tx_bytes));

		// get resource
		let ap = receiver.account().make_access_path();
		let bytes = bincode::serialize(&ap).expect("serialization failed");
		let blob = ExecutorModule::access_storage(bytes).unwrap();
		let account_type = executor.struct_def();
		let value = Account::read_account_resource(&blob, account_type).unwrap();
		assert_eq!(AccountResource::read_balance(&value), transfer_amount);

		let value = executor.read_account_resource(sender.account()).unwrap();
		assert_eq!(AccountResource::read_balance(&value), 1_999_000);
	});
}