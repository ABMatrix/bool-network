use mock::account::{Account, AccountData, AccountResource, ALICE, BOB, GENESIS_KEYPAIR};
use mock::common::*;
use mock::compile::*;
use mock::executor::FakeExecutor;
use mock::*;
use vm::{bytecode_verifier::VerifiedModule, types::transaction::SignedTransaction};

use lazy_static::lazy_static;
use stdlib::stdlib_modules;

lazy_static! {
    pub static ref GENESIS_ACCOUNT: Account =
        { Account::with_keypair(GENESIS_KEYPAIR.0.clone(), GENESIS_KEYPAIR.1.clone()) };
    pub static ref ALICE_ACCOUNT: Account =
        { Account::with_keypair(ALICE.0.clone(), ALICE.1.clone()) };
    pub static ref BOB_ACCOUNT: Account = { Account::with_keypair(BOB.0.clone(), BOB.1.clone()) };
}

fn create_account() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new_with_account(GENESIS_ACCOUNT.clone(), 2_000_000, 1);

    // add module
    //	for module in stdlib_modules() {
    //		let compiledModule = module.as_inner();
    //		println!("module {:?}", compiledModule.self_id());
    //		executor.add_module(&compiledModule.self_id(), compiledModule);
    //	}

    executor.add_account_data(&sender);
    let new_account = Account::new();

    let initial_amount = 1000;
    let txn = create_account_txn(sender.account(), &new_account, 1, initial_amount);

    println!("txn : {:?}", txn);
    let output = executor.execute_block(vec![txn]);
    let txn_output = output.get(0).expect("must have a transaction output");

    println!("output : {:?}", txn_output);
    println!("write set {:?}", txn_output.write_set());
}

fn simple_arithmetic() {
    let program = String::from(
        "
        modules:
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
        script:
        import 0x0000000000000000000000000000000000000000000000000000000000000000.M;

        main() {
            let a: u64;
            let b: u64;
            let c: u64;
            let d: u64;

            a = 10;
            b = 2;
            c = M.max(copy(a), copy(b));
            d = M.sum(copy(a), copy(b));
            assert(copy(c) == 10, 42);
            assert(copy(d) == 12, 42);
            return;
        }
        ",
    );
    let ret = compile_and_execute(&program, vec![]);
    println!("ret : {:?}", ret);
}

fn upload_script_and_execute() {
    let mut executor = FakeExecutor::from_genesis_file();

    let sender = AccountData::new_with_account(ALICE_ACCOUNT.clone(), 2_000_000, 1);
    executor.add_account_data(&sender);

    let program = String::from(
        "
        module M {
            public max(a: u64, b: u64): u64 {
                if (copy(a) > copy(b)) {
                    return copy(a);
                } else {
                    return copy(b);
                }
                return 0;
            }

            public sum(a: u64, b: u64): u64 {
                let c: u64;
                c = copy(a) + copy(b);
                return copy(c);
            }
        }
		",
    );

    // compile must with address. why?
    let compiled_program = compile_inner_program_with_address(sender.address(), &program);
    let program = compile_program_with_address(sender.address(), &program, vec![]);

    // make transaction
    let txn = sender.account().create_signed_txn_with_program(
        program,
        1,
        gas_costs::TXN_RESERVED, // this is a default for gas
        0,                       // this is a default for gas
    );

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![txn];
    let output = executor.execute_block(txns);
    let txn_output = output.get(0).expect("must have a transaction output");
    println!("output {:?}", txn_output);

    executor.apply_write_set(txn_output.write_set());

    // call module
    let program = String::from(
        "
        import 0x826b443135b47a8bfdf090f0088a427aeae1de70c6cf833823715c07539bcc3a.M;

        main() {
            let a: u64;
            let b: u64;
            let c: u64;
            let d: u64;

            a = 10;
            b = 2;
            c = M.max(copy(a), copy(b));
            d = M.sum(copy(a), copy(b));
            assert(copy(c) == 10, 42);
            assert(copy(d) == 12, 42);
            return;
        }
		",
    );

    let deps: Vec<VerifiedModule> = compiled_program
        .modules
        .into_iter()
        .map(|m| VerifiedModule::new(m).unwrap())
        .collect();
    let program = compile_program_with_deps(sender.address(), &program, vec![], deps);

    // make transaction
    let txn = sender.account().create_signed_txn_with_program(
        program,
        2,
        gas_costs::TXN_RESERVED, // this is a default for gas
        0,                       // this is a default for gas
    );

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![txn];
    let output = executor.execute_block(txns);
    let txn_output = output.get(0).expect("must have a transaction output");
    println!("output {:?}", txn_output);
}

fn transfer() {
    let mut executor = FakeExecutor::from_genesis_file();

    let sender = AccountData::new_with_account(ALICE_ACCOUNT.clone(), 2_000_000, 1);
    let receiver = AccountData::new_with_account(BOB_ACCOUNT.clone(), 50_000, 1);

    executor.add_account_data(&sender);

    executor.add_account_data(&receiver);

    let value = executor.read_account_resource(sender.account());
    if let Some(value) = value {
        println!("value: {:?}", AccountResource::read_balance(&value));
    }

    let value = executor.read_account_resource(receiver.account());
    if let Some(value) = value {
        println!("value: {:?}", AccountResource::read_balance(&value));
    }

    // transfer
    let transfer_amount = 1_000;
    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 1, transfer_amount);

    println!("{:?}", txn);

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![txn];
    let output = executor.execute_block(txns);
    let txn_output = output.get(0).expect("must have a transaction output");

    println!("{:?}", output);

    executor.apply_write_set(txn_output.write_set());

    let gas = txn_output.gas_used();
    let sender_balance = 2_000_000 - transfer_amount - gas;
    let receiver_balance = 50_000 + transfer_amount;

    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_receiver = executor
        .read_account_resource(receiver.account())
        .expect("receiver must exist");

    assert_eq!(
        receiver_balance,
        AccountResource::read_balance(&updated_receiver)
    );
    assert_eq!(
        sender_balance,
        AccountResource::read_balance(&updated_sender)
    );
}
fn main() {
    //	create_account();
    // simple_arithmetic();
    //	transfer();
    upload_script_and_execute();
}
