use std::{
    ops::{Coroutine, CoroutineState}, pin::Pin, rc::Rc
};

use super::Clock;

/// Task yield values
pub enum TaskYield {
    Wait(u64),
    Break
}

/// Task which returns a value when it's completed
/// or yields next wake up time as an offset in half t-cycles to the current clock
pub trait Task<T> = Coroutine<(), Yield=TaskYield, Return=T> + Unpin;

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
    pub fn new(clock: &Rc<Clock>, tasks: Vec<Box<dyn NoReturnTask + 'a>>) -> Self {
        let htcycles = clock.get();
        let mut queue = None;
        for task_idx in 0..tasks.len() {
            queue = Some(Box::new(TaskSlot { htcycles, task_idx, next: queue }));
        }
        Self { clock: Rc::clone(clock), tasks, queue }
    }

    /// Run the scheduler for given htcycles or until break condition in any task
    /// Returns true if the scheduler ran until the target htcycles, false if a task triggered a break condition
    pub fn run(&mut self, htcycles: u64) -> bool {
        let target_htcycles = self.clock.get() + htcycles;
        loop {
            if self.queue.as_ref().is_none_or(|step| step.htcycles >= target_htcycles) {
                // No more tasks to execute or next task is scheduled
                // after target htcycles, so skip to target htcycles and break
                self.clock.set(target_htcycles);
                break true;
            }

            let TaskSlot { htcycles: task_htcycles, task_idx, next } = *self.queue.take().unwrap();
            self.queue = next;

            // Advance to task's htcycles and continue task execution
            self.clock.set(task_htcycles);
            match Pin::new(&mut self.tasks[task_idx]).resume(()) {
                CoroutineState::Yielded(TaskYield::Wait(offset)) => {
                    self.schedule(task_htcycles + offset, task_idx);
                }
                CoroutineState::Yielded(TaskYield::Break) => {
                    self.schedule(task_htcycles, task_idx);
                    break false;
                }
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
    use crate::{yield_break, yield_wait};
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
                    yield_wait!(self.state.clock.rising(3)); // skip to 3rd raising edge
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
                    yield_wait!(self.state.clock.falling(1)); // skip to 1st falling edge
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

        let mut scheduler = Scheduler::new(&clock, vec![foo.run(), bar.run()]);

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

    struct Breaker;
    impl Breaker {
        fn run<'a>(&'a self) -> Box<dyn NoReturnTask + 'a> {
            Box::new(#[coroutine] move || {
                loop {
                    yield_break!();
                }
            })
        }
    }

    #[test]
    fn scheduler_breaks_on_task_break() {
        let clock: Rc<Clock> = Default::default();
        let breaker = Breaker;
        let mut scheduler = Scheduler::new(&clock, vec![breaker.run()]);
        assert_eq!(scheduler.run(10), false);
    }

}
