// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    ledger_info::LedgerInfo,
    proof::{
        get_accumulator_root_hash, verify_signed_transaction, verify_transaction_list,
        AccumulatorProof, SignedTransactionProof,
    },
    vm_error::VMStatus,
    write_set::WriteSet,
	Version,
	ByteArray,
};
use crypto::{
    hash::{
        CryptoHash, CryptoHasher, EventAccumulatorHasher, RawTransactionHasher,
        SignedTransactionHasher, TransactionInfoHasher,
    },
    signing, HashValue, PrivateKey, PublicKey, Signature,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use byteorder::{LittleEndian, WriteBytesExt};
use failure::prelude::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, time::Duration, collections::HashMap};
use std::ops::Deref;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    code: Vec<u8>,
    args: Vec<TransactionArgument>,
    modules: Vec<Vec<u8>>,
}

impl Program {
    pub fn new(code: Vec<u8>, modules: Vec<Vec<u8>>, args: Vec<TransactionArgument>) -> Program {
        Program {
            code,
            modules,
            args,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn modules(&self) -> &[Vec<u8>] {
        &self.modules
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TransactionArgument>, Vec<Vec<u8>>) {
        (self.code, self.args, self.modules)
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // XXX note that "code" will eventually be encoded bytecode and will no longer be a
        // UTF8-ish string -- at that point the from_utf8_lossy will stop making sense.
        f.debug_struct("Program")
            .field("code", &String::from_utf8_lossy(&self.code))
            .field("args", &self.args)
            .finish()
    }
}


#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionArgument {
    U64(u64),
    Address(AccountAddress),
    ByteArray(ByteArray),
    String(String),
}

impl fmt::Debug for TransactionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionArgument::U64(value) => write!(f, "{{U64: {}}}", value),
            TransactionArgument::Address(address) => write!(f, "{{ADDRESS: {:?}}}", address),
            TransactionArgument::String(string) => write!(f, "{{STRING: {}}}", string),
            TransactionArgument::ByteArray(byte_array) => {
                write!(f, "{{ByteArray: 0x{}}}", byte_array)
            }
        }
    }
}

pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;

/// RawTransaction is the portion of a transaction that a client signs
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawTransaction {
    /// Sender's address.
    sender: AccountAddress,
    // Sequence number of this transaction corresponding to sender's account.
    sequence_number: u64,
    // The transaction script to execute.
    payload: TransactionPayload,

    // Maximal total gas specified by wallet to spend for this transaction.
    max_gas_amount: u64,
    // Maximal price can be paid per gas.
    gas_unit_price: u64,
    // Expiration time for this transaction.  If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    expiration_time: Duration,
}

