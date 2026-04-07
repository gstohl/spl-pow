use core::convert::From;

use pinocchio::program_error::ProgramError;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowError {
    InvalidInstruction = 6_000,
    InvalidConfigPda = 6_001,
    Unauthorized = 6_002,
    HashMissesTarget = 6_003,
    WrongRewardMint = 6_004,
    WrongVaultAccount = 6_005,
    ConfigNotWritable = 6_006,
    VaultAuthorityMismatch = 6_007,
}

impl From<PowError> for ProgramError {
    fn from(value: PowError) -> Self {
        ProgramError::Custom(value as u32)
    }
}
