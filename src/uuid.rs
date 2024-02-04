use std::hash::{Hash, Hasher};

#[cfg_attr(feature = "random-uuid", allow(unused_variables))]
/// Generate random number for the given value
pub fn get(value: impl Hash) -> u64 {
    #[cfg(all(not(feature = "xxhash-uuid"), not(feature = "crc32-uuid")))]
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    #[cfg(all(feature = "crc32-uuid", not(feature = "xxhash-uuid")))]
    let mut hasher = crc32fast::Hasher::new();

    #[cfg(feature = "xxhash-uuid")]
    // Prioritize xxhash over crc32 because it's faster
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();

    #[cfg(all(feature = "timestamp-uuid", not(feature = "random-uuid")))]
    // Don't use system time for generating UUIDs when random-uuid is enabled
    // because it's better anyway and we don't need another level of randomness
    std::time::SystemTime::now().hash(&mut hasher);

    #[cfg(feature = "random-uuid")]
    rand::random::<u128>().hash(&mut hasher);

    #[cfg(not(feature = "random-uuid"))]
    // Don't feed the value into the hasher if we use random numbers generator
    // This is a good performance optimization because this library is planned
    // to be used with *large* values
    value.hash(&mut hasher);

    hasher.finish()
}
