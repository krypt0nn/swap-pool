/// Transformers are needed to mutate entities values
/// before/after saving them to the swap files
/// 
/// You can use transformers to implement swap files compression
/// or any other operation
pub trait SwapTransformer {
    /// Mutate entity value before saving it to the swap file
    fn forward(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Mutate swap file value before loading it to the entity
    fn backward(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

pub struct SwapIdentityTransformer;

impl SwapTransformer for SwapIdentityTransformer {
    #[inline]
    fn forward(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(data)
    }

    #[inline]
    fn backward(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(data)
    }
}
