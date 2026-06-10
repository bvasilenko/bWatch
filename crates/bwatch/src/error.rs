use thiserror::Error;

#[derive(Debug, Error)]
pub enum BwatchError {
    #[error("source input is malformed: {0}")]
    SourceMalformed(String),
    #[error("mission reference is invalid: {0}")]
    MissionMalformed(String),
    #[error("unknown finding category: {0}")]
    TaxonomyUnknown(String),
    #[error("unknown outward source state: {0}")]
    OutwardStateUnknown(String),
    #[error("unknown outward source substrate: {0}")]
    SubstrateUnknown(String),
    #[error("argument usage is invalid: {0}")]
    Usage(String),
    #[error(transparent)]
    Core(#[from] bsuite_core::BsuiteCoreError),
}

impl BwatchError {
    pub const fn exit_code(&self) -> bsuite_core::ExitCode {
        match self {
            Self::Usage(_) => bsuite_core::ExitCode::Usage,
            Self::SourceMalformed(_)
            | Self::MissionMalformed(_)
            | Self::TaxonomyUnknown(_)
            | Self::OutwardStateUnknown(_)
            | Self::SubstrateUnknown(_)
            | Self::Core(_) => bsuite_core::ExitCode::InternalError,
        }
    }

    pub fn process_exit_code(&self) -> std::process::ExitCode {
        process_exit_code(self.exit_code())
    }
}

pub fn process_exit_code(code: bsuite_core::ExitCode) -> std::process::ExitCode {
    std::process::ExitCode::from(code.as_i32() as u8)
}
