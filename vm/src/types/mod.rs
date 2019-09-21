use crate::types::transaction::TransactionArgument;
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
    SimpleSerializer,
};
use crypto::{
    hash::{AccessPathHasher, AccountAddressHasher, CryptoHash, CryptoHasher, HashValue},
    PublicKey as LegacyPublicKey,
};
use failure::prelude::*;
use hex;
use nextgen_crypto::{ed25519::*, VerifyingKey};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::path::Display;
use std::{
    convert::TryFrom,
    fmt::{self, Formatter},
    slice::Iter,
    str::{self, FromStr},
};
use tiny_keccak::Keccak;
use vm_error::VMStatus;

pub mod account_config; //-
pub mod account_state_blob; //--
pub mod contract_event; //--
pub mod ledger_info; //--
pub mod proof; //--
pub mod transaction;
pub mod validator_verifier; //-
pub mod vm_error; //-
pub mod write_set; //--

//
//

pub type Version = u64; // Height - also used for MVCC in StateDB
pub const MAX_TRANSACTION_SIZE_IN_BYTES: usize = 4096;
pub const ADDRESS_LENGTH: usize = 32;
pub const SCRIPT_HASH_LENGTH: usize = 32;

const SHORT_STRING_LENGTH: usize = 4;
const LIBRA_NETWORK_ID_SHORT: &str = "lb";
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone, Copy, Deserialize, Serialize)]
pub struct AccountAddress([u8; ADDRESS_LENGTH]);

impl AccountAddress {
    pub fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        AccountAddress(address)
    }

    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let buf: [u8; 32] = rng.gen();
        AccountAddress::new(buf)
    }

    // Helpful in log messages
    pub fn short_str(&self) -> String {
        hex::encode(&self.0[0..SHORT_STRING_LENGTH]).to_string()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_public_key<PublicKey: VerifyingKey>(public_key: &PublicKey) -> Self {
        // TODO: using keccak directly instead of crypto::hash because we have to make sure we use
        // the same hash function that the Move transaction prologue is using.
        // TODO: keccak is just a placeholder, make a principled choice for the hash function
        let mut keccak = Keccak::new_sha3_256();
        let mut hash = [0u8; ADDRESS_LENGTH];
        keccak.update(&public_key.to_bytes());
        keccak.finalize(&mut hash);
        AccountAddress::new(hash)
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

impl fmt::Debug for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

impl fmt::LowerHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl AsRef<[u8]> for AccountAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<String> for AccountAddress {
    type Error = failure::Error;

    fn try_from(s: String) -> Result<AccountAddress> {
        assert!(!s.is_empty());
        let bytes_out = ::hex::decode(s)?;
        AccountAddress::try_from(bytes_out.as_slice())
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = failure::Error;
    /// Tries to convert the provided byte array into Address.
    fn try_from(bytes: &[u8]) -> Result<AccountAddress> {
        ensure!(
            bytes.len() == ADDRESS_LENGTH,
            "The Address {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AccountAddress(addr))
    }
}

impl TryFrom<Vec<u8>> for AccountAddress {
    type Error = failure::Error;

    /// Tries to convert the provided byte buffer into Address.
    fn try_from(bytes: Vec<u8>) -> Result<AccountAddress> {
        AccountAddress::try_from(&bytes[..])
    }
}

impl From<AccountAddress> for Vec<u8> {
    fn from(addr: AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<&AccountAddress> for Vec<u8> {
    fn from(addr: &AccountAddress) -> Vec<u8> {
        addr.0.to_vec()
    }
}

impl From<LegacyPublicKey> for AccountAddress {
    fn from(public_key: LegacyPublicKey) -> AccountAddress {
        let ed25519_public_key: Ed25519PublicKey = public_key.into();
        AccountAddress::from_public_key(&ed25519_public_key)
    }
}

impl CryptoHash for AccountAddress {
    type Hasher = AccountAddressHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&self.0);
        state.finish()
    }
}

impl CanonicalSerialize for AccountAddress {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_variable_length_bytes(&self.0)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccountAddress {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let bytes = deserializer.decode_variable_length_bytes()?;
        Self::try_from(bytes)
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ModuleId {
    address: AccountAddress,
    name: String,
}

impl<'a> From<&'a ModuleId> for AccessPath {
    fn from(module_id: &'a ModuleId) -> Self {
        AccessPath::code_access_path(module_id)
    }
}

impl ModuleId {
    pub fn new(address: AccountAddress, name: String) -> Self {
        ModuleId { address, name }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.address(), self.name())
    }
}

impl CanonicalSerialize for ModuleId {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_variable_length_bytes(self.name.as_bytes())?;
        Ok(())
    }
}

impl CanonicalDeserialize for ModuleId {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let name = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;

        Ok(Self { address, name })
    }
}

impl CryptoHash for ModuleId {
    type Hasher = AccessPathHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).unwrap());
        state.finish()
    }
}

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

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct ByteArray(Vec<u8>);

