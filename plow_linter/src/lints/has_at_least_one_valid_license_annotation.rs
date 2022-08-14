use std::any::Any;

use crate::{
    lint::{lint_failure, lint_success, Lint, LintResult},
    Linter,
};

use super::{
    ExistsRegistryLicense, ExistsRegistryLicenseSPDX, HasRegistryLicense, HasRegistryLicenseSPDX,
};

#[derive(Debug, Default)]
pub struct HasAtLeastOneValidLicenseAnnotation;

impl Lint for HasAtLeastOneValidLicenseAnnotation {
    fn can_run_in_parallel(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:license` or `registry:licenseSPDX`, both missing are not allowed."
    }

    // TODO: Refactor and finish this function later
    #[allow(clippy::unwrap_used)]
    fn run(&self, linter: &Linter) -> LintResult {
        if let Some(ref sub_lints) = linter.sub_lints {
            let exists_registry_license_lint = sub_lints
                .iter()
                .find_map(|lint| {
                    if let Some(lint) = lint.as_any().downcast_ref::<ExistsRegistryLicense>() {
                        return Some(lint);
                    }
                    None
                })
                .unwrap();
            let exists_registry_license_spdx_lint = sub_lints
                .iter()
                .find_map(|lint| {
                    if let Some(lint) = lint.as_any().downcast_ref::<ExistsRegistryLicenseSPDX>() {
                        return Some(lint);
                    }
                    None
                })
                .unwrap();
            let has_registry_license_lint = sub_lints
                .iter()
                .find_map(|lint| {
                    if let Some(lint) = lint.as_any().downcast_ref::<HasRegistryLicense>() {
                        return Some(lint);
                    }
                    None
                })
                .unwrap();
            let has_registry_license_spdx_lint = sub_lints
                .iter()
                .find_map(|lint| {
                    if let Some(lint) = lint.as_any().downcast_ref::<HasRegistryLicenseSPDX>() {
                        return Some(lint);
                    }
                    None
                })
                .unwrap();

            let license_exists = exists_registry_license_lint.run(linter).is_success();
            let license_spdx_exists = exists_registry_license_spdx_lint.run(linter).is_success();

            if license_spdx_exists && license_exists {
                let license_validity = has_registry_license_lint.run(linter);
                let license_spdx_validity = has_registry_license_spdx_lint.run(linter);
                if license_validity.is_failure() {
                    return license_validity;
                }
                if license_spdx_validity.is_failure() {
                    return license_spdx_validity;
                }
                lint_success!(
                    "Both `registry:license` and `registry:licenseSPDX` are present and valid."
                )
            } else if license_spdx_exists {
                has_registry_license_spdx_lint.run(linter)
            } else if license_exists {
                has_registry_license_lint.run(linter)
            } else {
                lint_failure!("Both `registry:license` and `registry:licenseSPDX` are missing. At least one of these annotations needs to be present.")
            }
        } else {
            lint_failure!("Sub lints needs to be provided to run this lint. Lint could not be run.")
        }
    }
}
