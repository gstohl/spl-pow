use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{create_program_address, find_program_address, Pubkey},
};

use crate::{error::PowError, hash::PowHash};

pub const CONFIG_SEED: &[u8] = b"pow-config";
pub const CONFIG_LEN: usize = 147;

const OFFSET_INITIALIZED: usize = 0;
const OFFSET_AUTHORITY: usize = 1;
const OFFSET_REWARD_MINT: usize = 33;
const OFFSET_VAULT: usize = 65;
const OFFSET_DIFFICULTY: usize = 97;
const OFFSET_REWARD_AMOUNT: usize = 98;
const OFFSET_TOTAL_SOLUTIONS: usize = 106;
const OFFSET_CHALLENGE: usize = 114;
const OFFSET_BUMP: usize = 146;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    pub authority: Pubkey,
    pub reward_mint: Pubkey,
    pub vault: Pubkey,
    pub difficulty: u8,
    pub reward_amount: u64,
    pub total_solutions: u64,
    pub challenge: PowHash,
    pub bump: u8,
}

impl Config {
    pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let data = unsafe { account.borrow_data_unchecked() };
        Self::from_bytes(data)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[OFFSET_INITIALIZED] != 1 {
            return Err(ProgramError::UninitializedAccount);
        }

        let mut authority = [0u8; 32];
        authority.copy_from_slice(&data[OFFSET_AUTHORITY..OFFSET_AUTHORITY + 32]);

        let mut reward_mint = [0u8; 32];
        reward_mint.copy_from_slice(&data[OFFSET_REWARD_MINT..OFFSET_REWARD_MINT + 32]);

        let mut vault = [0u8; 32];
        vault.copy_from_slice(&data[OFFSET_VAULT..OFFSET_VAULT + 32]);

        let difficulty = data[OFFSET_DIFFICULTY];

        let reward_amount = u64::from_le_bytes(
            data[OFFSET_REWARD_AMOUNT..OFFSET_REWARD_AMOUNT + 8]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let total_solutions = u64::from_le_bytes(
            data[OFFSET_TOTAL_SOLUTIONS..OFFSET_TOTAL_SOLUTIONS + 8]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        let mut challenge = [0u8; 32];
        challenge.copy_from_slice(&data[OFFSET_CHALLENGE..OFFSET_CHALLENGE + 32]);

        Ok(Self {
            authority,
            reward_mint,
            vault,
            difficulty,
            reward_amount,
            total_solutions,
            challenge,
            bump: data[OFFSET_BUMP],
        })
    }

    pub fn store(&self, account: &AccountInfo) -> Result<(), ProgramError> {
        let data = &mut *account.try_borrow_mut_data()?;
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        data.fill(0);
        data[OFFSET_INITIALIZED] = 1;
        data[OFFSET_AUTHORITY..OFFSET_AUTHORITY + 32].copy_from_slice(&self.authority);
        data[OFFSET_REWARD_MINT..OFFSET_REWARD_MINT + 32].copy_from_slice(&self.reward_mint);
        data[OFFSET_VAULT..OFFSET_VAULT + 32].copy_from_slice(&self.vault);
        data[OFFSET_DIFFICULTY] = self.difficulty;
        data[OFFSET_REWARD_AMOUNT..OFFSET_REWARD_AMOUNT + 8]
            .copy_from_slice(&self.reward_amount.to_le_bytes());
        data[OFFSET_TOTAL_SOLUTIONS..OFFSET_TOTAL_SOLUTIONS + 8]
            .copy_from_slice(&self.total_solutions.to_le_bytes());
        data[OFFSET_CHALLENGE..OFFSET_CHALLENGE + 32].copy_from_slice(&self.challenge);
        data[OFFSET_BUMP] = self.bump;

        Ok(())
    }

    pub fn is_initialized(account: &AccountInfo) -> Result<bool, ProgramError> {
        let data = unsafe { account.borrow_data_unchecked() };
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        Ok(data[OFFSET_INITIALIZED] == 1)
    }
}

pub fn derive_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[CONFIG_SEED], program_id)
}

pub fn assert_config_pda(
    program_id: &Pubkey,
    account: &AccountInfo,
    bump: u8,
) -> Result<(), ProgramError> {
    let bump_seed = [bump];
    let derived = create_program_address(&[CONFIG_SEED, &bump_seed], program_id)
        .map_err(|_| ProgramError::from(PowError::InvalidConfigPda))?;

    if account.key() != &derived {
        return Err(PowError::InvalidConfigPda.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Config, CONFIG_LEN};

    #[test]
    fn config_roundtrip() {
        let original = Config {
            authority: [1u8; 32],
            reward_mint: [2u8; 32],
            vault: [4u8; 32],
            difficulty: 11,
            reward_amount: 10_000,
            total_solutions: 7,
            challenge: [3u8; 32],
            bump: 254,
        };

        let mut bytes = [0u8; CONFIG_LEN];
        bytes[0] = 1;
        bytes[1..33].copy_from_slice(&original.authority);
        bytes[33..65].copy_from_slice(&original.reward_mint);
        bytes[65..97].copy_from_slice(&original.vault);
        bytes[97] = original.difficulty;
        bytes[98..106].copy_from_slice(&original.reward_amount.to_le_bytes());
        bytes[106..114].copy_from_slice(&original.total_solutions.to_le_bytes());
        bytes[114..146].copy_from_slice(&original.challenge);
        bytes[146] = original.bump;

        let decoded = Config::from_bytes(&bytes).expect("config should decode");
        assert_eq!(decoded, original);
    }
}