impl ByteArray {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn new(buf: Vec<u8>) -> Self {
        ByteArray(buf)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl fmt::Display for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "b\"{}\"", hex::encode(&self.0))
    }
}

impl CanonicalSerialize for ByteArray {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_variable_length_bytes(&self.0)?;
        Ok(())
    }
}

impl CanonicalDeserialize for ByteArray {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let bytes = deserializer.decode_variable_length_bytes()?;
        Ok(ByteArray(bytes))
    }
}

#[derive(Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: Vec<u8>,
}

impl AccessPath {
    const CODE_TAG: u8 = 0;
    const RESOURCE_TAG: u8 = 1;

    pub fn new(address: AccountAddress, path: Vec<u8>) -> Self {
        AccessPath { address, path }
    }

    /// Given an address, returns the corresponding access path that stores the Account resource.
    pub fn new_for_account(address: AccountAddress) -> Self {
        Self::new(address, account_resource_path())
    }

    /// Create an AccessPath for a ContractEvent.
    /// That is an AccessPah that uniquely identifies a given event for a published resource.
    pub fn new_for_event(address: AccountAddress, root: &[u8], key: &[u8]) -> Self {
        let mut path: Vec<u8> = Vec::new();
        path.extend_from_slice(root);
        path.push(b'/');
        path.extend_from_slice(key);
        path.push(b'/');
        Self::new(address, path)
    }

    /// Create an AccessPath to the event for the sender account in a deposit operation.
    /// The sent counter in LibraAccount.T (LibraAccount.T.sent_events_count) is used to generate
    /// the AccessPath.
    /// That AccessPath can be used as a key into the event storage to retrieve all sent
    /// events for a given account.
    pub fn new_for_sent_event(address: AccountAddress) -> Self {
        Self::new(address, account_sent_event_path())
    }

    /// Create an AccessPath to the event for the target account (the receiver)
    /// in a deposit operation.
    /// The received counter in LibraAccount.T (LibraAccount.T.received_events_count) is used to
    /// generate the AccessPath.
    /// That AccessPath can be used as a key into the event storage to retrieve all received
    /// events for a given account.
    pub fn new_for_received_event(address: AccountAddress) -> Self {
        Self::new(address, account_received_event_path())
    }

    pub fn resource_access_vec(tag: &StructTag, accesses: &Accesses) -> Vec<u8> {
        let mut key = vec![];
        key.push(Self::RESOURCE_TAG);

        key.append(&mut tag.hash().to_vec());

        // We don't need accesses in production right now. Accesses are appended here just for
        // passing the old tests.
        key.append(&mut accesses.as_separated_string().into_bytes());
        key
    }

    /// Convert Accesses into a byte offset which would be used by the storage layer to resolve
    /// where fields are stored.
    pub fn resource_access_path(key: &ResourceKey, accesses: &Accesses) -> AccessPath {
        let path = AccessPath::resource_access_vec(&key.type_(), accesses);
        AccessPath {
            address: key.address().to_owned(),
            path,
        }
    }

    fn code_access_path_vec(key: &ModuleId) -> Vec<u8> {
        let mut root = vec![];
        root.push(Self::CODE_TAG);
        root.append(&mut key.hash().to_vec());
        root
    }

