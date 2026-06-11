use solana_pubkey::Pubkey;
use yellowstone_shield_client::{accounts, PolicyTrait};
pub use yellowstone_shield_client::{types::PermissionStrategy, ID};

#[derive(Debug, Clone)]
pub struct Policy {
    pub strategy: PermissionStrategy,
    pub identities: Vec<Pubkey>,
}

impl Policy {
    pub fn new(strategy: PermissionStrategy, identities: Vec<Pubkey>) -> Self {
        Self {
            strategy,
            identities,
        }
    }
}

/// Shield Program State
#[allow(clippy::large_enum_variant, dead_code)]
#[derive(Debug, Clone)]
pub enum ShieldProgramState {
    Policy(u64, Pubkey, Policy),
}

impl ShieldProgramState {
    pub fn try_parse_account_data(
        slot: u64,
        pubkey: Pubkey,
        owner: &Pubkey,
        data: &[u8],
        expected_program_id: Option<&Pubkey>,
    ) -> Result<Self, String> {
        let expected_id = expected_program_id.unwrap_or(&ID);
        if owner != expected_id {
            return Err(format!(
                "Invalid owner: expected {}, got {}",
                expected_id, owner
            ));
        }

        if data.is_empty() {
            return Err("Data is empty".to_owned());
        }

        let (strategy, identities) = match data[0] {
            0 => {
                let policy = accounts::Policy::from_bytes(data).map_err(|e| e.to_string())?;
                let strategy = policy.try_strategy().map_err(|e| e.to_string())?;
                let identities = accounts::Policy::try_deserialize_identities(data)
                    .map_err(|e| e.to_string())?;
                (strategy, identities)
            }
            1 => {
                let policy = accounts::PolicyV2::from_bytes(data).map_err(|e| e.to_string())?;
                let strategy = policy.try_strategy().map_err(|e| e.to_string())?;
                let identities = accounts::PolicyV2::try_deserialize_identities(data)
                    .map_err(|e| e.to_string())?;
                (strategy, identities)
            }
            _ => return Err("Unsupported data type".to_owned()),
        };

        let policy = Policy::new(strategy, identities);
        Ok(ShieldProgramState::Policy(slot, pubkey, policy))
    }
}

pub fn parse_account(
    slot: u64,
    pubkey: Pubkey,
    owner: &Pubkey,
    data: &[u8],
    expected_program_id: Option<&Pubkey>,
) -> Result<ShieldProgramState, String> {
    ShieldProgramState::try_parse_account_data(slot, pubkey, owner, data, expected_program_id)
}
