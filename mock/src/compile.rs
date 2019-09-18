// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.

use compiler::{Compiler};
use vm::{
    types::{
        AccountAddress,
        transaction::{Program, TransactionArgument},
    },
    bytecode_verifier::VerifiedModule,
    def::file_format::CompiledProgram,
};

// pub fn compile_program(body: &ast::Program) -> Vec<u8> {
//     let compiled_program =
//         compile_program(&AccountAddress::default(), body, stdlib_modules()).unwrap();
//     let mut script_bytes = vec![];
//     compiled_program
//         .script
//         .serialize(&mut script_bytes)
//         .unwrap();
//     script_bytes
// }

/// Compile the provided Move code into a blob which can be used as the code for a [`Program`].
///
/// The script is compiled with the default account address (`0x0`).
pub fn compile_script(code: &str) -> Vec<u8> {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_script_blob().unwrap()
}


/// Compile the provided Move code and arguments into a `Program`.
///
/// This supports both scripts and modules defined in the same Move code. The code is compiled with
/// the default account address (`0x0`).
pub fn compile_program(code: &str, args: Vec<TransactionArgument>) -> Program {
    let compiler = Compiler {
        code,
        ..Compiler::default()
    };
    compiler.into_program(args).unwrap()
}


/// Compile the provided Move code and arguments into a `Program` using `address` as the
/// self address for any modules in `code`.
pub fn compile_program_with_address(
    address: &AccountAddress,
    code: &str,
    args: Vec<TransactionArgument>,
) -> Program {
    let compiler = Compiler {
        address: *address,
        code,
        ..Compiler::default()
    };
    compiler.into_program(args).unwrap()
}

pub fn compile_inner_program_with_address(
    address: &AccountAddress,
    code: &str,
) -> CompiledProgram {
    let compiler = Compiler {
        address: *address,
        code,
        ..Compiler::default()
    };
    compiler.into_compiled_program().unwrap()
}

/// Compile the provided Move code and arguments into a `Program` using `address` as the
/// self address for any modules in `code`.
/// extranal dependences.
pub fn compile_program_with_deps(
    address: &AccountAddress,
    code: &str,
    args: Vec<TransactionArgument>,
    extra_deps: Vec<VerifiedModule>
) -> Program {
    let compiler = Compiler {
        address: *address,
        code,
        extra_deps,
        ..Compiler::default()
    };
    compiler.into_program(args).unwrap()
}
