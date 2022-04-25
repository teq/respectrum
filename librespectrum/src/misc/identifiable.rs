
pub trait Identifiable {
    fn id(&self) -> &'static str;
}
