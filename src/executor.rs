use futures::lock::Mutex;
use futures::task::{self, ArcWake, Waker};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::os::wasi::prelude::RawFd;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use wasmedge_wasi_socket::poll::{poll, Event, EventType, Subscription};

scoped_tls::scoped_thread_local!(pub(crate) static EXECUTOR: Executor);

struct Poller {
    subs: HashMap<i32, Subscription>,
}

impl Poller {
    fn add(&mut self, fd: RawFd) {
        self.subs.insert(
            fd,
            Subscription::IO {
                userdata: fd as u64,
                fd: fd,
                read_event: true,
                write_event: true,
            },
        );
    }
    fn delete(&mut self, fd: RawFd) {
        self.subs.remove(&fd);
    }
    fn modify(&mut self, fd: RawFd, interest: Interest) {
        self.subs.entry(fd).and_modify(|e| {
            *e = match interest {
                Interest::Read => Subscription::IO {
                    userdata: fd as u64,
                    fd: fd,
                    read_event: true,
                    write_event: false,
                },
                Interest::Write => Subscription::IO {
                    userdata: fd as u64,
                    fd: fd,
                    read_event: false,
                    write_event: true,
                },
                Interest::All => Subscription::IO {
                    userdata: fd as u64,
                    fd: fd,
                    read_event: true,
                    write_event: true,
                },
            };
        });
    }
    fn poll(&self) -> std::io::Result<Vec<Event>> {
        poll(
            &(self
                .subs
                .clone()
                .into_values()
                .collect::<Vec<Subscription>>()),
        )
    }
}

pub enum Interest {
    Read,
    Write,
    All,
}

pub struct TaskQueue {
    queue: RefCell<VecDeque<Arc<Task>>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        const DEFAULT_TASK_QUEUE_SIZE: usize = 4096;
        Self::new_with_capacity(DEFAULT_TASK_QUEUE_SIZE)
    }
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queue: RefCell::new(VecDeque::with_capacity(capacity)),
        }
    }

    pub(crate) fn push(&self, runnable: Arc<Task>) {
        // println!("add task");
        self.queue.borrow_mut().push_back(runnable);
    }

    pub(crate) fn pop(&self) -> Option<Arc<Task>> {
        // println!("remove task");
        self.queue.borrow_mut().pop_front()
    }
}

pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
        EXECUTOR.with(|ex| ex.tasks.push(arc_self.clone()));
    }
}

pub struct Reactor {
    poll: Poller,
    wakers_map: HashMap<u64, Waker>,
}

impl Reactor {
    pub fn new() -> Self {
        Self {
            poll: Poller {
                subs: HashMap::new(),
            },
            wakers_map: HashMap::new(),
        }
    }
    pub fn wait(&mut self) -> std::io::Result<()> {
        let events = self.poll.poll()?;
        for event in events {
            let token = event.userdata;
            let waker = match event.event_type {
                EventType::Read => self.wakers_map.remove(&(token * 2)),
                EventType::Write => self.wakers_map.remove(&(token * 2 + 1)),
                EventType::Timeout => None,
                EventType::Error(e) => {
                    return Err(e);
                }
            };
            if let Some(waker) = waker {
                waker.wake();
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Timeout is not suuported",
                ));
            }
        }
        Ok(())
    }
    pub fn add(&mut self, fd: RawFd) {
        self.poll.add(fd);
    }

    pub fn delete(&mut self, fd: RawFd) {
        self.wakers_map.remove(&(fd as u64 * 2));
        self.wakers_map.remove(&(fd as u64 * 2 + 1));
        self.poll.delete(fd);
    }

    pub fn modify(&mut self, fd: RawFd, interest: Interest, cx: &mut Context) {
        match interest {
            Interest::Read => {
                self.wakers_map.insert(fd as u64 * 2, cx.waker().clone());
            }
            Interest::Write => {
                self.wakers_map
                    .insert(fd as u64 * 2 + 1, cx.waker().clone());
            }
            Interest::All => {
                self.wakers_map.insert(fd as u64 * 2, cx.waker().clone());
                self.wakers_map
                    .insert(fd as u64 * 2 + 1, cx.waker().clone());
            }
        }
        self.poll.modify(fd, interest)
    }
}

pub struct Executor {
    tasks: TaskQueue,
    pub reactor: RefCell<Reactor>,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let task = Task {
        future: Mutex::new(Box::pin(future)),
    };
    EXECUTOR.with(|ex| {
        ex.tasks.push(Arc::new(task));
    });
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: TaskQueue::new(),
            reactor: RefCell::new(Reactor::new()),
        }
    }

    pub fn block_on<F, T, O>(&mut self, f: F) -> std::io::Result<O>
    where
        F: Fn() -> T,
        T: Future<Output = O> + 'static,
    {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        EXECUTOR.set(self, || {
            let mut fut = f();
            let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
            let ret = loop {
                if let std::task::Poll::Ready(t) = fut.as_mut().poll(&mut cx) {
                    break t;
                }
                while let Some(t) = self.tasks.pop() {
                    if let Some(future) = t.future.try_lock() {
                        let w = task::waker(t.clone());
                        let mut context = Context::from_waker(&w);
                        let _ = Pin::new(future).as_mut().poll(&mut context);
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Cannot lock",
                        ));
                    }
                }

                if let Err(e) = self.reactor.borrow_mut().wait() {
                    return Err(e);
                };
            };
            Ok(ret)
        })
    }
}
