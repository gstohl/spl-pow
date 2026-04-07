use pinocchio::{error::ProgramError, AccountView, Address};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub authority: Address,
    pub reward_mint: Address,
    pub vault: Address,
    pub difficulty: u8,
    pub reward_amount: u64,
    pub total_solutions: u64,
    pub challenge: PowHash,
    pub bump: u8,
}

impl Config {
    pub fn load(account: &AccountView) -> Result<Self, ProgramError> {
        let data = account.try_borrow()?;
        Self::from_bytes(&data)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        if data[OFFSET_INITIALIZED] != 1 {
            return Err(ProgramError::UninitializedAccount);
        }

        let authority = Address::new_from_array(
            data[OFFSET_AUTHORITY..OFFSET_AUTHORITY + 32]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );
        let reward_mint = Address::new_from_array(
            data[OFFSET_REWARD_MINT..OFFSET_REWARD_MINT + 32]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );
        let vault = Address::new_from_array(
            data[OFFSET_VAULT..OFFSET_VAULT + 32]
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

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

    pub fn store(&self, account: &AccountView) -> Result<(), ProgramError> {
        let data = &mut *account.try_borrow_mut()?;
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        data[OFFSET_INITIALIZED] = 1;
        data[OFFSET_AUTHORITY..OFFSET_AUTHORITY + 32].copy_from_slice(self.authority.as_ref());
        data[OFFSET_REWARD_MINT..OFFSET_REWARD_MINT + 32]
            .copy_from_slice(self.reward_mint.as_ref());
        data[OFFSET_VAULT..OFFSET_VAULT + 32].copy_from_slice(self.vault.as_ref());
        data[OFFSET_DIFFICULTY] = self.difficulty;
        data[OFFSET_REWARD_AMOUNT..OFFSET_REWARD_AMOUNT + 8]
            .copy_from_slice(&self.reward_amount.to_le_bytes());
        data[OFFSET_TOTAL_SOLUTIONS..OFFSET_TOTAL_SOLUTIONS + 8]
            .copy_from_slice(&self.total_solutions.to_le_bytes());
        data[OFFSET_CHALLENGE..OFFSET_CHALLENGE + 32].copy_from_slice(&self.challenge);
        data[OFFSET_BUMP] = self.bump;

        Ok(())
    }

    pub fn is_initialized(account: &AccountView) -> Result<bool, ProgramError> {
        let data = account.try_borrow()?;
        if data.len() < CONFIG_LEN {
            return Err(ProgramError::AccountDataTooSmall);
        }

        Ok(data[OFFSET_INITIALIZED] == 1)
    }
}

pub fn derive_config_pda(program_id: &Address, authority: &Address) -> (Address, u8) {
    Address::find_program_address(&[CONFIG_SEED, authority.as_ref()], program_id)
}

pub fn assert_config_pda(
    program_id: &Address,
    account: &AccountView,
    authority: &Address,
    bump: u8,
) -> Result<(), ProgramError> {
    let bump_seed = [bump];
    let derived =
        Address::create_program_address(&[CONFIG_SEED, authority.as_ref(), &bump_seed], program_id)
            .map_err(|_| ProgramError::from(PowError::InvalidConfigPda))?;

    if account.address() != &derived {
        return Err(PowError::InvalidConfigPda.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use pinocchio::Address;

    use super::{Config, CONFIG_LEN};

    #[test]
    fn config_roundtrip() {
        let original = Config {
            authority: Address::new_from_array([1u8; 32]),
            reward_mint: Address::new_from_array([2u8; 32]),
            vault: Address::new_from_array([4u8; 32]),
            difficulty: 11,
            reward_amount: 10_000,
            total_solutions: 7,
            challenge: [3u8; 32],
            bump: 254,
        };

        let mut bytes = [0u8; CONFIG_LEN];
        bytes[0] = 1;
        bytes[1..33].copy_from_slice(original.authority.as_ref());
        bytes[33..65].copy_from_slice(original.reward_mint.as_ref());
        bytes[65..97].copy_from_slice(original.vault.as_ref());
        bytes[97] = original.difficulty;
        bytes[98..106].copy_from_slice(&original.reward_amount.to_le_bytes());
        bytes[106..114].copy_from_slice(&original.total_solutions.to_le_bytes());
        bytes[114..146].copy_from_slice(&original.challenge);
        bytes[146] = original.bump;

        let decoded = Config::from_bytes(&bytes).expect("config should decode");
        assert_eq!(decoded, original);
    }
}
