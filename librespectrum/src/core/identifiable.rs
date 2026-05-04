pub type Identifier = usize;

/// Represents identifiable object
pub trait Identifiable {

    /// Get object ID
    fn id(&self) -> Identifier;

}

impl Identifiable for usize {
    fn id(&self) -> Identifier {
        *self
    }
}
