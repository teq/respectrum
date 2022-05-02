/// Represents identifiable object
pub trait Identifiable {

    /// Get object ID
    fn id(&self) -> u32;

}