impl RawTransaction {
    /// Create a new `RawTransaction` with a program.
    ///
    /// Almost all transactions are program transactions. See `new_write_set` for write-set
    /// transactions.
    pub fn new(
        sender: AccountAddress,
        sequence_number: u64,
        program: Program,
        max_gas_amount: u64,
        gas_unit_price: u64,
        expiration_time: Duration,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::Program(program),
            max_gas_amount,
            gas_unit_price,
            expiration_time,
        }
    }

    pub fn new_write_set(
        sender: AccountAddress,
        sequence_number: u64,
        write_set: WriteSet,
    ) -> Self {
        RawTransaction {
            sender,
            sequence_number,
            payload: TransactionPayload::WriteSet(write_set),
            // Since write-set transactions bypass the VM, these fields aren't relevant.
            max_gas_amount: 0,
            gas_unit_price: 0,
            // Write-set transactions are special and important and shouldn't expire.
            expiration_time: Duration::new(u64::max_value(), 0),
        }
    }

    /// Signs the given `RawTransaction`. Note that this consumes the `RawTransaction` and turns it
    /// into a `SignatureCheckedTransaction`.
    ///
    /// For a transaction that has just been signed, its signature is expected to be valid.
    pub fn sign(
        self,
        private_key: &PrivateKey,
        public_key: PublicKey,
    ) -> Result<SignatureCheckedTransaction> {
        let raw_txn_bytes = bincode::serialize(&self).expect("serialization failed");
        let hash = RawTransactionBytes(&raw_txn_bytes).hash();
        let signature = signing::sign_message(hash, private_key)?;
        Ok(SignatureCheckedTransaction(SignedTransaction {
            raw_txn: self,
            public_key,
            signature,
            raw_txn_bytes,
        }))
    }

    pub fn into_payload(self) -> TransactionPayload {
        self.payload
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        let empty_vec = vec![];
        let (code, args) = match &self.payload {
            TransactionPayload::Program(program) => {
                (get_transaction_name(program.code()), program.args())
            }
            TransactionPayload::WriteSet(_) => ("genesis".to_string(), &empty_vec[..]),
        };
        let mut f_args: String = "".to_string();
        for arg in args {
            f_args = format!("{}\n\t\t\t{:#?},", f_args, arg);
        }
        format!(
            "RawTransaction {{ \n\
             \tsender: {}, \n\
             \tsequence_number: {}, \n\
             \tpayload: {{, \n\
             \t\ttransaction: {}, \n\
             \t\targs: [ {} \n\
             \t\t]\n\
             \t}}, \n\
             \tmax_gas_amount: {}, \n\
             \tgas_unit_price: {}, \n\
             \texpiration_time: {:#?}, \n\
             }}",
            self.sender,
            self.sequence_number,
            code,
            f_args,
            self.max_gas_amount,
            self.gas_unit_price,
            self.expiration_time,
        )
    }
    /// Return the sender of this transaction.
    pub fn sender(&self) -> AccountAddress {
        self.sender
    }
}

pub struct RawTransactionBytes<'a>(pub &'a [u8]);

impl<'a> CryptoHash for RawTransactionBytes<'a> {
    type Hasher = RawTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(self.0);
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionPayload {
    /// A regular programmatic transaction that is executed by the VM.
    Program(Program),
    WriteSet(WriteSet),
}

/// A transaction that has been signed.
///
/// A `SignedTransaction` is a single transaction that can be atomically executed. Clients submit
/// these to validator nodes, and the validator and executor submits these to the VM.
///
/// **IMPORTANT:** The signature of a `SignedTransaction` is not guaranteed to be verified. For a
/// transaction whose signature is statically guaranteed to be verified, see
/// [`SignatureCheckedTransaction`].
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The raw transaction
    raw_txn: RawTransaction,

    /// Sender's public key. When checking the signature, we first need to check whether this key
    /// is indeed the pre-image of the pubkey hash stored under sender's account.
    public_key: PublicKey,

    /// Signature of the transaction that correspond to the public key
    signature: Signature,

    // The original raw bytes from the protobuf are also stored here so that we use
    // these bytes when generating the canonical serialization of the SignedTransaction struct
    // rather than re-serializing them again to avoid risk of non-determinism in the process

    // the raw transaction bytes generated from the wallet
    raw_txn_bytes: Vec<u8>,
}

/// A transaction for which the signature has been verified. Created by
/// [`SignedTransaction::check_signature`] and [`RawTransaction::sign`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SignatureCheckedTransaction(SignedTransaction);

impl SignatureCheckedTransaction {
    /// Returns a reference to the `SignedTransaction` within.
    pub fn as_inner(&self) -> &SignedTransaction {
        &self.0
    }

    /// Returns the `SignedTransaction` within.
    pub fn into_inner(self) -> SignedTransaction {
        self.0
    }

    /// Returns the `RawTransaction` within.
    pub fn into_raw_transaction(self) -> RawTransaction {
        self.0.into_raw_transaction()
    }
}

