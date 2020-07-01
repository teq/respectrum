use super::task::NoReturnTask;

/// Generic bus device
pub trait Device<'a> {

    /// Create task to run on scheduler
    fn run(&'a self) -> Box<dyn NoReturnTask + 'a>;

}