    pub fn code_access_path(key: &ModuleId) -> AccessPath {
        let path = AccessPath::code_access_path_vec(key);
        AccessPath {
            address: *key.address(),
            path,
        }
    }

    pub fn is_resource_path(&self) -> bool {
        return Self::new_for_account(self.address) == *self;
    }
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccessPath {{ address: {:x}, path: {} }}",
            self.address,
            hex::encode(&self.path)
        )
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.path.len() < 1 + HashValue::LENGTH {
            write!(f, "{:?}", self)
        } else {
            write!(f, "AccessPath {{ address: {:x}, ", self.address)?;
            match self.path[0] {
                Self::RESOURCE_TAG => write!(f, "type: Resource, ")?,
                Self::CODE_TAG => write!(f, "type: Module, ")?,
                tag => write!(f, "type: {:?}, ", tag)?,
            };
            write!(
                f,
                "hash: {:?}, ",
                hex::encode(&self.path[1..=HashValue::LENGTH])
            )?;
            write!(
                f,
                "suffix: {:?} }} ",
                String::from_utf8_lossy(&self.path[1 + HashValue::LENGTH..])
            )
        }
    }
}

impl CanonicalSerialize for AccessPath {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_variable_length_bytes(&self.path)?;
        Ok(())
    }
}

impl CanonicalDeserialize for AccessPath {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let path = deserializer.decode_variable_length_bytes()?;

        Ok(Self { address, path })
    }
}

#[derive(Default, Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Field(String);

impl Field {
    pub fn new(s: &str) -> Field {
        Field(s.to_string())
    }

    pub fn name(&self) -> &String {
        &self.0
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Field(s)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, Hash, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub enum Access {
    Field(Field),
    Index(u64),
}

impl Access {
    pub fn new(s: &str) -> Self {
        Access::Field(Field::new(s))
    }
}

impl FromStr for Access {
    type Err = ::std::num::ParseIntError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        if let Ok(idx) = s.parse::<u64>() {
            Ok(Access::Index(idx))
        } else {
            Ok(Access::Field(Field::new(s)))
        }
    }
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Access::Field(field) => write!(f, "\"{}\"", field),
            Access::Index(i) => write!(f, "{}", i),
        }
    }
}

/// Non-empty sequence of field accesses
#[derive(Eq, Hash, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub struct Accesses(Vec<Access>);

/// SEPARATOR is used as a delimiter between fields. It should not be a legal part of any identifier
/// in the language
const SEPARATOR: char = '/';

impl Accesses {
    pub fn empty() -> Self {
        Accesses(vec![])
    }

    pub fn new(field: Field) -> Self {
        Accesses(vec![Access::Field(field)])
    }

    /// Add a field to the end of the sequence
    pub fn add_field_to_back(&mut self, field: Field) {
        self.0.push(Access::Field(field))
    }

    /// Add an index to the end of the sequence
    pub fn add_index_to_back(&mut self, idx: u64) {
        self.0.push(Access::Index(idx))
    }

    pub fn append(&mut self, accesses: &mut Accesses) {
        self.0.append(&mut accesses.0)
    }

    /// Returns the first field in the sequence and reference to the remaining fields
    pub fn split_first(&self) -> (&Access, &[Access]) {
        self.0.split_first().unwrap()
    }

    /// Return the last access in the sequence
    pub fn last(&self) -> &Access {
        self.0.last().unwrap() // guaranteed not to fail because sequence is non-empty
    }

    pub fn iter(&self) -> Iter<'_, Access> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_separated_string(&self) -> String {
        let mut path = String::new();
        for access in self.0.iter() {
            match access {
                Access::Field(s) => {
                    let access_str = s.name().as_ref();
                    assert!(access_str != "");
                    path.push_str(access_str)
                }
                Access::Index(i) => path.push_str(i.to_string().as_ref()),
            };
            path.push(SEPARATOR);
        }
        path
    }

    pub fn take_nth(&self, new_len: usize) -> Accesses {
        assert!(self.0.len() >= new_len);
        Accesses(self.0.clone().into_iter().take(new_len).collect())
    }
}

