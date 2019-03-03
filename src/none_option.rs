#[derive(Debug, Clone)]
pub struct NoneOption;

impl std::fmt::Display for NoneOption {
     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "option returned none")
    }
}

impl std::error::Error for NoneOption {}
