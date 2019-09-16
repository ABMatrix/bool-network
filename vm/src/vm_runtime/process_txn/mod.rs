use crate::vm_runtime::{
    code_cache::module_cache::ModuleCache, data_cache::RemoteCache,
    loaded_data::loaded_module::LoadedModule,
	config::VMPublishingOption,
};
use std::marker::PhantomData;
use crate::types::{
	transaction::SignatureCheckedTransaction,
	vm_error::VMStatus,
};
use vm_cache_map::Arena;

pub mod execute;
pub mod validate;
pub mod verify;

use validate::{ValidatedTransaction, ValidationMode};

/// The starting point for processing a transaction. All the different states involved are described
/// through the types present in submodules.
pub struct ProcessTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignatureCheckedTransaction,
    module_cache: P,
    data_cache: &'txn dyn RemoteCache,
    allocator: &'txn Arena<LoadedModule>,
    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> ProcessTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Creates a new instance of `ProcessTransaction`.
    pub fn new(
        txn: SignatureCheckedTransaction,
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
        allocator: &'txn Arena<LoadedModule>,
    ) -> Self {
        Self {
            txn,
            module_cache,
            data_cache,
            allocator,
            phantom: PhantomData,
        }
    }

    /// Validates this transaction. Returns a `ValidatedTransaction` on success or `VMStatus` on
    /// failure.
    pub fn validate(
        self,
        mode: ValidationMode,
        publishing_option: &VMPublishingOption,
    ) -> Result<ValidatedTransaction<'alloc, 'txn, P>, VMStatus> {
        ValidatedTransaction::new(self, mode, publishing_option)
    }
}
