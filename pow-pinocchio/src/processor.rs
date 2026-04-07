use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    program_error::ProgramError,
    pubkey::{pubkey_eq, Pubkey},
    seeds, ProgramResult,
};
use pinocchio_tkn::{
    common::TransferChecked,
    helpers::{
        assert_account_not_frozen, assert_is_mint, assert_is_token_account, assert_owned_by,
        get_token_program_id,
    },
    state::Mint,
};

use crate::{
    error::PowError,
    hash::{genesis_challenge, next_challenge, satisfies_difficulty, solution_hash},
    instruction::PowInstruction,
    state::{assert_config_pda, derive_config_pda, Config},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    difficulty: u8,
    reward_amount: u64,
) -> ProgramResult {
    let [config_account, authority, reward_mint, vault_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !config_account.is_writable() {
        return Err(PowError::ConfigNotWritable.into());
    }
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !pubkey_eq(config_account.owner(), program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }
    if Config::is_initialized(config_account)? {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let (expected_pda, bump) = derive_config_pda(program_id);
    if !pubkey_eq(config_account.key(), &expected_pda) {
        return Err(PowError::InvalidConfigPda.into());
    }

    assert_is_mint(reward_mint)?;
    let token_program_id = get_token_program_id(reward_mint);
    assert_owned_by(vault_account, token_program_id)?;
    assert_is_token_account(
        vault_account,
        Some(reward_mint.key()),
        Some(config_account.key()),
    )?;
    assert_account_not_frozen(vault_account)?;

    let config = Config {
        authority: *authority.key(),
        reward_mint: *reward_mint.key(),
        vault: *vault_account.key(),
        difficulty,
        reward_amount,
        total_solutions: 0,
        challenge: genesis_challenge(
            program_id,
            authority.key(),
            reward_mint.key(),
            vault_account.key(),
            difficulty,
            reward_amount,
        ),
        bump,
    };

    config.store(config_account)
}

fn process_mine(program_id: &Pubkey, accounts: &[AccountInfo], nonce: u64) -> ProgramResult {
    let [config_account, vault_account, reward_mint, miner_reward_account, miner] = accounts else {
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
    if !pubkey_eq(config_account.owner(), program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut config = Config::load(config_account)?;
    assert_config_pda(program_id, config_account, config.bump)?;

    if !pubkey_eq(reward_mint.key(), &config.reward_mint) {
        return Err(PowError::WrongRewardMint.into());
    }
    if !pubkey_eq(vault_account.key(), &config.vault) {
        return Err(PowError::WrongVaultAccount.into());
    }

    let token_program_id = get_token_program_id(reward_mint);
    assert_owned_by(vault_account, token_program_id)?;
    assert_owned_by(miner_reward_account, token_program_id)?;
    assert_is_token_account(
        vault_account,
        Some(reward_mint.key()),
        Some(config_account.key()),
    )?;
    assert_is_mint(reward_mint)?;
    assert_is_token_account(
        miner_reward_account,
        Some(reward_mint.key()),
        Some(miner.key()),
    )?;
    assert_account_not_frozen(vault_account)?;
    assert_account_not_frozen(miner_reward_account)?;

    let mint_state = Mint::from_account_info(reward_mint)?;

    let candidate = solution_hash(&config.challenge, miner.key(), nonce);
    if !satisfies_difficulty(&candidate, config.difficulty) {
        return Err(PowError::HashMissesTarget.into());
    }

    config.total_solutions = config
        .total_solutions
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    config.challenge = next_challenge(&candidate, miner.key(), config.total_solutions);

    let bump_seed = [config.bump];
    let signer_seeds = seeds!(crate::state::CONFIG_SEED, &bump_seed);
    let signer = Signer::from(&signer_seeds);

    TransferChecked {
        source: vault_account,
        mint: reward_mint,
        destination: miner_reward_account,
        authority: config_account,
        amount: config.reward_amount,
        decimals: mint_state.decimals(),
        program_id: Some(token_program_id),
    }
    .invoke_signed(&[signer])?;

    config.store(config_account)
}

fn process_set_difficulty(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
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
    if !pubkey_eq(config_account.owner(), program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut config = Config::load(config_account)?;
    assert_config_pda(program_id, config_account, config.bump)?;

    if !pubkey_eq(authority.key(), &config.authority) {
        return Err(PowError::Unauthorized.into());
    }

    config.difficulty = difficulty;
    config.store(config_account)
}
