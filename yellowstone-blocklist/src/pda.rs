//! Program Derived Address (PDA) handling module for the blocklist program.
//!
//! Note: Pinocchio's PDA functions only work in production (on Solana runtime) as they rely on syscalls.
//! For testing, we use solana-sdk which provides a local implementation of PDA derivation.

use pinocchio::{
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[cfg(test)]
use solana_sdk;

pub struct BlockList;

impl BlockList {
    pub const SEED_PREFIX: &'static [u8] = b"noneknows";

    pub fn get_signer_seeds(authority: &Pubkey) -> [Seed; 2] {
        [
            Seed::from(Self::SEED_PREFIX),
            Seed::from(authority.as_ref()),
        ]
    }

    pub fn get_signers<'a>(seeds: &'a [Seed<'a>]) -> Vec<Signer<'a, 'a>> {
        vec![Signer::from(seeds)]
    }

    pub fn seeds(authority: &Pubkey) -> Vec<&[u8]> {
        vec![Self::SEED_PREFIX, authority.as_ref()]
    }

    #[cfg(not(test))]
    pub fn pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
        let seeds = Self::seeds(authority);
        pinocchio::pubkey::find_program_address(&seeds, program_id)
    }

    #[cfg(test)]
    pub fn pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
        use solana_sdk::pubkey::Pubkey as SolanaPubkey;
        let program_id = SolanaPubkey::new_from_array(*program_id);
        let authority = SolanaPubkey::new_from_array(*authority);

        let (pda, bump) = SolanaPubkey::find_program_address(
            &[Self::SEED_PREFIX, authority.as_ref()],
            &program_id,
        );
        (pda.to_bytes(), bump)
    }

    pub fn check_pda(
        program_id: &Pubkey,
        authority: &Pubkey,
        pda: &Pubkey,
    ) -> Result<u8, ProgramError> {
        let (pda_check, bump_seed) = Self::pda(program_id, authority);

        if pda != &pda_check {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(bump_seed)
    }

    pub fn verify_pda(
        program_id: &Pubkey,
        authority: &Pubkey,
        pda: &Pubkey,
    ) -> Result<(Pubkey, u8), ProgramError> {
        let (pda_check, bump_seed) = Self::pda(program_id, authority);

        if pda != &pda_check {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok((pda_check, bump_seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey as SolanaPubkey;

    #[test]
    fn test_seeds() {
        let program_id = SolanaPubkey::new_unique();
        let authority = SolanaPubkey::new_unique();

        let program_id_bytes = program_id.to_bytes();
        let authority_bytes = authority.to_bytes();

        let seeds = BlockList::seeds(&authority_bytes);
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0], BlockList::SEED_PREFIX);
        assert_eq!(seeds[1], authority_bytes.as_ref());

        let (pda, bump) = BlockList::pda(&program_id_bytes, &authority_bytes);

        let (solana_pda, solana_bump) = SolanaPubkey::find_program_address(
            &[BlockList::SEED_PREFIX, authority.as_ref()],
            &program_id,
        );

        assert_eq!(pda, solana_pda.to_bytes());
        assert_eq!(bump, solana_bump);

        let result = BlockList::check_pda(&program_id_bytes, &authority_bytes, &pda);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), bump);

        let result = BlockList::verify_pda(&program_id_bytes, &authority_bytes, &pda);
        assert!(result.is_ok());
        let (verified_pda, verified_bump) = result.unwrap();
        assert_eq!(verified_pda, pda);
        assert_eq!(verified_bump, bump);
    }

    #[test]
    fn test_signer_seeds() {
        let authority = SolanaPubkey::new_unique();
        let authority_bytes = authority.to_bytes();

        let seeds = BlockList::get_signer_seeds(&authority_bytes);
        assert_eq!(seeds.len(), 2);

        let signers = BlockList::get_signers(&seeds);
        assert_eq!(signers.len(), 1);
    }
}
