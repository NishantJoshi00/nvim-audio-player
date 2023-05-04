#[derive(thiserror::Error, Debug)]
pub enum OxiMorons {
    // #[error("This error isn't possible")]
    // Infallible,
    #[error("Error: {0}")]
    ComError(&'static str)
}
