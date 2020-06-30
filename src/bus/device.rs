use super::task::NoReturnTask;

/// Generic bus device
pub trait Device {

    /// Create task to run on scheduler
    fn run<'a>(&'a mut self) -> Box<dyn NoReturnTask + 'a>;

}