impl Deref for SignatureCheckedTransaction {
    type Target = SignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for SignedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SignedTransaction {{ \n \
             {{ raw_txn: {:#?}, \n \
             public_key: {:#?}, \n \
             signature: {:#?}, \n \
             }} \n \
             }}",
            self.raw_txn, self.public_key, self.signature,
        )
    }
}

impl SignedTransaction {
    pub fn craft_signed_transaction_for_client(
        raw_txn: RawTransaction,
        public_key: PublicKey,
        signature: Signature,
    ) -> SignedTransaction {
        SignedTransaction {
            raw_txn: raw_txn.clone(),
            public_key,
            signature,
            // In real world raw_txn should be derived from raw_txn_bytes, not the opposite.
            raw_txn_bytes: bincode::serialize(&raw_txn).expect("Should convert."),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn signature(&self) -> Signature {
        self.signature
    }

    pub fn sender(&self) -> AccountAddress {
        self.raw_txn.sender
    }

    pub fn into_raw_transaction(self) -> RawTransaction {
        self.raw_txn
    }

    pub fn sequence_number(&self) -> u64 {
        self.raw_txn.sequence_number
    }

    pub fn payload(&self) -> &TransactionPayload {
        &self.raw_txn.payload
    }

    pub fn max_gas_amount(&self) -> u64 {
        self.raw_txn.max_gas_amount
    }

    pub fn gas_unit_price(&self) -> u64 {
        self.raw_txn.gas_unit_price
    }

    pub fn expiration_time(&self) -> Duration {
        self.raw_txn.expiration_time
    }

    pub fn raw_txn_bytes_len(&self) -> usize {
        self.raw_txn_bytes.len()
    }

    /// Checks that the signature of given transaction. Returns `Ok(SignatureCheckedTransaction)` if
    /// the signature is valid.
    pub fn check_signature(self) -> Result<SignatureCheckedTransaction> {
        let hash = RawTransactionBytes(&self.raw_txn_bytes).hash();
        signing::verify_signature(hash, &self.signature, &self.public_key)?;
        Ok(SignatureCheckedTransaction(self))
    }

    pub fn format_for_client(&self, get_transaction_name: impl Fn(&[u8]) -> String) -> String {
        format!(
            "SignedTransaction {{ \n \
             raw_txn: {}, \n \
             public_key: {:#?}, \n \
             signature: {:#?}, \n \
             }}",
            self.raw_txn.format_for_client(get_transaction_name),
            self.public_key,
            self.signature,
        )
    }

	pub fn into_bytes(&self) -> Result<Vec<u8>> {
		return Ok(vec![1, 2])
	}
}

impl CryptoHash for SignedTransaction {
    type Hasher = SignedTransactionHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).expect("serialization failed"));
        state.finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct SignedTransactionWithProof {
    pub version: Version,
    pub signed_transaction: SignedTransaction,
    pub events: Option<Vec<ContractEvent>>,
    pub proof: SignedTransactionProof,
}

impl SignedTransactionWithProof {
    /// Verifies the signed transaction with the proof, both carried by `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. This signed transaction exists in the ledger represented by `ledger_info`.
    ///   2. And this signed transaction has the same `version`, `sender`, and `sequence_number` as
    /// indicated by the parameter list. If any of these parameter is unknown to the call site that
    /// is supposed to be informed via this struct, get it from the struct itself, such as:
    /// `signed_txn_with_proof.version`, `signed_txn_with_proof.signed_transaction.sender()`, etc.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        version: Version,
        sender: AccountAddress,
        sequence_number: u64,
    ) -> Result<()> {
        ensure!(
            self.version == version,
            "Version ({}) is not expected ({}).",
            self.version,
            version,
        );
        ensure!(
            self.signed_transaction.sender() == sender,
            "Sender ({}) not expected ({}).",
            self.signed_transaction.sender(),
            sender,
        );
        ensure!(
            self.signed_transaction.sequence_number() == sequence_number,
            "Sequence number ({}) not expected ({}).",
            self.signed_transaction.sequence_number(),
            sequence_number,
        );

        let events_root_hash = self.events.as_ref().map(|events| {
            let event_hashes: Vec<_> = events.iter().map(ContractEvent::hash).collect();
            get_accumulator_root_hash::<EventAccumulatorHasher>(&event_hashes)
        });
        verify_signed_transaction(
            ledger_info,
            self.signed_transaction.hash(),
            events_root_hash,
            version,
            &self.proof,
        )
    }
}

impl CanonicalSerialize for SignedTransaction {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_variable_length_bytes(&self.raw_txn_bytes)?
            .encode_variable_length_bytes(&self.public_key.to_slice())?
            .encode_variable_length_bytes(&self.signature.to_compact())?;
        Ok(())
    }
}

