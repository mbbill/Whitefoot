//! Async TCP plus bounded CPU offload for the existing compute_protocol.h.
//! One current-thread I/O driver and B-1 Rayon workers share a total B-thread
//! execution budget. The CPU-only layout control remains an independent bin.
//! `mixed-retain` transfers admission with the result until its consumer takes
//! or discards it. Without that feature the original publication policy stays.

use std::{
    env,
    future::poll_fn,
    io,
    net::{Ipv4Addr, SocketAddrV4},
    process::ExitCode,
    sync::Arc,
    task::Poll,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Builder,
    sync::{Semaphore, oneshot},
    task::JoinSet,
};

const MAX_ROUNDS: u64 = 16_777_216;

fn churn(mut value: u64, rounds: u64) -> u64 {
    for _ in 0..rounds {
        value = (value ^ value.rotate_left(17))
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
    }
    value
}

#[derive(Clone, Copy)]
struct Config {
    port: u16,
    connections: usize,
    threads: usize,
    queue: u32,
}

impl Config {
    fn parse(arguments: &[String]) -> Result<Self, &'static str> {
        let usage = "expected PORT CONNECTIONS --threads TOTAL [--queue JOBS]; TOTAL includes one I/O thread";
        if arguments.len() != 5 && arguments.len() != 7 {
            return Err(usage);
        }
        if arguments[3] != "--threads" || (arguments.len() == 7 && arguments[5] != "--queue") {
            return Err(usage);
        }
        let port = arguments[1].parse().map_err(|_| usage)?;
        let connections = arguments[2].parse().map_err(|_| usage)?;
        let threads = arguments[4].parse().map_err(|_| usage)?;
        if port == 0 || !(1..=1_048_576).contains(&connections) || !(2..=256).contains(&threads) {
            return Err("port must be 1..65535, connections 1..1048576, total threads 2..256");
        }
        let queue = if arguments.len() == 7 {
            arguments[6].parse().map_err(|_| usage)?
        } else {
            2 * (threads as u32 - 1)
        };
        if !(1..=1_048_576).contains(&queue) {
            return Err("queue must be 1..1048576 (includes running CPU jobs)");
        }
        Ok(Self {
            port,
            connections,
            threads,
            queue,
        })
    }
}

#[cfg(any(test, feature = "mixed-observe"))]
mod observe {
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    #[derive(Default)]
    pub(super) struct Stats {
        pub waiting: AtomicUsize,
        pub waiting_peak: AtomicUsize,
        pub inflight: AtomicUsize,
        pub inflight_peak: AtomicUsize,
        pub active: AtomicUsize,
        pub active_peak: AtomicUsize,
        pub submitted: AtomicUsize,
        pub completed: AtomicUsize,
        pub light_while_active: AtomicUsize,
        pub light_while_saturated: AtomicUsize,
        pub send_waits: AtomicUsize,
    }

    pub(super) struct Count<'a>(&'a AtomicUsize);

    impl<'a> Count<'a> {
        pub fn enter(current: &'a AtomicUsize, peak: &AtomicUsize) -> Self {
            peak.fetch_max(current.fetch_add(1, SeqCst) + 1, SeqCst);
            Self(current)
        }
    }

    impl Drop for Count<'_> {
        fn drop(&mut self) {
            self.0.fetch_sub(1, SeqCst);
        }
    }
}

