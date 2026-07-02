use crate::{BwatchError, FindingCategory};
use bsuite_core::{corpus::parse_signed_corpus, prompt_resolver::DirectiveString};
use ed25519_dalek::VerifyingKey;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct ExtendedCorpusFile {
    entries: Vec<ExtendedCorpusEntry>,
}

#[derive(Deserialize)]
struct ExtendedCorpusEntry {
    finding_category: String,
    directive: String,
}

#[derive(Debug)]
pub struct FindingCategoryCorpusIndex {
    by_category: HashMap<FindingCategory, DirectiveString>,
}

impl FindingCategoryCorpusIndex {
    pub fn from_toml_signed(corpus_toml: &str, pubkey: &VerifyingKey) -> Result<Self, BwatchError> {
        parse_signed_corpus(corpus_toml, pubkey).map_err(BwatchError::Core)?;
        let extended: ExtendedCorpusFile =
            toml::from_str(corpus_toml).map_err(|e| BwatchError::CorpusLoad(e.to_string()))?;
        Self::build_index(extended.entries)
    }

    fn build_index(entries: Vec<ExtendedCorpusEntry>) -> Result<Self, BwatchError> {
        let mut by_category = HashMap::with_capacity(FindingCategory::ALL.len());

        for entry in entries {
            let category = entry
                .finding_category
                .parse::<FindingCategory>()
                .map_err(|_| {
                    BwatchError::CorpusLoad(format!(
                        "unrecognised finding_category in corpus: {}",
                        entry.finding_category
                    ))
                })?;

            if by_category.contains_key(&category) {
                return Err(BwatchError::CorpusLoad(format!(
                    "duplicate finding_category in corpus: {}",
                    entry.finding_category
                )));
            }

            by_category.insert(category, DirectiveString::new(entry.directive));
        }

        for category in FindingCategory::ALL {
            if !by_category.contains_key(&category) {
                return Err(BwatchError::CorpusLoad(format!(
                    "corpus missing entry for finding_category: {}",
                    category.stable_name()
                )));
            }
        }

        Ok(Self { by_category })
    }

    pub fn resolve(&self, category: FindingCategory) -> &DirectiveString {
        self.by_category.get(&category).expect(
            "FindingCategoryCorpusIndex invariant: every FindingCategory variant indexed at construction",
        )
    }
}
