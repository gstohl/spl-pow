use pinocchio::Address;

#[cfg(target_os = "solana")]
use pinocchio::syscalls::sol_sha256;

pub type PowHash = [u8; 32];

#[inline]
pub fn sha256v(values: &[&[u8]]) -> PowHash {
    #[cfg(target_os = "solana")]
    {
        let mut output = [0u8; 32];
        unsafe {
            sol_sha256(
                values.as_ptr() as *const u8,
                values.len() as u64,
                output.as_mut_ptr(),
            );
        }
        output
    }

    #[cfg(not(target_os = "solana"))]
    {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        for value in values {
            hasher.update(value);
        }

        let digest = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&digest);
        output
    }
}

#[inline]
pub fn solution_hash(challenge: &PowHash, miner: &Address, nonce: u64) -> PowHash {
    let nonce_bytes = nonce.to_le_bytes();
    sha256v(&[challenge.as_ref(), miner.as_ref(), &nonce_bytes])
}

#[inline]
pub fn next_challenge(
    previous_challenge: &PowHash,
    solution: &PowHash,
    total_solutions: u64,
    recent_slot_hash: &PowHash,
) -> PowHash {
    let total_bytes = total_solutions.to_le_bytes();
    sha256v(&[
        b"pow-next",
        previous_challenge.as_ref(),
        solution.as_ref(),
        recent_slot_hash.as_ref(),
        &total_bytes,
    ])
}

#[inline]
pub fn genesis_challenge(
    program_id: &Address,
    authority: &Address,
    reward_mint: &Address,
    vault: &Address,
    difficulty: u8,
    reward_amount: u64,
    recent_slot_hash: &PowHash,
) -> PowHash {
    let reward_bytes = reward_amount.to_le_bytes();
    let difficulty_bytes = [difficulty];
    sha256v(&[
        b"pow-genesis",
        program_id.as_ref(),
        authority.as_ref(),
        reward_mint.as_ref(),
        vault.as_ref(),
        &difficulty_bytes,
        &reward_bytes,
        recent_slot_hash.as_ref(),
    ])
}

#[inline]
pub fn leading_zero_bits(hash: &PowHash) -> u16 {
    let mut total = 0u16;

    for byte in hash {
        if *byte == 0 {
            total += 8;
            continue;
        }

        total += byte.leading_zeros() as u16;
        break;
    }

    total
}

#[inline]
pub fn satisfies_difficulty(hash: &PowHash, difficulty: u8) -> bool {
    leading_zero_bits(hash) >= difficulty as u16
}

#[cfg(test)]
mod tests {
    use pinocchio::Address;

    use super::{leading_zero_bits, satisfies_difficulty, solution_hash};

    #[test]
    fn counts_zero_bits_correctly() {
        assert_eq!(leading_zero_bits(&[0u8; 32]), 256);
        assert_eq!(
            leading_zero_bits(&[
                0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff,
            ]),
            20
        );
        assert!(!satisfies_difficulty(&[0xff; 32], 1));
    }

    #[test]
    fn finds_nonce_for_easy_target() {
        let challenge = [7u8; 32];
        let miner = Address::new_from_array([9u8; 32]);

        let nonce = (0u64..50_000)
            .find(|candidate| {
                satisfies_difficulty(&solution_hash(&challenge, &miner, *candidate), 8)
            })
            .expect("expected an easy nonce");

        assert!(satisfies_difficulty(
            &solution_hash(&challenge, &miner, nonce),
            8
        ));
    }
}
