use std::{
    ops::{Coroutine, CoroutineState}, pin::Pin, rc::Rc
};

use super::Clock;

/// Task which returns a value when it's completed
/// or yields next wake up time as an offset in half t-cycles to the current clock
pub trait Task<T> = Coroutine<(), Yield=usize, Return=T> + Unpin;

/// Task which never returns
pub trait NoReturnTask = Task<!>;

/// Task execution time slot
struct TaskSlot {
    pub htcycles: u64,
    pub task_idx: usize,
    pub next: Option<Box<TaskSlot>>,
}

/// Clock-synced tasks scheduler
pub struct Scheduler<'a> {

    /// System clock
    clock: Rc<Clock>,

    /// Managed tasks
    tasks: Vec<Box<dyn NoReturnTask + 'a>>,

    /// Task queue head
    queue: Option<Box<TaskSlot>>,

}

impl<'a> Scheduler<'a> {

    /// Create new scheduler instance
    pub fn new(clock: Rc<Clock>, tasks: Vec<Box<dyn NoReturnTask + 'a>>) -> Self {
        let htcycles = clock.get();
        let mut queue = None;
        for task_idx in 0..tasks.len() {
            queue = Some(Box::new(TaskSlot { htcycles, task_idx, next: queue }));
        }
        Self { clock, tasks, queue }
    }

    /// Advance N half t-cycles forward
    pub fn run(&mut self, offset: u64) {

        // htcycles to advance to
        let target_htcycles = self.clock.get() + offset;

        loop {

            // Check if next step is before target_htcycles
            if let Some(step) = &self.queue && step.htcycles < target_htcycles {

                // Consume the step and set head to the next one
                let TaskSlot { htcycles: task_htcycles, task_idx, next } = *self.queue.take().unwrap();
                let task = &mut self.tasks[task_idx];
                self.queue = next;

                // Advance to task's htcycles and continue task execution
                self.clock.set(task_htcycles);
                let CoroutineState::Yielded(offset) = Pin::new(task).resume(());

                // Re-schedule current task with returned htcycles offset
                self.schedule(task_htcycles + offset as u64, task_idx);

            } else {

                // Just advance to target_htcycles
                self.clock.set(target_htcycles);
                break;

            }
        }

    }

    /// Schedule given task at given htcycles
    fn schedule(&mut self, htcycles: u64, task_idx: usize) {
        let mut cursor = &mut self.queue;
        while cursor.as_ref().is_some_and(|step| step.htcycles <= htcycles) {
            cursor = &mut cursor.as_mut().unwrap().next;
        }
        let next = cursor.take();
        *cursor = Some(Box::new(TaskSlot { htcycles, task_idx, next }));
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
            Box::new(#[coroutine] move || {
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
            Box::new(#[coroutine] move || {
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

        scheduler.run(10);
        assert_eq!(
            *state.seq.borrow(),
            vec![(1, false), (3, false), (5, false), (6, true), (7, false), (9, false)]
        );

        state.seq.borrow_mut().clear();
        scheduler.run(10);
        assert_eq!(
            *state.seq.borrow(),
            vec![(11, false), (12, true), (13, false), (15, false), (17, false), (18, true), (19, false)]
        );

    }

}
