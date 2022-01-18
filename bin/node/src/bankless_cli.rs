use bankless_primitives::DEFAULT_UNIT_CREATION_DELAY;
use finality_bankless::UnitCreationDelay;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
pub struct BanklessCli {
    #[structopt(long)]
    pub unit_creation_delay: Option<u64>,
}

impl BanklessCli {
    pub fn unit_creation_delay(&self) -> UnitCreationDelay {
        UnitCreationDelay(
            self.unit_creation_delay
                .unwrap_or(DEFAULT_UNIT_CREATION_DELAY),
        )
    }
}
