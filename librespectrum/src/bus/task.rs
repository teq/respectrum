use std::{
    fmt,
    rc::Rc,
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

/// Task execution step
struct Step {
    pub htcycles: u64,
    pub task_idx: usize,
    pub next: Option<Box<Step>>,
}

impl fmt::Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.next {
            &Some(ref step) => write!(f, "({:?},{:?}),{:?}", self.htcycles, self.task_idx, *step),
            &None => write!(f, "({:?},{:?})", self.htcycles, self.task_idx),
        }
    }
}

impl Step {
    /// Schedule given task at given htcycles
    fn schedule(head: &mut Option<Box<Step>>, htcycles: u64, task_idx: usize) {
        match head {
            Some(step) if step.htcycles > htcycles => {
                *head = Some(Box::new(Step { htcycles, task_idx, next: head.take() }));
            },
            Some(step) => Step::schedule(&mut step.next, htcycles, task_idx),
            None => *head = Some(Box::new(Step { htcycles, task_idx, next: None }))
        }
    }
}

/// Clock-synced tasks scheduler
pub struct Scheduler<'a> {

    /// System clock
    clock: Rc<Clock>,

    /// Managed tasks
    tasks: Vec<Box<dyn NoReturnTask + 'a>>,

    /// Execution steps queue head
    head: Option<Box<Step>>,

}

impl fmt::Debug for Scheduler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "clock: {:?}, queue: {:?}", self.clock.get(), self.head)
    }
}

impl<'a> Scheduler<'a> {

    /// Create new scheduler instance
    pub fn new(clock: Rc<Clock>, tasks: Vec<Box<dyn NoReturnTask + 'a>>) -> Self {
        fn init(htcycles: u64, i: usize) -> Option<Box<Step>> {
            if i == 0 {None} else {
                Some(Box::new(Step { htcycles, task_idx: i-1, next: init(htcycles, i-1) }))
            }
        }
        let htcycles = clock.get();
        let tasks_len = tasks.len();
        Self { clock, tasks, head: init(htcycles, tasks_len) }
    }

    /// Advance N half t-cycles forward
    pub fn advance(&mut self, offset: u64) {

        // htcycles to advance to
        let target_htcycles = self.clock.get() + offset;

        // Check if next step is before target_htcycles
        if let Some(step) = &self.head && step.htcycles < target_htcycles {

            // Consume the step and set head to the next one
            let Step { htcycles: task_htcycles, task_idx, next } = *self.head.take().unwrap();
            let task = &mut self.tasks[task_idx];
            self.head = next;

            // Advance to task's htcycles and continue task execution
            self.clock.set(task_htcycles);
            if let GeneratorState::Yielded(offset) = Pin::new(task).resume(()) {
                // Re-schedule current task with returned htcycles offset
                Step::schedule(&mut self.head, task_htcycles + offset as u64, task_idx);
            } else {
                panic!("Expecting task to never return (complete)");
            }

            // Recursively advance to the next step
            let htcycles_left = target_htcycles - task_htcycles;
            if htcycles_left > 0 {
                self.advance(htcycles_left);
            }

        } else {

            // Just advance to target_htcycles
            self.clock.set(target_htcycles);

        }

    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use std::cell::RefCell;

    struct SharedState {
        clock: Rc<Clock>,
        seq: RefCell<Vec<(u64, bool)>>
    }

    struct Foo { state: Rc<SharedState> }
    impl Foo {
        fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {
            Box::new(move || {
                loop {
                    yield self.state.clock.rising(3); // skip to 3rd raising edge
                    self.state.seq.borrow_mut().push((self.state.clock.get(), true));
                }
            })
        }
    }

    struct Bar { state: Rc<SharedState> }
    impl Bar {
        fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {
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

        let clock: Rc<Clock> = Default::default();

        let state = Rc::new(SharedState {
            clock: Rc::clone(&clock),
            seq: RefCell::new(vec![])
        });

        let foo = Foo { state: Rc::clone(&state) };
        let bar = Bar { state: Rc::clone(&state) };

        let mut scheduler = Scheduler::new(clock, vec![foo.run(), bar.run()]);

        scheduler.advance(10);
        assert_eq!(
            *state.seq.borrow(),
            vec![(1, false), (3, false), (5, false), (6, true), (7, false), (9, false)]
        );

        state.seq.borrow_mut().clear();
        scheduler.advance(10);
        assert_eq!(
            *state.seq.borrow(),
            vec![(11, false), (12, true), (13, false), (15, false), (17, false), (18, true), (19, false)]
        );

    }

}
