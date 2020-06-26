use std::{
    rc::Rc,
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use super::Clock;

#[macro_export]
macro_rules! yield_task {
    ($input: expr) => {
        {
            let mut task = $input;
            loop {
                match std::pin::Pin::new(&mut task).resume(()) {
                    std::ops::GeneratorState::Yielded(some) => yield some,
                    std::ops::GeneratorState::Complete(result) => break result
                }
            }
        }
    }
}

/// Generator which returns a value when it's completed
/// or yields next wake up time as an offset in half t-cycles to the current clock
pub trait Task<T> = Generator<(), Yield=usize, Return=T> + Unpin;

/// Generator which never returns and yields next wake up time
/// as an offset in half t-cycles to the current clock
pub trait NoReturnTask = Task<!>;

/// Clock-synced tasks scheduler
pub struct Scheduler {

    /// System clock
    clock: Rc<Clock>,

    /// Managed tasks stored as tuples: (<next wake up htcycles>, <task>)
    tasks: Vec<(u64, Box<dyn NoReturnTask>)>,

}

impl Scheduler {

    /// Create new scheduler instance
    pub fn new(clock: Rc<Clock>) -> Scheduler {
        Scheduler { clock, tasks: Default::default() }
    }

    /// Add new task
    pub fn add(&mut self, task: Box<dyn NoReturnTask>) {
        self.tasks.push((self.clock.htcycles(), task));
    }

    /// Advance N half t-cycles forward
    pub fn advance(&mut self, n: u64) {

        let target_htcycles = self.clock.htcycles() + n;

        while self.clock.htcycles() < target_htcycles {

            // Get htcycles of the earliest task
            match self.tasks.iter().map(|t| t.0).min() {

                // We have such task(-s) and it's before target_htcycles
                Some(task_htcycles) if task_htcycles < target_htcycles => {

                    self.clock.htcycles.set(task_htcycles);

                    // Run each task and store htcycles for the next wake up
                    for tuple in self.tasks.iter_mut().filter(|t| t.0 == task_htcycles) {
                        let (ref mut next_wakeup, ref mut task) = tuple;
                        if let GeneratorState::Yielded(offset) = Pin::new(task).resume(()) {
                            *next_wakeup = task_htcycles + offset as u64;
                        } else {
                            panic!("Expecting task to never return (complete)");
                        }
                    }

                },

                // No such task(-s) => just advance to target_htcycles
                _ => self.clock.htcycles.set(target_htcycles)

            }

        }

    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct TestState {
        clock: Rc<Clock>,
        seq: Rc<RefCell<Vec<(u64, String)>>>,
    }

    #[test]
    fn scheduler_executes_tasks() {

        let state: Rc<TestState> = Default::default();
        let mut scheduler = Scheduler::new(Rc::clone(&state.clock));

        scheduler.add(Box::new({ let state = Rc::clone(&state); move || {
            loop {
                yield state.clock.rising(3); // skip to 3rd raising edge
                state.seq.borrow_mut().push((state.clock.htcycles(), String::from("rise")));
            }
        }}));

        scheduler.add(Box::new({ let state = Rc::clone(&state); move || {
            loop {
                yield state.clock.falling(1); // skip to 1st falling edge
                state.seq.borrow_mut().push((state.clock.htcycles(), String::from("fall")));
            }
        }}));

        scheduler.advance(10);
        assert_eq!(
            format!("{:?}", state.seq),
            r#"RefCell { value: [(1, "fall"), (3, "fall"), (5, "fall"), (6, "rise"), (7, "fall"), (9, "fall")] }"#
        );

        state.seq.borrow_mut().clear();
        scheduler.advance(10);
        assert_eq!(
            format!("{:?}", state.seq),
            r#"RefCell { value: [(11, "fall"), (12, "rise"), (13, "fall"), (15, "fall"), (17, "fall"), (18, "rise"), (19, "fall")] }"#
        );

    }

}
