use std::fmt;

#[derive(Debug, Clone)]
pub enum CertificateError {
    INVALID,
}

impl fmt::Display for CertificateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertificateError::INVALID => {
                write!(f, "Invalid EC public key input")
            }
        }
    }
}
