#[macro_export]
macro_rules! extract_mandatory_annotation_from {
    ($annotation: literal, $map: expr) => {
        $map.get($annotation)
            .ok_or_else(|| anyhow::anyhow!("Missing {} in the manifest file", $annotation))?
            .map_err(|err| {
                anyhow::anyhow!(
                    "Error parsing {} in the manifest file. Details: {err}",
                    $annotation
                )
            })?
    };
}

#[macro_export]
macro_rules! extract_optional_string_annotation_from {
    ($annotation: literal, $map: expr) => {
        if let Some(literal) = $map.get($annotation) {
            let literal = literal.map_err(|err| {
                anyhow::anyhow!(
                    "Error parsing {} in the manifest file. Details: {err}",
                    $annotation
                )
            })?;
            Some(
                literal
                    .first()
                    .ok_or_else(|| anyhow!("Missing {}", $annotation))?
                    .to_string(),
            )
        } else {
            None
        }
    };
}

pub use extract_mandatory_annotation_from;
pub use extract_optional_string_annotation_from;
