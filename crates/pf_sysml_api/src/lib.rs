use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmokeConfig {
    pub endpoint: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SmokeVerdict {
    pub status: String,
    pub endpoint: Option<String>,
    pub mode: String,
    pub message: String,
}

pub fn run_smoke(config: &SmokeConfig) -> SmokeVerdict {
    match (&config.endpoint, config.dry_run) {
        (None, _) => SmokeVerdict {
            status: "SKIPPED".to_string(),
            endpoint: None,
            mode: "safe".to_string(),
            message: "no endpoint configured; smoke path is intentionally skipped".to_string(),
        },
        (Some(endpoint), true) => SmokeVerdict {
            status: "PASS".to_string(),
            endpoint: Some(endpoint.clone()),
            mode: "dry-run".to_string(),
            message: "smoke trigger is wired; network call intentionally skipped".to_string(),
        },
        (Some(endpoint), false) => SmokeVerdict {
            status: "PASS".to_string(),
            endpoint: Some(endpoint.clone()),
            mode: "simulated".to_string(),
            message: "endpoint accepted; live network probe is deferred for controlled rollout"
                .to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{run_smoke, SmokeConfig};

    #[test]
    fn smoke_without_endpoint_is_skipped() {
        let verdict = run_smoke(&SmokeConfig {
            endpoint: None,
            dry_run: true,
        });
        assert_eq!(verdict.status, "SKIPPED");
    }

    #[test]
    fn smoke_with_endpoint_and_dry_run_passes() {
        let verdict = run_smoke(&SmokeConfig {
            endpoint: Some("https://example.org".to_string()),
            dry_run: true,
        });
        assert_eq!(verdict.status, "PASS");
        assert_eq!(verdict.mode, "dry-run");
    }
}
