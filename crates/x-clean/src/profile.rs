use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    #[default]
    Focused,
    Exploratory,
}

const FOCUSED: &[&str] = &[
    "unformatted-file",
    "exact-function-body",
    "duplicate-statement-window",
    "duplicate-literal-table",
    "collect-length-only",
    "regex-in-loop",
];

impl Profile {
    pub fn includes(self, rule: &str) -> bool {
        matches!(self, Self::Exploratory) || FOCUSED.contains(&rule)
    }
}

pub(crate) fn enabled(config: &crate::Config, rule: &str) -> bool {
    !config
        .disabled_rules
        .iter()
        .any(|identifier| identifier == rule)
        && if config.only_rules.is_empty() {
            config.profile.includes(rule)
        } else {
            config
                .only_rules
                .iter()
                .any(|identifier| identifier == rule)
        }
}
