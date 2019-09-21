pub mod account;
pub mod common;
pub mod compile;
pub mod data_store;
pub mod executor;
pub mod gas_costs;
pub mod genesis;

use compiler::Compiler;
use data_store::FakeDataStore;
use failure::prelude::*;
use vm::{
    bytecode_verifier::{VerifiedModule, VerifiedScript},
    def::{
        errors::*,
        file_format::{CompiledModule, CompiledScript},
    },
    types::{
        transaction::{Program, TransactionArgument},
        AccessPath, AccountAddress,
    },
    vm_runtime::{execute_function, static_verify_program},
};

/// Compiles a program with the given arguments and executes it in the VM.
pub fn compile_and_execute(program: &str, args: Vec<TransactionArgument>) -> VMResult<()> {
    let address = AccountAddress::default();
    let compiler = Compiler {
        code: program,
        address,
        ..Compiler::default()
    };
    let compiled_program = compiler.into_compiled_program().expect("Failed to compile");
    let (verified_script, modules) =
        verify(&address, compiled_program.script, compiled_program.modules);
    execute(verified_script, args, modules)
}

pub fn execute(
    script: VerifiedScript,
    args: Vec<TransactionArgument>,
    modules: Vec<VerifiedModule>,
) -> VMResult<()> {
    // set up the DB
    let mut data_view = FakeDataStore::default();
    data_view.set(
        AccessPath::new(AccountAddress::random(), vec![]),
        vec![0, 0],
    );
    execute_function(script, modules, args, &data_view)
}

fn verify(
    sender_address: &AccountAddress,
    compiled_script: CompiledScript,
    modules: Vec<CompiledModule>,
) -> (VerifiedScript, Vec<VerifiedModule>) {
    let (verified_script, verified_modules) =
        static_verify_program(sender_address, compiled_script, modules)
            .expect("verification failure");
    (verified_script, verified_modules)
}
