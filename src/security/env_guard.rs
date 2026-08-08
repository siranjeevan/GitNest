use crate::domain::error::Result;
use std::env;

const DANGEROUS_ENV_VARS: &[&str] = &[
    "GIT_SSH_COMMAND",
    "GIT_SSH",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COMMAND",
    "GIT_DIR",
    "GIT_WORK_TREE",
];

pub struct EnvGuard;

impl EnvGuard {
    /// Inspects and reports dangerous environment overrides that could bypass isolation
    pub fn inspect_environment() -> Vec<String> {
        let mut warnings = Vec::new();
        for var in DANGEROUS_ENV_VARS {
            if let Ok(val) = env::var(var) {
                if !val.is_empty() {
                    warnings.push(format!("Environment variable {} is set to '{}'", var, val));
                }
            }
        }
        warnings
    }

    /// Enforces clean environment for execution by overriding GIT_SSH_COMMAND safely
    pub fn validate_and_sanitize() -> Result<()> {
        let warnings = Self::inspect_environment();
        if !warnings.is_empty() {
            let msg = format!(
                "BLOCKED: Dangerous Environment Variable Override Detected!\n\
                GitNest detected external environment variables ({}) that could bypass account identity isolation.\n\
                Please unset these environment variables before executing GitNest commands.",
                warnings.join(", ")
            );
            return Err(crate::domain::error::GitNestError::EnvOverrideBlocked(msg));
        }
        Ok(())
    }
}
