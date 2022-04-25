use crate::{bus::NoReturnTask, misc::Identifiable};

pub trait Device: Identifiable {
    fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a>;
}
