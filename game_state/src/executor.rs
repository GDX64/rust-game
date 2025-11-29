//! A simple single-threaded executor that can spawn non-`Send` futures.

pub use async_task::{Runnable, Task};
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{OnceLock, RwLock};
use std::task::Waker;
use std::time::{Duration, SystemTime};

static GLOBAL_EXECUTOR_SENDER: OnceLock<Sender<Runnable>> = OnceLock::new();
static GLOBAL_REACTOR_SENDER: OnceLock<Sender<Waker>> = OnceLock::new();
static TIME_PROVIDER: RwLock<SystemTime> = RwLock::new(SystemTime::UNIX_EPOCH);

struct Reactor {
    tick_sender: Sender<Waker>,
    tick_receiver: Receiver<Waker>,
}

impl Reactor {
    fn new() -> Self {
        let (tick_sender, tick_receiver) = std::sync::mpsc::channel();
        Self {
            tick_sender,
            tick_receiver,
        }
    }
}

pub struct Delay {
    when: SystemTime,
}

impl Future for Delay {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Delay::current_time() >= self.when {
            std::task::Poll::Ready(())
        } else {
            let sender = GLOBAL_REACTOR_SENDER
                .get()
                .expect("Global reactor not initialized")
                .clone();
            sender.send(cx.waker().clone()).unwrap();
            std::task::Poll::Pending
        }
    }
}

impl Delay {
    pub fn from_duration(dur: Duration) -> Self {
        Self {
            when: Delay::current_time() + dur,
        }
    }

    pub fn current_time() -> SystemTime {
        *TIME_PROVIDER.read().unwrap()
    }
}

pub struct GlExec {
    queue: (Sender<Runnable>, Receiver<Runnable>),
    reactor: Reactor,
}

impl GlExec {
    fn new() -> Self {
        let queue = std::sync::mpsc::channel();
        let reactor = Reactor::new();
        Self { queue, reactor }
    }

    pub fn new_global(time: SystemTime) -> Self {
        let exec = Self::new();
        exec.register_globally();
        *TIME_PROVIDER.write().unwrap() = time;
        return exec;
    }

    pub fn register_globally(&self) {
        GLOBAL_REACTOR_SENDER.get_or_init(|| self.reactor.tick_sender.clone());
        GLOBAL_EXECUTOR_SENDER.get_or_init(|| self.queue.0.clone());
        log::info!("Global executor registered");
    }

    pub fn spawn<F, T>(future: F) -> Task<T>
    where
        F: Future<Output = T> + 'static + Send,
        T: 'static + Send,
    {
        // Create a task that is scheduled by pushing itself into the queue.
        let sender = GLOBAL_EXECUTOR_SENDER
            .get()
            .expect("Global executor not initialized")
            .clone();
        let schedule = move |runnable| {
            if let Err(e) = sender.send(runnable) {
                eprintln!("Error sending runnable: {}", e);
            }
        };
        let (runnable, task) = async_task::spawn(future, schedule);

        // Schedule the task by pushing it into the queue.
        runnable.schedule();

        task
    }

    pub fn block_on(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Self::spawn(future);
        loop {
            self.tick(SystemTime::now());
            if task.is_finished() {
                return;
            }
            std::thread::yield_now();
        }
    }

    pub fn tick(&self, time: SystemTime) {
        // Run all tasks that are ready to run.
        while let Ok(waker) = self.reactor.tick_receiver.try_recv() {
            waker.wake();
        }
        while let Ok(runnable) = self.queue.1.try_recv() {
            runnable.run();
        }
        *TIME_PROVIDER.write().unwrap() = time;
    }
}

#[cfg(test)]
mod tests {

    use std::{
        thread::sleep,
        time::{Duration, SystemTime},
    };

    use crate::GlExec;

    #[test]
    fn executor() {
        let exec = GlExec::new_global(SystemTime::now());
        let task2 = GlExec::spawn(async move {
            let task1 = GlExec::spawn(async {
                return 10;
            });
            let result = task1.await;
            let delay = crate::Delay::from_duration(Duration::from_millis(25));
            delay.await;
            return result;
        });

        loop {
            if task2.is_finished() {
                return;
            }
            exec.tick(SystemTime::now());
            sleep(Duration::from_millis(16));
            println!("Tick, time = {:?}", crate::Delay::current_time());
        }
    }
}