#[cfg(feature = "mixed-retain")]
mod retained {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::SeqCst},
    };
    use tokio::sync::{Notify, OwnedSemaphorePermit};

    #[derive(Default)]
    pub(super) struct Tails {
        unfinished: AtomicUsize,
        changed: Notify,
        #[cfg(any(test, feature = "mixed-observe"))]
        pub produced: AtomicUsize,
        #[cfg(any(test, feature = "mixed-observe"))]
        pub consumed: AtomicUsize,
        #[cfg(any(test, feature = "mixed-observe"))]
        pub discarded: AtomicUsize,
        #[cfg(any(test, feature = "mixed-observe"))]
        pub held: AtomicUsize,
        #[cfg(any(test, feature = "mixed-observe"))]
        pub held_peak: AtomicUsize,
    }

    impl Tails {
        pub fn start(self: &Arc<Self>) -> Tail {
            self.unfinished.fetch_add(1, SeqCst);
            Tail(self.clone())
        }

        pub fn unfinished(&self) -> usize {
            self.unfinished.load(SeqCst)
        }

        pub async fn drain(&self) {
            // The sole owner calls this after all handlers are joined: no new
            // submission can race the zero check. notify_one retains a permit
            // if the final decrement precedes polling the notification.
            while self.unfinished() != 0 {
                self.changed.notified().await;
            }
        }
    }

    pub(super) struct Tail(Arc<Tails>);

    impl Tail {
        pub fn result(&self, value: u64, permit: OwnedSemaphorePermit) -> Result {
            Result::new(value, permit, &self.0)
        }
    }

    impl Drop for Tail {
        fn drop(&mut self) {
            if self.0.unfinished.fetch_sub(1, SeqCst) == 1 {
                self.0.changed.notify_one();
            }
        }
    }

    pub(super) struct Result {
        value: u64,
        _permit: OwnedSemaphorePermit,
        #[cfg(any(test, feature = "mixed-observe"))]
        tails: Arc<Tails>,
        #[cfg(any(test, feature = "mixed-observe"))]
        consumed: std::cell::Cell<bool>,
    }

    impl Result {
        pub fn new(value: u64, permit: OwnedSemaphorePermit, tails: &Arc<Tails>) -> Self {
            // Produced includes a send rejected by an already canceled owner.
            // Discarded also covers a received-but-never-consumed channel value.
            #[cfg(any(test, feature = "mixed-observe"))]
            {
                tails.produced.fetch_add(1, SeqCst);
                let held = tails.held.fetch_add(1, SeqCst) + 1;
                tails.held_peak.fetch_max(held, SeqCst);
            }
            #[cfg(not(any(test, feature = "mixed-observe")))]
            let _ = tails;
            Self {
                value,
                _permit: permit,
                #[cfg(any(test, feature = "mixed-observe"))]
                tails: tails.clone(),
                #[cfg(any(test, feature = "mixed-observe"))]
                consumed: std::cell::Cell::new(false),
            }
        }

        pub fn take(self) -> u64 {
            #[cfg(any(test, feature = "mixed-observe"))]
            {
                self.consumed.set(true);
            }
            // Destruction returns admission before the handler encodes/writes.
            self.value
        }
    }

    impl Drop for Result {
        fn drop(&mut self) {
            #[cfg(any(test, feature = "mixed-observe"))]
            {
                self.tails.held.fetch_sub(1, SeqCst);
                let counter = if self.consumed.get() {
                    &self.tails.consumed
                } else {
                    &self.tails.discarded
                };
                counter.fetch_add(1, SeqCst);
            }
        }
    }
}

struct Engine {
    pool: rayon::ThreadPool,
    slots: Arc<Semaphore>,
    queue: u32,
    #[cfg(any(test, feature = "mixed-observe"))]
    stats: observe::Stats,
    #[cfg(feature = "mixed-retain")]
    tails: Arc<retained::Tails>,
    #[cfg(test)]
    gates: std::sync::Mutex<Option<Arc<tests::JobGates>>>,
    #[cfg(test)]
    write_gate: std::sync::Mutex<Option<tests::WriteGate>>,
}

