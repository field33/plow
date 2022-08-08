use crate::lint::{lint_failure, lint_success, Lint, LintResult};
use harriet::TurtleDocument;

use super::{
    ExistsRegistryLicense, ExistsRegistryLicenseSPDX, HasRegistryLicense, HasRegistryLicenseSPDX,
};

#[derive(Debug, Default)]
pub struct HasAtLeastOneValidLicenseAnnotation;

impl Lint for HasAtLeastOneValidLicenseAnnotation {
    fn short_description(&self) -> &str {
        "Check that the field is annotated with a value for `registry:license` or `registry:licenseSPDX`, both missing are not allowed."
    }

    fn lint(&self, document: &TurtleDocument) -> LintResult {
        let license_exists = ExistsRegistryLicense::default().lint(document).is_success();
        let license_spdx_exists = ExistsRegistryLicenseSPDX::default()
            .lint(document)
            .is_success();

        if license_spdx_exists && license_exists {
            let license_validity = HasRegistryLicense::default().lint(document);
            let license_spdx_validity = HasRegistryLicenseSPDX::default().lint(document);
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
            HasRegistryLicenseSPDX::default().lint(document)
        } else if license_exists {
            HasRegistryLicense::default().lint(document)
        } else {
            lint_failure!("Both `registry:license` and `registry:licenseSPDX` are missing. At least one of these annotations needs to be present.")
        }
    }
}