impl<'a> IntoIterator for &'a Accesses {
    type Item = &'a Access;
    type IntoIter = Iter<'a, Access>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<Access>> for Accesses {
    fn from(accesses: Vec<Access>) -> Accesses {
        Accesses(accesses)
    }
}

impl From<Vec<u8>> for Accesses {
    fn from(mut raw_bytes: Vec<u8>) -> Accesses {
        let access_str = String::from_utf8(raw_bytes.split_off(HashValue::LENGTH + 1)).unwrap();
        let fields_str = access_str.split(SEPARATOR).collect::<Vec<&str>>();
        let mut accesses = vec![];
        for access_str in fields_str.into_iter() {
            if access_str != "" {
                accesses.push(Access::from_str(access_str).unwrap());
            }
        }
        Accesses::from(accesses)
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: String,
    pub name: String,
    pub type_params: Vec<StructTag>,
}

impl CanonicalSerialize for StructTag {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.address)?
            .encode_variable_length_bytes(self.module.as_bytes())?
            .encode_variable_length_bytes(self.name.as_bytes())?
            .encode_vec(&self.type_params)?;
        Ok(())
    }
}

impl CanonicalDeserialize for StructTag {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let address = deserializer.decode_struct::<AccountAddress>()?;
        let module = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;
        let name = String::from_utf8(deserializer.decode_variable_length_bytes()?)?;
        let type_params = deserializer.decode_vec::<StructTag>()?;
        Ok(Self {
            address,
            name,
            module,
            type_params,
        })
    }
}

impl CryptoHash for StructTag {
    type Hasher = AccessPathHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&SimpleSerializer::<Vec<u8>>::serialize(self).unwrap());
        state.finish()
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct ResourceKey {
    address: AccountAddress,
    type_: StructTag,
}

impl ResourceKey {
    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn type_(&self) -> &StructTag {
        &self.type_
    }
}

impl ResourceKey {
    pub fn new(address: AccountAddress, type_: StructTag) -> Self {
        ResourceKey { address, type_ }
    }
}

// LibraCoin
pub const COIN_MODULE_NAME: &str = "LibraCoin";
pub const COIN_STRUCT_NAME: &str = "T";

// Account
pub const ACCOUNT_MODULE_NAME: &str = "LibraAccount";
pub const ACCOUNT_STRUCT_NAME: &str = "T";

// Hash
pub const HASH_MODULE_NAME: &str = "Hash";

/// Generic struct that represents an Account event.
/// Both SentPaymentEvent and ReceivedPaymentEvent are representable with this struct.
/// They have an AccountAddress for the sender or receiver and the amount transferred.
#[derive(Debug, Default)]
pub struct AccountEvent {
    account: AccountAddress,
    amount: u64,
}

impl AccountEvent {
    /// Get the account related to the event
    pub fn account(&self) -> AccountAddress {
        self.account
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

impl CanonicalDeserialize for AccountEvent {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        // TODO: this is a horrible hack and we need to come up with a proper separation of
        // data/code so that we don't need the entire VM to read an Account event.
        // Also we cannot depend on the VM here as we would have a circular dependency and
        // it's not clear if this API should live in the VM or in types
        let amount = deserializer.decode_u64()?;
        let account = deserializer.decode_struct()?;

        Ok(AccountEvent { account, amount })
    }
}

/// Return the path to the Account resource. It can be used to create an AccessPath for an
/// Account resource.
pub fn account_resource_path() -> Vec<u8> {
    AccessPath::resource_access_vec(
        &StructTag {
            address: core_code_address(),
            module: ACCOUNT_MODULE_NAME.to_string(),
            name: ACCOUNT_STRUCT_NAME.to_string(),
            type_params: vec![],
        },
        &Accesses::empty(),
    )
}

/// Return the path to the sent event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub fn account_sent_event_path() -> Vec<u8> {
    let mut path = account_resource_path();
    path.push(b'/');
    path.extend_from_slice(b"sent_events_count");
    path.push(b'/');
    path
}

