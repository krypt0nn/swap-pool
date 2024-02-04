pub type SwapResult<T> = Result<T, SwapError>;

#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum SwapError {
    #[cfg_attr(feature = "thiserror", error(transparent))]
    Io(#[cfg_attr(feature = "thiserror", from)] std::io::Error),

    #[cfg_attr(feature = "thiserror", error("Failed to serialize value to bytes: {0}"))]
    Serialize(#[cfg_attr(feature = "thiserror", source)] Box<dyn std::error::Error + 'static>),

    #[cfg_attr(feature = "thiserror", error("Failed to deserialize value from bytes: {0}"))]
    Deserialize(#[cfg_attr(feature = "thiserror", source)] Box<dyn std::error::Error + 'static>),

    #[cfg_attr(feature = "thiserror", error("Failed to transform value forward: {0}"))]
    TransformForward(#[cfg_attr(feature = "thiserror", source)] Box<dyn std::error::Error + 'static>),

    #[cfg_attr(feature = "thiserror", error("Failed to transform value backward: {0}"))]
    TransformBackward(#[cfg_attr(feature = "thiserror", source)] Box<dyn std::error::Error + 'static>)
}

#[cfg(not(feature = "thiserror"))]
impl std::fmt::Display for SwapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serialize(error) => write!(f, "Failed to serialize value to bytes: {error}"),
            Self::Deserialize(error) => write!(f, "Failed to deserialize value from bytes: {error}"),
            Self::TransformForward(error) => write!(f, "Failed to transform value forward: {error}"),
            Self::TransformBackward(error) => write!(f, "Failed to transform value backward: {error}")
        }
    }
}

#[cfg(not(feature = "thiserror"))]
impl From<std::io::Error> for SwapError {
    #[inline]
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(not(feature = "thiserror"))]
impl std::error::Error for SwapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => error.source(),

            Self::Serialize(error) |
            Self::Deserialize(error) |
            Self::TransformForward(error) |
            Self::TransformBackward(error) => error.source()
        }
    }
}
