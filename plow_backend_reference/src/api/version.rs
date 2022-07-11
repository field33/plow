/// An enum for API versions
pub enum Version {
    V1,
}

impl Version {
    pub const fn as_path(&self) -> &'static str {
        match self {
            Self::V1 => "/v1",
        }
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        match self {
            Self::V1 => "v1".to_owned(),
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "v1" => Ok(Self::V1),
            _ => Err(format!("Unknown API: {}", s)),
        }
    }
}
