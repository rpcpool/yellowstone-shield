use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStrategy {
    Deny,
    Allow,
}

// Convert from program's PermissionStrategy
impl From<yellowstone_shield::state::PermissionStrategy> for PermissionStrategy {
    fn from(s: yellowstone_shield::state::PermissionStrategy) -> Self {
        match s {
            yellowstone_shield::state::PermissionStrategy::Deny => PermissionStrategy::Deny,
            yellowstone_shield::state::PermissionStrategy::Allow => PermissionStrategy::Allow,
        }
    }
}

// Convert to program's PermissionStrategy
impl From<PermissionStrategy> for yellowstone_shield::state::PermissionStrategy {
    fn from(s: PermissionStrategy) -> Self {
        match s {
            PermissionStrategy::Deny => yellowstone_shield::state::PermissionStrategy::Deny,
            PermissionStrategy::Allow => yellowstone_shield::state::PermissionStrategy::Allow,
        }
    }
}

// Program ID
pub const ID: Pubkey = solana_program::pubkey!("b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W");

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Program error: {0}")]
    ProgramError(#[from] solana_program::program_error::ProgramError),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    Custom(String),
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IoError(e.to_string())
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Custom(s)
    }
}

impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError::Custom(s.to_owned())
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

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

const POLICY_V1_HEADER_SIZE: usize = 7; // kind(1) + strategy(1) + nonce(1) + identities_len(4)
const POLICY_V2_HEADER_SIZE: usize = 39; // kind(1) + strategy(1) + nonce(1) + mint(32) + identities_len(4)
const PUBKEY_SIZE: usize = 32;

impl ShieldProgramState {
    fn parse_policy_v1(data: &[u8]) -> ParseResult<Policy> {
        if data.len() < POLICY_V1_HEADER_SIZE {
            return Err(ParseError::from("Data too short for PolicyV1"));
        }

        let strategy: yellowstone_shield::state::PermissionStrategy = match data[1] {
            0 => yellowstone_shield::state::PermissionStrategy::Deny,
            1 => yellowstone_shield::state::PermissionStrategy::Allow,
            _ => return Err(ParseError::from("Invalid strategy")),
        };
        let strategy = PermissionStrategy::from(strategy);

        let identities_len_bytes: [u8; 4] = data[3..7]
            .try_into()
            .map_err(|_| ParseError::from("Failed to read identities length"))?;
        let identities_len = u32::from_le_bytes(identities_len_bytes) as usize;

        // Parse identities from the buffer after the header
        let identities_start = POLICY_V1_HEADER_SIZE;
        let identities_end = identities_start + (identities_len * PUBKEY_SIZE);

        if data.len() < identities_end {
            return Err(ParseError::from("Data too short for identities"));
        }

        let mut identities = Vec::with_capacity(identities_len);
        for i in 0..identities_len {
            let start = identities_start + (i * PUBKEY_SIZE);
            let end = start + PUBKEY_SIZE;
            let pubkey_bytes: [u8; 32] = data[start..end]
                .try_into()
                .map_err(|_| ParseError::from("Failed to read identity pubkey"))?;
            identities.push(Pubkey::new_from_array(pubkey_bytes));
        }

        Ok(Policy {
            strategy,
            identities,
        })
    }

    fn parse_policy_v2(data: &[u8]) -> ParseResult<Policy> {
        if data.len() < POLICY_V2_HEADER_SIZE {
            return Err(ParseError::from("Data too short for PolicyV2"));
        }

        let strategy: yellowstone_shield::state::PermissionStrategy = match data[1] {
            0 => yellowstone_shield::state::PermissionStrategy::Deny,
            1 => yellowstone_shield::state::PermissionStrategy::Allow,
            _ => return Err(ParseError::from("Invalid strategy")),
        };
        let strategy = PermissionStrategy::from(strategy);

        let identities_len_bytes: [u8; 4] = data[35..39]
            .try_into()
            .map_err(|_| ParseError::from("Failed to read identities length"))?;
        let identities_len = u32::from_le_bytes(identities_len_bytes) as usize;

        // Parse identities from the buffer after the header
        let identities_start = POLICY_V2_HEADER_SIZE;
        let identities_end = identities_start + (identities_len * PUBKEY_SIZE);

        if data.len() < identities_end {
            return Err(ParseError::from("Data too short for identities"));
        }

        let mut identities = Vec::with_capacity(identities_len);
        for i in 0..identities_len {
            let start = identities_start + (i * PUBKEY_SIZE);
            let end = start + PUBKEY_SIZE;
            let pubkey_bytes: [u8; 32] = data[start..end]
                .try_into()
                .map_err(|_| ParseError::from("Failed to read identity pubkey"))?;
            identities.push(Pubkey::new_from_array(pubkey_bytes));
        }

        Ok(Policy {
            strategy,
            identities,
        })
    }

    /// Parse account data from raw bytes
    pub fn try_parse(slot: u64, pubkey: Pubkey, owner: &Pubkey, data: &[u8], expected_program_id: Option<&Pubkey>) -> ParseResult<Self> {
        // Verify owner is the Shield program
        let expected_id = expected_program_id.unwrap_or(&ID);
        if owner != expected_id {
            return Err(ParseError::Custom(format!(
                "Invalid owner: expected {}, got {}",
                expected_id, owner
            )));
        }

        if data.is_empty() {
            return Err(ParseError::from("Data is empty"));
        }

        let policy = match data[0] {
            0 => Self::parse_policy_v1(data)?,
            1 => Self::parse_policy_v2(data)?,
            _ => {
                return Err(ParseError::from("Unsupported policy kind"));
            }
        };

        Ok(ShieldProgramState::Policy(slot, pubkey, policy))
    }
}

/// Parse a Shield program account from raw account data
pub fn parse_account(
    slot: u64,
    pubkey: Pubkey,
    owner: &Pubkey,
    data: &[u8],
    expected_program_id: Option<&Pubkey>,
) -> ParseResult<ShieldProgramState> {
    ShieldProgramState::try_parse(slot, pubkey, owner, data, expected_program_id)
}

/// Get the Shield program ID
pub fn program_id() -> Pubkey {
    ID
}
