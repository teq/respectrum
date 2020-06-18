use super::clock::Task;

/// Generic bus device
pub trait Device {

    /// Create task to run on scheduler
    fn run<'a>(&'a self) -> Box<dyn Task + 'a>;

}
