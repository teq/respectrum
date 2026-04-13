/// Represents identifiable object
pub trait Identifiable {

    /// Get object ID
    fn id(&self) -> usize;

}

impl Identifiable for usize {
    fn id(&self) -> usize {
        *self
    }
}
