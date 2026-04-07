use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::slot_hashes::SlotHashes,
    AccountView, Address, ProgramResult,
};
use pinocchio_token::{
    instructions::TransferChecked,
    state::{Mint, TokenAccount},
};

use crate::{
    error::PowError,
    hash::{genesis_challenge, next_challenge, satisfies_difficulty, solution_hash},
    instruction::PowInstruction,
    state::{assert_config_pda, derive_config_pda, Config},
};

#[inline(never)]
pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match PowInstruction::unpack(instruction_data)? {
        PowInstruction::Initialize {
            difficulty,
            reward_amount,
        } => process_initialize(program_id, accounts, difficulty, reward_amount),
        PowInstruction::Mine { nonce } => process_mine(program_id, accounts, nonce),
        PowInstruction::SetDifficulty { difficulty } => {
            process_set_difficulty(program_id, accounts, difficulty)
        }
    }
}

fn process_initialize(
    program_id: &Address,
    accounts: &[AccountView],
    difficulty: u8,
    reward_amount: u64,
) -> ProgramResult {
    let [config_account, authority, reward_mint, vault_account, slot_hashes] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !config_account.is_writable() {
        return Err(PowError::ConfigNotWritable.into());
    }
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !config_account.owned_by(program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }
    if Config::is_initialized(config_account)? {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (expected_pda, bump) = derive_config_pda(program_id, authority.address());
    if config_account.address() != &expected_pda {
        return Err(PowError::InvalidConfigPda.into());
    }

    let reward_mint_state = Mint::from_account_view(reward_mint)?;
    assert_mint_initialized(&reward_mint_state)?;
    let vault_state = TokenAccount::from_account_view(vault_account)?;
    assert_token_account(
        &vault_state,
        reward_mint.address(),
        config_account.address(),
    )?;
    assert_not_frozen(&vault_state)?;

    let recent_slot_hash = latest_slot_hash(slot_hashes)?;

    let config = Config {
        authority: authority.address().clone(),
        reward_mint: reward_mint.address().clone(),
        vault: vault_account.address().clone(),
        difficulty,
        reward_amount,
        total_solutions: 0,
        challenge: genesis_challenge(
            program_id,
            authority.address(),
            reward_mint.address(),
            vault_account.address(),
            difficulty,
            reward_amount,
            &recent_slot_hash,
        ),
        bump,
    };

    config.store(config_account)
}

fn process_mine(program_id: &Address, accounts: &[AccountView], nonce: u64) -> ProgramResult {
    let [config_account, vault_account, reward_mint, miner_reward_account, miner, slot_hashes] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !miner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !config_account.is_writable()
        || !vault_account.is_writable()
        || !miner_reward_account.is_writable()
    {
        return Err(PowError::ConfigNotWritable.into());
    }
    if !config_account.owned_by(program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut config = Config::load(config_account)?;
    assert_config_pda(program_id, config_account, &config.authority, config.bump)?;

    if reward_mint.address() != &config.reward_mint {
        return Err(PowError::WrongRewardMint.into());
    }
    if vault_account.address() != &config.vault {
        return Err(PowError::WrongVaultAccount.into());
    }

    let mint_state = Mint::from_account_view(reward_mint)?;
    assert_mint_initialized(&mint_state)?;
    let vault_state = TokenAccount::from_account_view(vault_account)?;
    let miner_state = TokenAccount::from_account_view(miner_reward_account)?;
    assert_token_account(
        &vault_state,
        reward_mint.address(),
        config_account.address(),
    )?;
    assert_token_account(&miner_state, reward_mint.address(), miner.address())?;
    assert_not_frozen(&vault_state)?;
    assert_not_frozen(&miner_state)?;

    let candidate = solution_hash(&config.challenge, miner.address(), nonce);
    if !satisfies_difficulty(&candidate, config.difficulty) {
        return Err(PowError::HashMissesTarget.into());
    }

    let previous_challenge = config.challenge;
    let recent_slot_hash = latest_slot_hash(slot_hashes)?;
    config.total_solutions = config
        .total_solutions
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    config.challenge = next_challenge(
        &previous_challenge,
        &candidate,
        config.total_solutions,
        &recent_slot_hash,
    );

    let bump_seed = [config.bump];
    let signer_seeds = [
        Seed::from(crate::state::CONFIG_SEED),
        Seed::from(config.authority.as_ref()),
        Seed::from(&bump_seed),
    ];
    let signer = Signer::from(&signer_seeds);

    TransferChecked {
        from: vault_account,
        mint: reward_mint,
        to: miner_reward_account,
        authority: config_account,
        amount: config.reward_amount,
        decimals: mint_state.decimals(),
    }
    .invoke_signed(&[signer])?;

    config.store(config_account)
}

fn process_set_difficulty(
    program_id: &Address,
    accounts: &[AccountView],
    difficulty: u8,
) -> ProgramResult {
    let [config_account, authority] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !config_account.is_writable() {
        return Err(PowError::ConfigNotWritable.into());
    }
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !config_account.owned_by(program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut config = Config::load(config_account)?;
    assert_config_pda(program_id, config_account, &config.authority, config.bump)?;

    if authority.address() != &config.authority {
        return Err(PowError::Unauthorized.into());
    }

    config.difficulty = difficulty;
    config.store(config_account)
}

fn latest_slot_hash(slot_hashes_account: &AccountView) -> Result<[u8; 32], ProgramError> {
    let slot_hashes = SlotHashes::from_account_view(slot_hashes_account)?;
    let entry = slot_hashes
        .get_entry(0)
        .ok_or(ProgramError::UnsupportedSysvar)?;
    Ok(entry.hash)
}

fn assert_mint_initialized(mint: &Mint) -> Result<(), ProgramError> {
    if !mint.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(())
}

fn assert_token_account(
    account: &TokenAccount,
    expected_mint: &Address,
    expected_owner: &Address,
) -> Result<(), ProgramError> {
    if account.mint() != expected_mint {
        return Err(ProgramError::InvalidAccountData);
    }
    if account.owner() != expected_owner {
        return Err(ProgramError::IllegalOwner);
    }
    if !account.is_initialized() {
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(())
}

fn assert_not_frozen(account: &TokenAccount) -> Result<(), ProgramError> {
    if account.is_frozen() {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}
