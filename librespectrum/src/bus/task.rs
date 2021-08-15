use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

use super::Clock;

/// Generator which returns a value when it's completed
/// or yields next wake up time as an offset in half t-cycles to the current clock
pub trait Task<T> = Generator<(), Yield=usize, Return=T> + Unpin;

/// Generator which never returns and yields next wake up time
/// as an offset in half t-cycles to the current clock
pub trait NoReturnTask = Task<!>;

/// Generic bus device
pub trait Device<'a> {

    /// Create task to run with the scheduler
    fn run(&'a self) -> Box<dyn NoReturnTask + 'a>;

}

/// Clock-synced tasks scheduler
pub struct Scheduler<'a> {

    /// System clock
    clock: &'a Clock,

    /// Managed tasks stored as tuples: (htcycles, task)
    tasks: Vec<(u64, Box<dyn NoReturnTask + 'a>)>,

}

impl<'a> Scheduler<'a> {

    /// Create new scheduler instance
    pub fn new(clock: &'a Clock) -> Self {
        Self { clock, tasks: Default::default() }
    }

    /// Add new device
    pub fn add(&mut self, device: &'a dyn Device<'a>) {
        let task = device.run();
        self.tasks.push((self.clock.get(), task));
    }

    /// Advance N half t-cycles forward
    pub fn advance(&mut self, n: u64) {

        let target_htcycles = self.clock.get() + n;

        while self.clock.get() < target_htcycles {

            // Get htcycles of the earliest task
            match self.tasks.iter().map(|t| t.0).min() {

                // We have such task(-s) and it's before target_htcycles
                Some(task_htcycles) if task_htcycles < target_htcycles => {

                    self.clock.set(task_htcycles);

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
                _ => self.clock.set(target_htcycles)

            }

        }

    }

}

#[cfg(test)]
mod tests {
    
    use super::*;
    use std::cell::RefCell;

    struct SharedState {
        clock: Clock,
        seq: RefCell<Vec<(u64, bool)>>
    }

    struct Foo<'a> { state: &'a SharedState }
    impl<'a> Device<'a> for Foo<'a> {
        fn run(&'a self) -> Box<dyn NoReturnTask + 'a> {
            Box::new(move || {
                loop {
                    yield self.state.clock.rising(3); // skip to 3rd raising edge
                    self.state.seq.borrow_mut().push((self.state.clock.get(), true));
                }
            })
        }
    }

    struct Bar<'a> { state: &'a SharedState }
    impl<'a> Device<'a> for Bar<'a> {
        fn run(&'a self) -> Box<dyn NoReturnTask + 'a> {
            Box::new(move || {
                loop {
                    yield self.state.clock.falling(1); // skip to 1st falling edge
                    self.state.seq.borrow_mut().push((self.state.clock.get(), false));
                }
            })
        }
    }

    #[test]
    fn scheduler_executes_tasks() {

        let state = SharedState {
            clock: Clock::new(),
            seq: RefCell::new(vec!())
        };

        let foo = Foo { state: &state };
        let bar = Bar { state: &state };

        let mut scheduler = Scheduler::new(&state.clock);
        scheduler.add(&foo);
        scheduler.add(&bar);

        scheduler.advance(10);
        assert_eq!(
            *state.seq.borrow(),
            vec!((1, false), (3, false), (5, false), (6, true), (7, false), (9, false))
        );

        state.seq.borrow_mut().clear();
        scheduler.advance(10);
        assert_eq!(
            *state.seq.borrow(),
            vec!((11, false), (12, true), (13, false), (15, false), (17, false), (18, true), (19, false))
        );

    }

}