impl Engine {
    fn new(config: Config) -> io::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(config.threads - 1)
                .build()
                .map_err(io::Error::other)?,
            slots: Arc::new(Semaphore::new(config.queue as usize)),
            queue: config.queue,
            #[cfg(any(test, feature = "mixed-observe"))]
            stats: observe::Stats::default(),
            #[cfg(feature = "mixed-retain")]
            tails: Arc::default(),
            #[cfg(test)]
            gates: std::sync::Mutex::new(None),
            #[cfg(test)]
            write_gate: std::sync::Mutex::new(None),
        }))
    }

    async fn compute(self: &Arc<Self>, seed: u64, rounds: u64) -> io::Result<u64> {
        #[cfg(any(test, feature = "mixed-observe"))]
        let waiting = observe::Count::enter(&self.stats.waiting, &self.stats.waiting_peak);
        let permit = self
            .slots
            .clone()
            .acquire_owned()
            .await
            .map_err(io::Error::other)?;
        #[cfg(any(test, feature = "mixed-observe"))]
        drop(waiting);
        let (send, receive) = oneshot::channel();
        #[cfg(feature = "mixed-retain")]
        let tail = self.tails.start();
        #[cfg(test)]
        let gates = self.gates.lock().unwrap().clone();
        #[cfg(any(test, feature = "mixed-observe"))]
        let engine = self.clone();
        #[cfg(any(test, feature = "mixed-observe"))]
        {
            use std::sync::atomic::Ordering::SeqCst;
            self.stats.submitted.fetch_add(1, SeqCst);
            let count = self.stats.inflight.fetch_add(1, SeqCst) + 1;
            self.stats.inflight_peak.fetch_max(count, SeqCst);
        }
        self.pool.spawn_fifo(move || {
            #[cfg(test)]
            if let Some(gates) = &gates {
                gates.before.wait();
            }
            #[cfg(any(test, feature = "mixed-observe"))]
            let active = observe::Count::enter(&engine.stats.active, &engine.stats.active_peak);
            let value = churn(seed, rounds);
            #[cfg(not(feature = "mixed-retain"))]
            let _ = send.send(value);
            #[cfg(any(test, feature = "mixed-observe"))]
            {
                use std::sync::atomic::Ordering::SeqCst;
                drop(active);
                engine.stats.completed.fetch_add(1, SeqCst);
                engine.stats.inflight.fetch_sub(1, SeqCst);
            }
            // Release admission independently of socket backpressure after
            // publishing the result; the receiver may already be writing.
            #[cfg(not(feature = "mixed-retain"))]
            drop(permit);
            #[cfg(feature = "mixed-retain")]
            let _ = send.send(tail.result(value, permit));
            #[cfg(test)]
            if let Some(gates) = &gates {
                gates.after.wait();
            }
            #[cfg(any(test, feature = "mixed-observe"))]
            drop(engine);
            // The consumer can release the logical permit before this point.
            // Drain also waits for this final source-controlled cleanup point;
            // the guard's notification/Arc drop and Rayon's internal epilogue
            // can still be finishing. Their storage remains independently owned.
            #[cfg(feature = "mixed-retain")]
            drop(tail);
        });
        #[cfg(not(feature = "mixed-retain"))]
        {
            receive.await.map_err(io::Error::other)
        }
        #[cfg(feature = "mixed-retain")]
        {
            receive
                .await
                .map(retained::Result::take)
                .map_err(io::Error::other)
        }
    }

    async fn drain(&self) -> io::Result<()> {
        // All handlers must first finish or be aborted and joined. Taking
        // every slot then proves that no queued/running job retains a permit.
        // Retained results can return theirs while the publisher is finishing,
        // so that mode also awaits its independent closure-tail accounting.
        // Dropping ThreadPool alone is not used as a synchronous join.
        let permits = self
            .slots
            .acquire_many(self.queue)
            .await
            .map_err(io::Error::other)?;
        drop(permits);
        #[cfg(feature = "mixed-retain")]
        self.tails.drain().await;
        Ok(())
    }
}

async fn serve_one(mut socket: TcpStream, engine: Arc<Engine>) -> io::Result<()> {
    socket.set_nodelay(true)?;
    // Untimed qualification makes TCP backpressure reachable with bounded
    // fixtures. Production and ordinary timing retain the host defaults.
    #[cfg(test)]
    socket2::SockRef::from(&socket).set_send_buffer_size(4096)?;
    let mut bytes = [0_u8; 64];
    loop {
        let mut received = 0;
        while received < bytes.len() {
            let next = match socket.read(&mut bytes[received..]).await {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result?,
            };
            if next == 0 {
                return if received == 0 {
                    Ok(())
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "truncated compute frame",
                    ))
                };
            }
            received += next;
        }
        let seed = u64::from_be_bytes(bytes[..8].try_into().expect("fixed eight-byte prefix"));
        let rounds = u64::from_be_bytes(bytes[8..16].try_into().expect("fixed eight-byte word"));
        if rounds > MAX_ROUNDS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "compute count exceeds protocol maximum",
            ));
        }
        let value = if rounds == 0 {
            #[cfg(any(test, feature = "mixed-observe"))]
            {
                use std::sync::atomic::Ordering::SeqCst;
                let active = engine.stats.active.load(SeqCst);
                if active > 0 {
                    engine.stats.light_while_active.fetch_add(1, SeqCst);
                }
                if active == engine.pool.current_num_threads() {
                    engine.stats.light_while_saturated.fetch_add(1, SeqCst);
                }
            }
            seed
        } else {
            engine.compute(seed, rounds).await?
        };
        for (bit, byte) in bytes.iter_mut().enumerate() {
            *byte = ((value >> bit) & 1) as u8;
        }
        #[cfg(any(test, feature = "mixed-observe"))]
        {
            use std::{future::Future, sync::atomic::Ordering::SeqCst};
            let mut writing = std::pin::pin!(socket.write_all(&bytes));
            #[cfg(test)]
            let mut pause = None;
            poll_fn(|cx| {
                #[cfg(test)]
                if tests::poll_write_pause(&mut pause, cx) {
                    return Poll::Pending;
                }
                let result = writing.as_mut().poll(cx);
                if result.is_pending() {
                    engine.stats.send_waits.fetch_add(1, SeqCst);
                    #[cfg(test)]
                    if let Some(gate) = engine.write_gate.lock().unwrap().take() {
                        pause = Some(gate.pause());
                        tests::poll_write_pause(&mut pause, cx);
                    }
                }
                result
            })
            .await?;
        }
        #[cfg(not(any(test, feature = "mixed-observe")))]
        socket.write_all(&bytes).await?;
        // Only the next iteration reads another frame: at most one request,
        // reply or admission waiter is retained per accepted connection.
    }
}

