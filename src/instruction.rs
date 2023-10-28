use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// Instructions supported by the vault program.
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum VaultInstruction {
    /// Initialize a vault record (by DART on behalf of a given authority).
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable]` The vault record account (must be uninitialized).
    /// 1. `[signer]` The securities intermediary (DART)
    /// 2. `[]` The record authority (trader)
    Initialize,

    /// Transfer ownership of a vault record
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable]` The vault record account (must be previously initialized).
    /// 1. `[signer]` The securities intermediary (DART)
    /// 2. `[signer]` The current record authority.
    /// 3. `[]` The new record authority
    TransferAuthority,

    /// Close a vault record account, draining lamports to the current authority.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable]` The vault record account (must be previously initialized).
    /// 1. `[signer]` The securities intermediary (DART)
    /// 2. `[signer, writable]` The record authority (receiver of account lamports).
    CloseAccount,
}

/// Create a `VaultInstruction::Initialize` instruction
pub fn initialize(
    program_id: Pubkey,
    pda: &Pubkey,
    dart: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    Instruction::new_with_borsh(
        program_id,
        &VaultInstruction::Initialize,
        vec![
            AccountMeta::new(*pda, false),
            AccountMeta::new_readonly(*dart, true),
            AccountMeta::new_readonly(*authority, false),
        ],
    )
}

/// Create a `VaultInstruction::TransferAuthority` instruction
pub fn transfer_authority(
    program_id: Pubkey,
    pda: &Pubkey,
    dart: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    Instruction::new_with_borsh(
        program_id,
        &VaultInstruction::TransferAuthority,
        vec![
            AccountMeta::new(*pda, false),
            AccountMeta::new_readonly(*dart, true),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(*new_authority, false),
        ],
    )
}

/// Create a `VaultInstruction::CloseAccount` instruction
pub fn close_account(
    program_id: Pubkey,
    pda: &Pubkey,
    dart: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    Instruction::new_with_borsh(
        program_id,
        &VaultInstruction::CloseAccount,
        vec![
            AccountMeta::new(*pda, false),
            AccountMeta::new_readonly(*dart, true),
            AccountMeta::new(*authority, true),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::program_error::ProgramError;

    /// very small data for easy testing
    const DATA_SIZE: usize = 8;

    /// Bytes for tests
    const TEST_BYTES: [u8; DATA_SIZE] = [42; DATA_SIZE];

    #[test]
    fn serialize_initialize() {
        let instruction = VaultInstruction::Initialize;
        let expected = vec![0];
        assert_eq!(instruction.try_to_vec().unwrap(), expected);
        assert_eq!(
            VaultInstruction::try_from_slice(&expected).unwrap(),
            instruction
        );
    }

    #[test]
    fn serialize_transfer_authority() {
        let instruction = VaultInstruction::TransferAuthority;
        let expected = vec![1];
        assert_eq!(instruction.try_to_vec().unwrap(), expected);
        assert_eq!(
            VaultInstruction::try_from_slice(&expected).unwrap(),
            instruction
        );
    }

    #[test]
    fn serialize_close_account() {
        let instruction = VaultInstruction::CloseAccount;
        let expected = vec![2];
        assert_eq!(instruction.try_to_vec().unwrap(), expected);
        assert_eq!(
            VaultInstruction::try_from_slice(&expected).unwrap(),
            instruction
        );
    }

    #[test]
    fn deserialize_invalid_instruction() {
        let mut expected = vec![12];
        expected.append(&mut TEST_BYTES.try_to_vec().unwrap());
        let err: ProgramError = VaultInstruction::try_from_slice(&expected)
            .unwrap_err()
            .into();
        assert!(matches!(err, ProgramError::BorshIoError(_)));
    }
}
