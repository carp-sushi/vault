use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{program_pack::IsInitialized, pubkey::Pubkey},
};

/// Struct providing metadata (and could be extended to support data).
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct VaultRecord {
    /// Struct version, allows for upgrades to the program
    pub version: u8,

    /// The account owner
    pub authority: Pubkey,

    /// The securities intermediary
    pub dart: Pubkey,
}

impl VaultRecord {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;
    /// Packed vault record space
    pub const LEN: usize = 65; // 1 + 32 + 32
}

impl IsInitialized for VaultRecord {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use solana_program::program_error::ProgramError;

    /// Version for tests
    pub const TEST_VERSION: u8 = 1;
    /// Authority pubkey
    pub const AUTH_PUBKEY: Pubkey = Pubkey::new_from_array([99; 32]);
    /// DART pubkey
    pub const DART_PUBKEY: Pubkey = Pubkey::new_from_array([66; 32]);
    /// VaultRecord for tests
    pub const TEST_RECORD_DATA: VaultRecord = VaultRecord {
        version: TEST_VERSION,
        authority: AUTH_PUBKEY,
        dart: DART_PUBKEY,
    };

    #[test]
    fn serialize_data() {
        let mut expected = vec![TEST_VERSION];
        expected.extend_from_slice(&AUTH_PUBKEY.to_bytes());
        expected.extend_from_slice(&DART_PUBKEY.to_bytes());
        assert_eq!(TEST_RECORD_DATA.try_to_vec().unwrap(), expected);
        assert_eq!(
            VaultRecord::try_from_slice(&expected).unwrap(),
            TEST_RECORD_DATA
        );
    }

    #[test]
    fn deserialize_invalid_slice() {
        let mut expected = vec![TEST_VERSION];
        expected.extend_from_slice(&AUTH_PUBKEY.to_bytes());
        let err: ProgramError = VaultRecord::try_from_slice(&expected).unwrap_err().into();
        assert!(matches!(err, ProgramError::BorshIoError(_)));
    }
}