impl CanonicalDeserialize for SignedTransaction {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let raw_txn_bytes = deserializer.decode_variable_length_bytes()?;
        let public_key_bytes = deserializer.decode_variable_length_bytes()?;
        let signature_bytes = deserializer.decode_variable_length_bytes()?;
		let raw_txn: RawTransaction = bincode::deserialize(&raw_txn_bytes[..])?;
        Ok(SignedTransaction {
            raw_txn: raw_txn,
            public_key: PublicKey::from_slice(&public_key_bytes)?,
            signature: Signature::from_compact(&signature_bytes)?,
            raw_txn_bytes,
        })
    }
}

/// The status of executing a transaction. The VM decides whether or not we should `Keep` the
/// transaction output or `Discard` it based upon the execution of the transaction. We wrap these
/// decisions around a `VMStatus` that provides more detail on the final execution state of the VM.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    /// Discard the transaction output
    Discard(VMStatus),

    /// Keep the transaction output
    Keep(VMStatus),
}

impl From<VMStatus> for TransactionStatus {
    fn from(vm_status: VMStatus) -> Self {
        let should_discard = match vm_status {
            // Any error that is a validation status (i.e. an error arising from the prologue)
            // causes the transaction to not be included.
            VMStatus::Validation(_) => true,
            // If the VM encountered an invalid internal state, we should discard the transaction.
            VMStatus::InvariantViolation(_) => true,
            // A transaction that publishes code that cannot be verified is currently not charged.
            // Therefore the transaction can be excluded.
            //
            // The original plan was to charge for verification, but the code didn't implement it
            // properly. The decision of whether to charge or not will be made based on data (if
            // verification checks are too expensive then yes, otherwise no).
            VMStatus::Verification(_) => true,
            // Even if we are unable to decode the transaction, there should be a charge made to
            // that user's account for the gas fees related to decoding, running the prologue etc.
            VMStatus::Deserialization(_) => false,
            // Any error encountered during the execution of the transaction will charge gas.
            VMStatus::Execution(_) => false,
        };

        if should_discard {
            TransactionStatus::Discard(vm_status)
        } else {
            TransactionStatus::Keep(vm_status)
        }
    }
}

/// The output of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutput {
    /// The list of writes this transaction intends to do.
    write_set: WriteSet,

    /// The list of events emitted during this transaction.
    events: Vec<ContractEvent>,

    /// The amount of gas used during execution.
    gas_used: u64,

    /// The execution status.
    status: TransactionStatus,
}

