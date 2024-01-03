pub struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self(msg.to_owned())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        format!("JSON Error: {e}").into()
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        format!("IO Error: {e}").into()
    }
}

impl From<bollard::errors::Error> for Error {
    fn from(e: bollard::errors::Error) -> Self {
        format!("Docker Error: {e}").into()
    }
}