/// Return the path to the received event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub fn account_received_event_path() -> Vec<u8> {
    let mut path = account_resource_path();
    path.push(b'/');
    path.extend_from_slice(b"received_events_count");
    path.push(b'/');
    path
}

pub fn core_code_address() -> AccountAddress {
    AccountAddress::default()
}

pub fn association_address() -> AccountAddress {
    AccountAddress::default()
}

#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "ParseError: {}", _0)]
    ParseError(String),
}

/// Parses the given string as address.
pub fn parse_as_address(s: &str) -> Result<TransactionArgument> {
    let mut s = s.to_ascii_lowercase();
    if !s.starts_with("0x") {
        return Err(ErrorKind::ParseError("address must start with '0x'".to_string()).into());
    }
    if s.len() == 2 {
        return Err(ErrorKind::ParseError("address cannot be empty".to_string()).into());
    }
    if s.len() % 2 != 0 {
        s = format!("0x0{}", &s[2..]);
    }
    let mut addr = hex::decode(&s[2..])?;
    if addr.len() > 32 {
        return Err(ErrorKind::ParseError("address must be 32 bytes or less".to_string()).into());
    }
    if addr.len() < 32 {
        addr = vec![0u8; 32 - addr.len()]
            .into_iter()
            .chain(addr.into_iter())
            .collect();
    }
    Ok(TransactionArgument::Address(AccountAddress::try_from(
        addr,
    )?))
}

/// Parses the given string as bytearray.
pub fn parse_as_byte_array(s: &str) -> Result<TransactionArgument> {
    if s.starts_with("b\"") && s.ends_with('"') && s.len() >= 3 {
        let s = &s[2..s.len() - 1];
        if s.is_empty() {
            return Err(ErrorKind::ParseError("byte array cannot be empty".to_string()).into());
        }
        let s = if s.len() % 2 == 0 {
            s.to_string()
        } else {
            format!("0{}", s)
        };
        Ok(TransactionArgument::ByteArray(ByteArray::new(hex::decode(
            &s,
        )?)))
    } else {
        Err(ErrorKind::ParseError(format!("\"{}\" is not a byte array", s)).into())
    }
}

/// Parses the given string as u64.
pub fn parse_as_u64(s: &str) -> Result<TransactionArgument> {
    Ok(TransactionArgument::U64(s.parse::<u64>()?))
}

macro_rules! return_if_ok {
    ($e: expr) => {{
        if let Ok(res) = $e {
            return Ok(res);
        }
    }};
}

/// Parses the given string as any transaction argument type.
pub fn parse_as_transaction_argument(s: &str) -> Result<TransactionArgument> {
    return_if_ok!(parse_as_address(s));
    return_if_ok!(parse_as_u64(s));
    return_if_ok!(parse_as_byte_array(s));
    Err(ErrorKind::ParseError(format!("cannot parse \"{}\" as transaction argument", s)).into())
}

#[cfg(test)]
mod test_transaction_argument {
    use super::*;

    #[test]
    fn parse_u64() {
        for s in &["0", "42", "18446744073709551615"] {
            parse_as_u64(s).unwrap();
        }
        for s in &["xx", "", "-3"] {
            parse_as_u64(s).unwrap_err();
        }
    }

    #[test]
    fn parse_address() {
        for s in &[
            "0x0",
            "0x1",
            "0x00",
            "0x05",
            "0x100",
            "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ] {
            parse_as_address(s).unwrap();
        }

        for s in &[
            "0x",
            "100",
            "",
            "0xG",
            "0xBBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ] {
            parse_as_address(s).unwrap_err();
        }
    }

    #[test]
    fn parse_byte_array() {
        for s in &["0", "00", "deadbeef", "aaa"] {
            parse_as_byte_array(&format!("b\"{}\"", s)).unwrap();
        }

        for s in &["", "b\"\"", "123", "b\"G\""] {
            parse_as_byte_array(s).unwrap_err();
        }
    }

    #[test]
    fn parse_args() {
        for s in &["123", "0xf", "b\"aaa\""] {
            parse_as_transaction_argument(s).unwrap();
        }

        for s in &["garbage", ""] {
            parse_as_transaction_argument(s).unwrap_err();
        }
    }
}
