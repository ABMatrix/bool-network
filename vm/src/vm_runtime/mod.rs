pub mod config; //-

pub mod identifier;
pub mod op_metrics;		//-
pub mod loaded_data;	//-
pub mod counters;	//-
pub mod vm_runtime_types; //-
pub mod code_cache;	//--
pub mod data_cache;	//--
pub mod process_txn; //--
pub mod gas_meter; 	//--
pub mod frame;				//--
pub mod execution_stack;	//--

pub mod txn_executor;
pub mod block_processor;	//--
pub mod runtime;			//---
pub mod move_vm;

pub use move_vm::MoveVM;
pub use process_txn::verify::static_verify_program;
pub use txn_executor::execute_function;

use crate::vm_runtime::config::VMConfig;
use crate::def::{errors::VMInvariantViolation, IndexKind};
use crate::state_view::StateView;
use crate::types::{
    transaction::{SignedTransaction, TransactionOutput},
    vm_error::VMStatus,
};

pub(crate) fn bounded_fetch<T>(
    pool: &[T],
    idx: usize,
    bound_type: IndexKind,
) -> Result<&T, VMInvariantViolation> {
    pool.get(idx)
        .ok_or_else(|| VMInvariantViolation::IndexOutOfBounds(bound_type, pool.len(), idx))
}

/// This trait describes the VM's verification interfaces.
pub trait VMVerifier {
    /// Executes the prologue of the Libra Account and verifies that the transaction is valid.
    /// only. Returns `None` if the transaction was validated, or Some(VMStatus) if the transaction
    /// was unable to be validated with status `VMStatus`.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus>;
}

/// This trait describes the VM's execution interface.
pub trait VMExecutor {
    // NOTE: At the moment there are no persistent caches that live past the end of a block (that's
    // why execute_block doesn't take &self.)
    // There are some cache invalidation issues around transactions publishing code that need to be
    // sorted out before that's possible.

    /// Executes a block of transactions and returns output for each one of them.
    fn execute_block(
        transactions: Vec<SignedTransaction>,
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Vec<TransactionOutput>;
}