impl TransactionOutput {
    pub fn new(
        write_set: WriteSet,
        events: Vec<ContractEvent>,
        gas_used: u64,
        status: TransactionStatus,
    ) -> Self {
        TransactionOutput {
            write_set,
            events,
            gas_used,
            status,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct TransactionInfo {
    /// The hash of this transaction.
    signed_transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    event_root_hash: HashValue,

    /// The amount of gas used.
    gas_used: u64,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using signed transaction hash, state root hash
    /// and event root hash.
    pub fn new(
        signed_transaction_hash: HashValue,
        state_root_hash: HashValue,
        event_root_hash: HashValue,
        gas_used: u64,
    ) -> TransactionInfo {
        TransactionInfo {
            signed_transaction_hash,
            state_root_hash,
            event_root_hash,
            gas_used,
        }
    }

    /// Returns the hash of this transaction.
    pub fn signed_transaction_hash(&self) -> HashValue {
        self.signed_transaction_hash
    }

    /// Returns root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub fn state_root_hash(&self) -> HashValue {
        self.state_root_hash
    }

    /// Returns the root hash of Merkle Accumulator storing all events emitted during this
    /// transaction.
    pub fn event_root_hash(&self) -> HashValue {
        self.event_root_hash
    }

    /// Returns the amount of gas used by this transaction.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}

impl CanonicalSerialize for TransactionInfo {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_raw_bytes(self.signed_transaction_hash.as_ref())?
            .encode_raw_bytes(self.state_root_hash.as_ref())?
            .encode_raw_bytes(self.event_root_hash.as_ref())?
            .encode_u64(self.gas_used)?;
        Ok(())
    }
}

impl CryptoHash for TransactionInfo {
    type Hasher = TransactionInfoHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            &SimpleSerializer::<Vec<u8>>::serialize(self).expect("Serialization should work."),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionToCommit {
    signed_txn: SignedTransaction,
    account_states: HashMap<AccountAddress, AccountStateBlob>,
    events: Vec<ContractEvent>,
    gas_used: u64,
}

impl TransactionToCommit {
    pub fn new(
        signed_txn: SignedTransaction,
        account_states: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>,
        gas_used: u64,
    ) -> Self {
        TransactionToCommit {
            signed_txn,
            account_states,
            events,
            gas_used,
        }
    }

    pub fn signed_txn(&self) -> &SignedTransaction {
        &self.signed_txn
    }

    pub fn account_states(&self) -> &HashMap<AccountAddress, AccountStateBlob> {
        &self.account_states
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }
}


/// The list may have three states:
/// 1. The list is empty. Both proofs must be `None`.
/// 2. The list has only 1 transaction/transaction_info. Then `proof_of_first_transaction`
/// must exist and `proof_of_last_transaction` must be `None`.
/// 3. The list has 2+ transactions/transaction_infos. The both proofs must exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionListWithProof {
    pub transaction_and_infos: Vec<(SignedTransaction, TransactionInfo)>,
    pub events: Option<Vec<Vec<ContractEvent>>>,
    pub first_transaction_version: Option<Version>,
    pub proof_of_first_transaction: Option<AccumulatorProof>,
    pub proof_of_last_transaction: Option<AccumulatorProof>,
}

impl TransactionListWithProof {
    /// Constructor.
    pub fn new(
        transaction_and_infos: Vec<(SignedTransaction, TransactionInfo)>,
        events: Option<Vec<Vec<ContractEvent>>>,
        first_transaction_version: Option<Version>,
        proof_of_first_transaction: Option<AccumulatorProof>,
        proof_of_last_transaction: Option<AccumulatorProof>,
    ) -> Self {
        Self {
            transaction_and_infos,
            events,
            first_transaction_version,
            proof_of_first_transaction,
            proof_of_last_transaction,
        }
    }

    /// Creates an empty transaction list.
    pub fn new_empty() -> Self {
        Self::new(Vec::new(), None, None, None, None)
    }

    /// Verifies the transaction list with the proofs, both carried on `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. All the transactions exist on the ledger represented by `ledger_info`.
    ///   2. And the transactions in the list has consecutive versions starting from
    /// `first_transaction_version`. When `first_transaction_version` is None, ensures the list is
    /// empty.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        first_transaction_version: Option<Version>,
    ) -> Result<()> {
        ensure!(
            self.first_transaction_version == first_transaction_version,
            "First transaction version ({}) not expected ({}).",
            Self::display_option_version(self.first_transaction_version),
            Self::display_option_version(first_transaction_version),
        );

        verify_transaction_list(ledger_info, self)
    }

    fn display_option_version(version: Option<Version>) -> String {
        match version {
            Some(v) => format!("{}", v),
            None => String::from("absent"),
        }
    }
}

