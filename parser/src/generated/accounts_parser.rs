use solana_program::pubkey::Pubkey;
use yellowstone_shield_client::ID;
use yellowstone_shield_client::{accounts, types::PermissionStrategy, PolicyTrait};
use yellowstone_vixen_core::AccountUpdate;

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
    fn parse_policy<T: PolicyTrait>(data: &[u8]) -> yellowstone_vixen_core::ParseResult<Policy> {
        let policy = T::from_bytes(data)?;
        let identities = accounts::Policy::try_deserialize_identities(data)?;
        let strategy = policy.try_strategy()?;

        Ok(Policy {
            strategy,
            identities,
        })
    }

    pub fn try_unpack(account_update: &AccountUpdate) -> yellowstone_vixen_core::ParseResult<Self> {
        let inner = account_update
            .account
            .as_ref()
            .ok_or(solana_program::program_error::ProgramError::InvalidArgument)?;
        let data = inner.data.as_slice();

        if data.is_empty() {
            return Err(yellowstone_vixen_core::ParseError::from(
                "Data is empty".to_owned(),
            ));
        }

        let policy = match data[0] {
            0 => Self::parse_policy::<accounts::Policy>(data)?,
            1 => Self::parse_policy::<accounts::PolicyV2>(data)?,
            _ => {
                return Err(yellowstone_vixen_core::ParseError::from(
                    "Unsupported data type".to_owned(),
                ))
            }
        };

        Ok(ShieldProgramState::Policy(
            account_update.slot,
            Pubkey::try_from(inner.pubkey.as_slice())?,
            policy,
        ))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AccountParser;

impl yellowstone_vixen_core::Parser for AccountParser {
    type Input = yellowstone_vixen_core::AccountUpdate;
    type Output = ShieldProgramState;

    fn id(&self) -> std::borrow::Cow<str> {
        "shield::AccountParser".into()
    }

    fn prefilter(&self) -> yellowstone_vixen_core::Prefilter {
        yellowstone_vixen_core::Prefilter::builder()
            .account_owners([ID])
            .build()
            .unwrap()
    }

    async fn parse(
        &self,
        acct: &yellowstone_vixen_core::AccountUpdate,
    ) -> yellowstone_vixen_core::ParseResult<Self::Output> {
        ShieldProgramState::try_unpack(acct)
    }
}

impl yellowstone_vixen_core::ProgramParser for AccountParser {
    #[inline]
    fn program_id(&self) -> yellowstone_vixen_core::Pubkey {
        ID.to_bytes().into()
    }
}