async fn serve(listener: TcpListener, count: usize, engine: Arc<Engine>) -> io::Result<()> {
    let mut accepted = 0;
    let mut handlers = JoinSet::new();
    let result = async {
        while accepted < count || !handlers.is_empty() {
            // Poll completed handlers before accept so a protocol failure
            // can stop admission even when fewer than count peers connected.
            enum Event {
                Accepted(io::Result<TcpStream>),
                Joined(Result<io::Result<()>, tokio::task::JoinError>),
            }
            let event = poll_fn(|cx| {
                if let Poll::Ready(Some(result)) = handlers.poll_join_next(cx) {
                    return Poll::Ready(Event::Joined(result));
                }
                if accepted < count {
                    return listener
                        .poll_accept(cx)
                        .map(|result| Event::Accepted(result.map(|(socket, _)| socket)));
                }
                Poll::Pending
            })
            .await;
            match event {
                Event::Accepted(socket) => {
                    accepted += 1;
                    handlers.spawn(serve_one(socket?, engine.clone()));
                }
                Event::Joined(result) => result.map_err(io::Error::other)??,
            }
        }
        Ok(())
    }
    .await;
    drop(listener);
    handlers.abort_all();
    while handlers.join_next().await.is_some() {}
    engine.drain().await?;
    result
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse(&env::args().collect::<Vec<_>>())?;
    let engine = Engine::new(config)?;
    let runtime = Builder::new_current_thread().enable_io().build()?;
    let result = runtime.block_on(async {
        let listener =
            TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, config.port)).await?;
        serve(listener, config.connections, engine.clone()).await
    });
    #[cfg(feature = "mixed-observe")]
    {
        use std::sync::atomic::Ordering::SeqCst;
        let s = &engine.stats;
        eprintln!(
            "mixed-rayon: total_threads={} io_threads=1 cpu_threads={} queue={} submitted={} completed={} inflight={} inflight_peak={} active={} active_peak={} waiting={} waiting_peak={} light_while_active={} light_while_saturated={} send_waits={}",
            config.threads,
            config.threads - 1,
            config.queue,
            s.submitted.load(SeqCst),
            s.completed.load(SeqCst),
            s.inflight.load(SeqCst),
            s.inflight_peak.load(SeqCst),
            s.active.load(SeqCst),
            s.active_peak.load(SeqCst),
            s.waiting.load(SeqCst),
            s.waiting_peak.load(SeqCst),
            s.light_while_active.load(SeqCst),
            s.light_while_saturated.load(SeqCst),
            s.send_waits.load(SeqCst)
        );
    }
    #[cfg(all(feature = "mixed-retain", feature = "mixed-observe"))]
    {
        use std::sync::atomic::Ordering::SeqCst;
        let t = &engine.tails;
        eprintln!(
            "mixed-retain: permit_release=consumer produced={} consumed={} discarded={} held={} held_peak={} unfinished_tails={}",
            t.produced.load(SeqCst),
            t.consumed.load(SeqCst),
            t.discarded.load(SeqCst),
            t.held.load(SeqCst),
            t.held_peak.load(SeqCst),
            t.unfinished(),
        );
    }
    result?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("mixed-rayon: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        io::{Read, Write},
        net::{Shutdown, SocketAddr, TcpStream as BlockingStream},
        sync::{atomic::Ordering::SeqCst, mpsc},
        thread,
        time::{Duration, Instant},
    };

    pub(super) struct Gate {
        entered: mpsc::Sender<()>,
        release: std::sync::Mutex<mpsc::Receiver<()>>,
    }

    impl Gate {
        pub(super) fn wait(&self) {
            self.entered.send(()).unwrap();
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
        }
    }

    pub(super) struct JobGates {
        pub before: Gate,
        pub after: Gate,
    }

    pub(super) struct WriteGate {
        entered: mpsc::Sender<()>,
        release: Arc<Semaphore>,
    }

    type WritePause = std::pin::Pin<Box<dyn Future<Output = ()> + Send>>;

    impl WriteGate {
        pub fn pause(self) -> WritePause {
            Box::pin(async move {
                self.entered.send(()).unwrap();
                self.release.acquire().await.unwrap().forget();
            })
        }
    }

    pub(super) fn poll_write_pause(
        pause: &mut Option<WritePause>,
        cx: &mut std::task::Context<'_>,
    ) -> bool {
        if let Some(future) = pause {
            if future.as_mut().poll(cx).is_pending() {
                return true;
            }
            *pause = None;
        }
        false
    }

    struct GateControl {
        entered: mpsc::Receiver<()>,
        release: mpsc::Sender<()>,
    }

    impl GateControl {
        fn pair() -> (Gate, Self) {
            let (entered, receive) = mpsc::channel();
            let (release, next) = mpsc::channel();
            (
                Gate {
                    entered,
                    release: std::sync::Mutex::new(next),
                },
                Self {
                    entered: receive,
                    release,
                },
            )
        }

        fn entered(&self) {
            self.entered.recv_timeout(Duration::from_secs(5)).unwrap();
        }

        fn release(&self) {
            self.release.send(()).unwrap();
        }
    }

    fn gated_engine(queue: u32) -> (Arc<Engine>, GateControl, GateControl) {
        let engine = Engine::new(Config {
            port: 1,
            connections: 4,
            threads: 2,
            queue,
        })
        .unwrap();
        let (before, start) = GateControl::pair();
        let (after, finish) = GateControl::pair();
        *engine.gates.lock().unwrap() = Some(Arc::new(JobGates { before, after }));
        (engine, start, finish)
    }

    fn poll_once<F: Future>(future: std::pin::Pin<&mut F>) -> Poll<F::Output> {
        future.poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
    }

    fn finish_engine(engine: &Engine) {
        Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(engine.drain())
            .unwrap();
        assert_eq!(engine.slots.available_permits(), engine.queue as usize);
        assert_eq!(engine.stats.inflight.load(SeqCst), 0);
        assert_eq!(engine.stats.active.load(SeqCst), 0);
        assert_eq!(engine.stats.waiting.load(SeqCst), 0);
        assert_eq!(
            engine.stats.submitted.load(SeqCst),
            engine.stats.completed.load(SeqCst)
        );
        #[cfg(feature = "mixed-retain")]
        assert_retired(engine);
    }

    #[cfg(feature = "mixed-retain")]
    fn assert_retired(engine: &Engine) {
        let t = &engine.tails;
        assert_eq!(t.unfinished(), 0);
        assert_eq!(t.held.load(SeqCst), 0);
        assert!(t.held_peak.load(SeqCst) <= engine.queue as usize);
        assert_eq!(t.produced.load(SeqCst), engine.stats.completed.load(SeqCst));
        assert_eq!(
            t.produced.load(SeqCst),
            t.consumed.load(SeqCst) + t.discarded.load(SeqCst)
        );
    }

    #[test]
    fn finished_results_distinguish_admission_from_consumption_and_closure_tail() {
        let (engine, start, finish) = gated_engine(2);
        let mut first = Box::pin(engine.compute(0, 1));
        let mut second = Box::pin(engine.compute(0, 1));
        assert!(poll_once(first.as_mut()).is_pending());
        assert!(poll_once(second.as_mut()).is_pending());
        start.entered();
        start.release();
        finish.entered();
        finish.release();
        start.entered();
        start.release();
        finish.entered();
        // Both values are published, and neither consumer has been polled.
        assert_eq!(engine.stats.completed.load(SeqCst), 2);
        assert_eq!(
            engine.slots.available_permits(),
            if cfg!(feature = "mixed-retain") { 0 } else { 2 }
        );
        let mut third = Box::pin(engine.compute(0, 1));
        assert!(poll_once(third.as_mut()).is_pending());
        assert_eq!(
            engine.stats.submitted.load(SeqCst),
            if cfg!(feature = "mixed-retain") { 2 } else { 3 }
        );
        // Cancellation while awaiting admission must neither create a job nor
        // release a permit held by another request.
        #[cfg(feature = "mixed-retain")]
        {
            let mut canceled_waiter = Box::pin(engine.compute(0, 1));
            assert!(poll_once(canceled_waiter.as_mut()).is_pending());
            drop(canceled_waiter);
            assert_eq!(engine.slots.available_permits(), 0);
            assert_eq!(engine.stats.submitted.load(SeqCst), 2);
        }
        assert!(matches!(
            poll_once(first.as_mut()),
            Poll::Ready(Ok(1_442_695_040_888_963_407))
        ));
        assert!(poll_once(third.as_mut()).is_pending());
        assert_eq!(engine.stats.submitted.load(SeqCst), 3);
        drop(first);
        drop(second); // A published result is discarded, releasing its permit.
        drop(third); // A queued producer still owns its permit until completion.
        let mut draining = Box::pin(engine.drain());
        assert!(poll_once(draining.as_mut()).is_pending());
        finish.release();
        start.entered();
        start.release();
        finish.entered();
        drop(draining);
        assert_eq!(engine.slots.available_permits(), 2);
        #[cfg(feature = "mixed-retain")]
        {
            assert_eq!(engine.tails.consumed.load(SeqCst), 1);
            assert_eq!(engine.tails.discarded.load(SeqCst), 2);
            assert_eq!(engine.tails.unfinished(), 1);
            let mut draining = Box::pin(engine.drain());
            assert!(poll_once(draining.as_mut()).is_pending());
        }
        finish.release();
        finish_engine(&engine);
    }

    #[test]
    fn canceling_a_running_consumer_keeps_admission_until_the_producer_finishes() {
        let (engine, start, finish) = gated_engine(1);
        let mut computing = Box::pin(engine.compute(0, 1));
        assert!(poll_once(computing.as_mut()).is_pending());
        start.entered();
        drop(computing);
        assert_eq!(engine.slots.available_permits(), 0);
        let mut draining = Box::pin(engine.drain());
        assert!(poll_once(draining.as_mut()).is_pending());
        start.release();
        finish.entered();
        drop(draining);
        assert_eq!(engine.slots.available_permits(), 1);
        #[cfg(feature = "mixed-retain")]
        {
            assert_eq!(engine.tails.discarded.load(SeqCst), 1);
            let mut draining = Box::pin(engine.drain());
            assert!(poll_once(draining.as_mut()).is_pending());
        }
        finish.release();
        finish_engine(&engine);
    }

    struct Server {
        address: SocketAddr,
        engine: Arc<Engine>,
        result: mpsc::Receiver<io::Result<()>>,
        thread: thread::JoinHandle<()>,
    }

    impl Server {
        fn new(connections: usize, threads: usize, queue: u32) -> Self {
            let engine = Engine::new(Config {
                port: 1,
                connections,
                threads,
                queue,
            })
            .unwrap();
            let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let (send, result) = mpsc::channel();
            let owner = engine.clone();
            let thread = thread::spawn(move || {
                let runtime = Builder::new_current_thread().enable_io().build().unwrap();
                let result = runtime.block_on(async {
                    serve(TcpListener::from_std(listener)?, connections, owner).await
                });
                let _ = send.send(result);
            });
            Self {
                address,
                engine,
                result,
                thread,
            }
        }

        fn connect(&self) -> BlockingStream {
            let stream = BlockingStream::connect(self.address).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream.set_nodelay(true).unwrap();
            stream
        }

        fn finish(self, expected: io::Result<()>) {
            self.finish_matching(|result| {
                assert_eq!(
                    result.as_ref().err().map(io::Error::kind),
                    expected.as_ref().err().map(io::Error::kind)
                );
            });
        }

        fn finish_matching(self, expected: impl FnOnce(&io::Result<()>)) {
            let result = self.result.recv_timeout(Duration::from_secs(5)).unwrap();
            expected(&result);
            self.thread.join().unwrap();
            let s = &self.engine.stats;
            assert_eq!(s.submitted.load(SeqCst), s.completed.load(SeqCst));
            assert_eq!(s.inflight.load(SeqCst), 0);
            assert_eq!(s.active.load(SeqCst), 0);
            assert_eq!(s.waiting.load(SeqCst), 0);
            assert!(s.inflight_peak.load(SeqCst) <= self.engine.queue as usize);
            assert!(s.active_peak.load(SeqCst) <= self.engine.pool.current_num_threads());
            assert_eq!(
                self.engine.slots.available_permits(),
                self.engine.queue as usize
            );
            #[cfg(feature = "mixed-retain")]
            assert_retired(&self.engine);
        }
    }

    fn request(seed: u64, rounds: u64) -> [u8; 64] {
        let mut bytes = [0x5a; 64]; // Reserved bytes are not constrained by the protocol.
        bytes[..8].copy_from_slice(&seed.to_be_bytes());
        bytes[8..16].copy_from_slice(&rounds.to_be_bytes());
        bytes
    }

    fn response(stream: &mut BlockingStream, expected: u64) {
        let mut bytes = [0; 64];
        stream.read_exact(&mut bytes).unwrap();
        for (bit, byte) in bytes.iter().enumerate() {
            assert_eq!(*byte, ((expected >> bit) & 1) as u8);
        }
    }

    fn eof(stream: &mut BlockingStream) {
        let mut byte = [0];
        assert_eq!(stream.read(&mut byte).unwrap(), 0);
    }

    fn wait_for(mut predicate: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !predicate() {
            assert!(
                Instant::now() < deadline,
                "observed lifecycle condition did not arrive"
            );
            thread::yield_now();
        }
    }

    #[test]
    fn fixed_external_vectors_and_every_frame_split_survive_full_frame_half_close() {
        // The nonzero vectors are independently fixed in stream_check.c.
        for (seed, rounds, answer) in [
            (0, 1, 1_442_695_040_888_963_407),
            (
                11_400_714_819_323_198_485,
                65_536,
                14_034_923_464_053_623_880,
            ),
            (u64::MAX, 4096, 16_385_165_208_160_242_592),
        ] {
            assert_eq!(churn(seed, rounds), answer);
        }
        let server = Server::new(1, 4, 6);
        let mut client = server.connect();
        for split in 1..64 {
            let bytes = request(0, 1);
            client.write_all(&bytes[..split]).unwrap();
            client.write_all(&bytes[split..]).unwrap();
            response(&mut client, 1_442_695_040_888_963_407);
        }
        // Pipelined input must still receive exactly ordered, complete frames.
        let mut pair = request(9, 0).to_vec();
        pair.extend_from_slice(&request(11_400_714_819_323_198_485, 65_536));
        client.write_all(&pair).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        response(&mut client, 9);
        response(&mut client, 14_034_923_464_053_623_880);
        eof(&mut client);
        server.finish(Ok(()));
    }

    #[test]
    fn saturated_cpu_and_admission_leave_light_io_live_and_bound_retained_work() {
        for (threads, queue) in [(2, 1), (4, 6)] {
            let server = Server::new(queue as usize + 4, threads, queue);
            let mut heavy: Vec<_> = (0..queue + 3).map(|_| server.connect()).collect();
            let mut light = server.connect();
            // Real dependent CPU work, with three more frames than admission
            // slots. No blocking stand-in occupies the workers.
            for peer in &mut heavy {
                peer.write_all(&request(7, MAX_ROUNDS)).unwrap();
            }
            wait_for(|| {
                server.engine.stats.active.load(SeqCst) == threads - 1
                    && server.engine.stats.waiting.load(SeqCst) >= 2
            });
            for _ in 0..8 {
                light.write_all(&request(123, 0)).unwrap();
                response(&mut light, 123);
            }
            assert!(server.engine.stats.light_while_saturated.load(SeqCst) > 0);
            light.shutdown(Shutdown::Write).unwrap();
            eof(&mut light);
            // The expected max-round answer is an independently fixed C oracle
            // vector; it is not computed concurrently with the measured server.
            for peer in &mut heavy {
                response(peer, 0xc39350d53d849e85);
                peer.shutdown(Shutdown::Write).unwrap();
                eof(peer);
            }
            assert_eq!(
                server.engine.stats.inflight_peak.load(SeqCst),
                queue as usize
            );
            assert!(server.engine.stats.waiting_peak.load(SeqCst) >= 2);
            server.finish(Ok(()));
        }
    }

    #[test]
    fn errors_cancel_handlers_and_drain_cpu_jobs_before_returning() {
        for truncated in [false, true] {
            let server = Server::new(8, 2, 1);
            let mut heavy = server.connect();
            let mut bad = server.connect();
            heavy.write_all(&request(7, MAX_ROUNDS)).unwrap();
            wait_for(|| server.engine.stats.active.load(SeqCst) == 1);
            if truncated {
                bad.write_all(&request(0, 0)[..63]).unwrap();
                bad.shutdown(Shutdown::Write).unwrap();
            } else {
                bad.write_all(&request(0, MAX_ROUNDS + 1)).unwrap();
            }
            // The failure must stop before all eight planned accepts arrive,
            // cancel the unrelated handler, then await its CPU-owned permit.
            let kind = if truncated {
                io::ErrorKind::UnexpectedEof
            } else {
                io::ErrorKind::InvalidData
            };
            server.finish(Err(io::Error::from(kind)));
        }
    }

    #[test]
    fn reset_during_cpu_work_reclaims_admission_and_reports_failure() {
        let server = Server::new(4, 2, 1);
        let mut client = server.connect();
        client.write_all(&request(7, MAX_ROUNDS)).unwrap();
        wait_for(|| server.engine.stats.active.load(SeqCst) == 1);
        socket2::SockRef::from(&client)
            .set_linger(Some(Duration::ZERO))
            .unwrap();
        drop(client);
        server.finish_matching(|result| {
            assert!(matches!(
                result.as_ref().err().map(io::Error::kind),
                Some(
                    io::ErrorKind::ConnectionReset
                        | io::ErrorKind::BrokenPipe
                        | io::ErrorKind::ConnectionAborted
                )
            ));
        });
    }

    #[test]
    fn a_backpressured_writer_keeps_only_one_reply_and_other_io_progresses() {
        let server = Server::new(2, 2, 1);
        let mut slow = server.connect();
        let mut light = server.connect();
        socket2::SockRef::from(&slow)
            .set_recv_buffer_size(4096)
            .unwrap();
        let mut requests = Vec::with_capacity(64 * 2048);
        for seed in 0..2048 {
            requests.extend_from_slice(&request(seed, 0));
        }
        slow.write_all(&requests).unwrap();
        wait_for(|| server.engine.stats.send_waits.load(SeqCst) > 0);
        light.write_all(&request(123, 0)).unwrap();
        response(&mut light, 123);
        light.shutdown(Shutdown::Write).unwrap();
        eof(&mut light);
        for seed in 0..2048 {
            response(&mut slow, seed);
        }
        slow.shutdown(Shutdown::Write).unwrap();
        eof(&mut slow);
        server.finish(Ok(()));
    }

    #[cfg(feature = "mixed-retain")]
    #[test]
    fn a_backpressured_cpu_reply_has_already_returned_its_permit() {
        let server = Server::new(2, 2, 1);
        let (entered, paused) = mpsc::channel();
        let release = Arc::new(Semaphore::new(0));
        *server.engine.write_gate.lock().unwrap() = Some(WriteGate {
            entered,
            release: release.clone(),
        });
        let mut slow = server.connect();
        let mut other = server.connect();
        socket2::SockRef::from(&slow)
            .set_recv_buffer_size(4096)
            .unwrap();
        let mut requests = Vec::with_capacity(64 * 2048);
        for seed in 0..2048 {
            requests.extend_from_slice(&request(seed, 1));
        }
        slow.write_all(&requests).unwrap();
        // Freeze this writer after a real Pending, without blocking the I/O
        // owner. A historical send_waits count alone is not a stable snapshot.
        paused.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(server.engine.stats.send_waits.load(SeqCst) > 0);
        assert_eq!(server.engine.slots.available_permits(), 1);
        assert_eq!(server.engine.tails.held.load(SeqCst), 0);
        // A real CPU job on another connection must obtain that same slot
        // while the first socket remains backpressured.
        other.write_all(&request(0, 1)).unwrap();
        response(&mut other, 1_442_695_040_888_963_407);
        other.shutdown(Shutdown::Write).unwrap();
        eof(&mut other);
        release.add_permits(1);
        for seed in 0..2048 {
            response(&mut slow, churn(seed, 1));
        }
        slow.shutdown(Shutdown::Write).unwrap();
        eof(&mut slow);
        let engine = server.engine.clone();
        server.finish(Ok(()));
        finish_engine(&engine);
        assert_eq!(engine.tails.produced.load(SeqCst), 2049);
        assert_eq!(engine.tails.consumed.load(SeqCst), 2049);
        assert_eq!(engine.tails.discarded.load(SeqCst), 0);
    }
}
