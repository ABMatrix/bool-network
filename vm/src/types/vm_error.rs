use crate::types::ModuleId;
use failure::Fail;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum VMValidationStatus {
    InvalidSignature,
    InvalidAuthKey,
    SequenceNumberTooOld,
    SequenceNumberTooNew,
    InsufficientBalanceForTransactionFee,
    TransactionExpired,
    SendingAccountDoesNotExist(String),
    RejectedWriteSet,
    InvalidWriteSet,
    ExceededMaxTransactionSize(String),
    UnknownScript,
    UnknownModule,
    MaxGasUnitsExceedsMaxGasUnitsBound(String),
    MaxGasUnitsBelowMinTransactionGasUnits(String),
    GasUnitPriceBelowMinBound(String),
    GasUnitPriceAboveMaxBound(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum VMVerificationError {
    IndexOutOfBounds(String),
    RangeOutOfBounds(String),
    NoModuleHandles(String),
    ModuleAddressDoesNotMatchSender(String),
    InvalidSignatureToken(String),
    InvalidFieldDefReference(String),
    RecursiveStructDefinition(String),
    InvalidResourceField(String),
    InvalidFallThrough(String),
    JoinFailure(String),
    NegativeStackSizeWithinBlock(String),
    UnbalancedStack(String),
    InvalidMainFunctionSignature(String),
    DuplicateElement(String),
    InvalidModuleHandle(String),
    UnimplementedHandle(String),
    InconsistentFields(String),
    UnusedFields(String),
    LookupFailed(String),
    VisibilityMismatch(String),
    TypeResolutionFailure(String),
    TypeMismatch(String),
    MissingDependency(String),
    PopReferenceError(String),
    PopResourceError(String),
    ReleaseRefTypeMismatchError(String),
    BrTypeMismatchError(String),
    AbortTypeMismatchError(String),
    StLocTypeMismatchError(String),
    StLocUnsafeToDestroyError(String),
    RetUnsafeToDestroyError(String),
    RetTypeMismatchError(String),
    FreezeRefTypeMismatchError(String),
    FreezeRefExistsMutableBorrowError(String),
    BorrowFieldTypeMismatchError(String),
    BorrowFieldBadFieldError(String),
    BorrowFieldExistsMutableBorrowError(String),
    CopyLocUnavailableError(String),
    CopyLocResourceError(String),
    CopyLocExistsBorrowError(String),
    MoveLocUnavailableError(String),
    MoveLocExistsBorrowError(String),
    BorrowLocReferenceError(String),
    BorrowLocUnavailableError(String),
    BorrowLocExistsBorrowError(String),
    CallTypeMismatchError(String),
    CallBorrowedMutableReferenceError(String),
    PackTypeMismatchError(String),
    UnpackTypeMismatchError(String),
    ReadRefTypeMismatchError(String),
    ReadRefResourceError(String),
    ReadRefExistsMutableBorrowError(String),
    WriteRefTypeMismatchError(String),
    WriteRefResourceError(String),
    WriteRefExistsBorrowError(String),
    WriteRefNoMutableReferenceError(String),
    IntegerOpTypeMismatchError(String),
    BooleanOpTypeMismatchError(String),
    EqualityOpTypeMismatchError(String),
    ExistsResourceTypeMismatchError(String),
    BorrowGlobalTypeMismatchError(String),
    BorrowGlobalNoResourceError(String),
    MoveFromTypeMismatchError(String),
    MoveFromNoResourceError(String),
    MoveToSenderTypeMismatchError(String),
    MoveToSenderNoResourceError(String),
    CreateAccountTypeMismatchError(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum VMVerificationStatus {
    /// Verification error in a transaction script.
    Script(VMVerificationError),
    /// Verification error in a module -- the first element is the index of the module with the
    /// error.
    Module(u16, VMVerificationError),
    /// Verification error in a dependent module.
    Dependency(ModuleId, VMVerificationError),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum VMInvariantViolationError {
    OutOfBoundsIndex,
    OutOfBoundsRange,
    EmptyValueStack,
    EmptyCallStack,
    PCOverflow,
    LinkerError,
    LocalReferenceError,
    StorageError,
    InternalTypeError,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BinaryError {
    Malformed,
    BadMagic,
    UnknownVersion,
    UnknownTableType,
    UnknownSignatureType,
    UnknownSerializedType,
    UnknownOpcode,
    BadHeaderTable,
    UnexpectedSignatureType,
    DuplicateTable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum DynamicReferenceErrorType {
    MoveOfBorrowedResource,
    GlobalRefAlreadyReleased,
    MissingReleaseRef,
    GlobalAlreadyBorrowed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ArithmeticErrorType {
    Underflow,
    Overflow,
    DivisionByZero,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ExecutionStatus {
    Executed,
    OutOfGas,
    ResourceDoesNotExist,
    ResourceAlreadyExists,
    EvictedAccountAccess,
    AccountAddressAlreadyExists,
    TypeError,
    MissingData,
    DataFormatError,
    InvalidData,
    RemoteDataError,
    CannotWriteExistingResource,
    ValueSerializationError,
    ValueDeserializationError,
    Aborted(u64),
    ArithmeticError(ArithmeticErrorType),
    DynamicReferenceError(DynamicReferenceErrorType),
    DuplicateModuleName,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum VMStatus {
    Validation(VMValidationStatus),
    InvariantViolation(VMInvariantViolationError),
    Deserialization(BinaryError),
    Execution(ExecutionStatus),
    Verification(Vec<VMVerificationStatus>),
}

#[derive(Debug, Fail, Eq, PartialEq)]
pub enum DecodingError {
    #[fail(display = "Module index {} greater than max possible value 65535", _0)]
    ModuleIndexTooBig(u32),
    #[fail(display = "Unknown Validation Status Encountered")]
    UnknownValidationStatusEncountered,
    #[fail(display = "Unknown Verification Error Encountered")]
    UnknownVerificationErrorEncountered,
    #[fail(display = "Unknown Invariant Violation Error Encountered")]
    UnknownInvariantViolationErrorEncountered,
    #[fail(display = "Unknown Transaction Binary Decoding Error Encountered")]
    UnknownBinaryErrorEncountered,
    #[fail(display = "Unknown Reference Error Type Encountered")]
    UnknownDynamicReferenceErrorTypeEncountered,
    #[fail(display = "Unknown Arithmetic Error Type Encountered")]
    UnknownArithmeticErrorTypeEncountered,
    #[fail(display = "Unknown Runtime Status Encountered")]
    UnknownRuntimeStatusEncountered,
    #[fail(display = "Unknown/Invalid VM Status Encountered")]
    InvalidVMStatusEncountered,
}
