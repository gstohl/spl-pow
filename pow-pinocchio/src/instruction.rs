use pinocchio::program_error::ProgramError;

use crate::error::PowError;

#[derive(Debug)]
pub enum PowInstruction {
    Initialize { difficulty: u8, reward_amount: u64 },
    Mine { nonce: u64 },
    SetDifficulty { difficulty: u8 },
}

impl PowInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match tag {
            0 => {
                if rest.len() != 9 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let difficulty = rest[0];
                let reward_amount = u64::from_le_bytes(
                    rest[1..9]
                        .try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(Self::Initialize {
                    difficulty,
                    reward_amount,
                })
            }
            1 => {
                if rest.len() != 8 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                let nonce = u64::from_le_bytes(
                    rest.try_into()
                        .map_err(|_| ProgramError::InvalidInstructionData)?,
                );

                Ok(Self::Mine { nonce })
            }
            2 => {
                if rest.len() != 1 {
                    return Err(ProgramError::InvalidInstructionData);
                }

                Ok(Self::SetDifficulty {
                    difficulty: rest[0],
                })
            }
            _ => Err(PowError::InvalidInstruction.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use pinocchio::program_error::ProgramError;

    use super::PowInstruction;

    #[test]
    fn unpacks_initialize() {
        let mut data = [0u8; 10];
        data[0] = 0;
        data[1] = 8;
        data[2..10].copy_from_slice(&42u64.to_le_bytes());

        match PowInstruction::unpack(&data).expect("initialize should parse") {
            PowInstruction::Initialize {
                difficulty,
                reward_amount,
            } => {
                assert_eq!(difficulty, 8);
                assert_eq!(reward_amount, 42);
            }
            _ => panic!("wrong instruction"),
        }
    }

    #[test]
    fn rejects_bad_tag() {
        let error = PowInstruction::unpack(&[9]).expect_err("bad tag should fail");
        assert!(matches!(error, ProgramError::Custom(6_000)));
    }
}
