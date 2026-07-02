use bsuite_core::{
    BinaryDefaults, BsuiteCoreError, FileSystemManifestOverlayReader, FileSystemTranscriptAppender,
    FullAdapterHostBinder, HostContext, HostInvocationContext, ManifestOverlayReader,
};
use bwatch::corpus_index::FindingCategoryCorpusIndex;
use std::path::{Path, PathBuf};

const CORPUS_TOML: &str = include_str!("../corpus/bwatch-v0.toml");
const PUBKEY_BYTES: &[u8] = include_bytes!("../corpus/bwatch-v0-pubkey.bin");

pub const EMBEDDED_CORPUS_VERSION: u32 = 1;

pub struct BinaryRuntime {
    pub corpus: FindingCategoryCorpusIndex,
    pub appender: FileSystemTranscriptAppender,
    pub host_context: HostContext,
    pub invocation_context: Option<HostInvocationContext>,
    pub install_dir: PathBuf,
    pub corpus_version: u32,
}

impl BinaryRuntime {
    pub fn init(install_dir: PathBuf) -> Result<Self, BsuiteCoreError> {
        let corpus = load_corpus()?;
        let defaults = load_defaults(&install_dir)?;
        let appender = FileSystemTranscriptAppender::from_base_dir(
            defaults.transcript_dir,
            defaults.transcript_retention_days,
        );
        let host_binder = FullAdapterHostBinder::from_env()?;
        Ok(Self {
            corpus,
            appender,
            host_context: host_binder.resolved_host_context(),
            invocation_context: host_binder.invocation_context().cloned(),
            install_dir,
            corpus_version: EMBEDDED_CORPUS_VERSION,
        })
    }
}

fn load_corpus() -> Result<FindingCategoryCorpusIndex, BsuiteCoreError> {
    let pubkey = load_pubkey()?;
    verify_and_build_corpus(&pubkey)
}

fn verify_and_build_corpus(
    pubkey: &ed25519_dalek::VerifyingKey,
) -> Result<FindingCategoryCorpusIndex, BsuiteCoreError> {
    FindingCategoryCorpusIndex::from_toml_signed(CORPUS_TOML, pubkey)
        .map_err(|e| BsuiteCoreError::CorpusDeserializationFailed(e.to_string()))
}

fn load_pubkey() -> Result<ed25519_dalek::VerifyingKey, BsuiteCoreError> {
    let bytes: [u8; 32] = PUBKEY_BYTES.try_into().map_err(|_| {
        BsuiteCoreError::CorpusDeserializationFailed("embedded pubkey is not 32 bytes".to_owned())
    })?;
    ed25519_dalek::VerifyingKey::from_bytes(&bytes)
        .map_err(|e| BsuiteCoreError::CorpusDeserializationFailed(e.to_string()))
}

fn load_defaults(install_dir: &Path) -> Result<BinaryDefaults, BsuiteCoreError> {
    let base_dir = FileSystemTranscriptAppender::new("bwatch")?
        .directory()
        .to_path_buf();
    let overlay_reader = FileSystemManifestOverlayReader::new("bwatch", install_dir);
    let overlay = overlay_reader.read()?;
    let mut defaults = BinaryDefaults {
        transcript_retention_days: env_transcript_retention_days(),
        transcript_dir: base_dir,
        corpus_dir: install_dir.to_path_buf(),
        update_check_interval_minutes: 60,
        stdout_byte_cap: 65536,
        binary_timeout_ms: 5000,
    };
    overlay.merge_into_defaults(&mut defaults);
    Ok(defaults)
}

fn env_transcript_retention_days() -> u32 {
    std::env::var("BSUITE_TRANSCRIPT_RETENTION_DAYS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    #[test]
    fn embedded_corpus_loads_successfully_with_bundled_pubkey() {
        assert!(
            load_corpus().is_ok(),
            "embedded corpus must verify and load successfully against the bundled public key"
        );
    }

    #[test]
    fn corpus_load_fails_with_deserialization_error_when_pubkey_does_not_match() {
        let wrong_seed = [0xAB_u8; 32];
        let wrong_pubkey = SigningKey::from_bytes(&wrong_seed).verifying_key();
        let err =
            verify_and_build_corpus(&wrong_pubkey).expect_err("wrong pubkey must be rejected");
        assert!(
            matches!(err, BsuiteCoreError::CorpusDeserializationFailed(_)),
            "pubkey mismatch must surface as CorpusDeserializationFailed, got: {err:?}"
        );
    }
}
