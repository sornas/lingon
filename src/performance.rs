use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use std::time::Instant;
use std::borrow::BorrowMut;

lazy_static! {
    pub static ref PERF_COUNTER: Arc<Mutex<Collector>> = Arc::new(Mutex::new(Collector::new()));
}

pub struct Marker {
    start: Instant,
    id: usize,
}

impl Drop for Marker {
    fn drop(&mut self) {
        PERF_COUNTER.lock().unwrap().end(self)
    }
}

pub struct Counter {
    name: &'static str,
    file: &'static str,
    line: u32,

    total_calls: usize,
    calls_this_frame: usize,
    total_time: f64,
    time_this_frame: f64,
}

impl Counter {
    pub fn new(name: &'static str,
               file: &'static str,
               line: u32) -> Self {
        Self {
            name,
            file,
            line,

            total_calls: 0,
            calls_this_frame: 0,
            total_time: 0.0,
            time_this_frame: 0.0,
        }
    }

    fn add(&mut self, start: Instant) {
        self.total_calls += 1;
        self.calls_this_frame += 1;
        let time = Instant::now().duration_since(start).as_secs_f64();
        self.total_time += time;
        self.time_this_frame += time;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CaptureWindow {
    Nothing,
    Silent,
    CaptureFor(usize),
    LogEvery(usize),
    Everything,
}

impl CaptureWindow {
    fn should_log(&self, frame: usize) -> bool {
        match self {
            CaptureWindow::Everything | CaptureWindow::CaptureFor(_) => true,
            CaptureWindow::LogEvery(x) => (frame % x) == 0,
            _ => false,
        }
    }

    fn should_capture(&self) -> bool {
        !matches!(self, CaptureWindow::Nothing)
    }

    fn step(self) -> Self {
        match self {
            CaptureWindow::CaptureFor(0) => CaptureWindow::Nothing,
            CaptureWindow::CaptureFor(x) => CaptureWindow::CaptureFor(x - 1),
            x => x,
        }
    }
}

pub fn capture_for(window: CaptureWindow) {
    PERF_COUNTER.lock().unwrap().borrow_mut().window = window;
}

pub fn frame() {
    let mut counter = PERF_COUNTER.lock().unwrap();
    let counter = counter.borrow_mut();
    if counter.window.should_capture() {
        counter.frame();
    }
    if counter.window.should_log(counter.num_frames) {
        counter.log();
    }
}

#[macro_export]
macro_rules! counter {
    ( $name:expr ) => {
        {
            let info = lingon::performance::Counter::new(
                $name,
                std::file!(),
                std::line!(),
            );
            let counter = lingon_macro::perf_counter!();
            lingon::performance::PERF_COUNTER.lock().unwrap().start(counter, info)
        }
    };
}

pub struct Collector {
    counters: Vec<Option<Counter>>,
    window: CaptureWindow,

    start: Instant,
    num_frames: usize,
    last_time: f64,
    weighted_time: f64,
    total_time: f64,
    min_frame_time: f64,
    max_frame_time: f64,
}

impl Collector {
    fn new() -> Self {
        Self {
            counters: Vec::new(),

            window: CaptureWindow::LogEvery(100),
            start: Instant::now(),
            num_frames: 0,
            last_time: 0.0,
            weighted_time: 0.0,
            total_time: 0.0,
            min_frame_time: f64::MAX,
            max_frame_time: f64::MIN,
        }
    }

    pub fn start(&mut self, id: usize, counter: Counter) -> Marker {
        if self.counters.len() <= id {
            self.counters.resize_with(id + 1, || None);
        }

        if matches!(self.counters[id], None) {
            self.counters[id] = Some(counter);
        }
        Marker {
            id,
            start: Instant::now(),
        }
    }

    pub fn end(&mut self, marker: &mut Marker) {
        self.counters.get_mut(marker.id).unwrap().as_mut().unwrap().add(marker.start);
    }

    pub fn frame(&mut self) {
        let end = Instant::now();
        let frame_time = end.duration_since(self.start).as_secs_f64();

        self.window = self.window.step();
        self.start = end;
        self.num_frames += 1;
        self.total_time += frame_time;
        self.min_frame_time = frame_time.min(self.min_frame_time);
        self.max_frame_time = frame_time.max(self.max_frame_time);
        self.last_time = frame_time;

        let weighting = 0.8;
        self.weighted_time = self.weighted_time * (1.0 - weighting) + frame_time * weighting;
    }

    pub fn log(&mut self) {
        return;
        println!("PERFORMANCE: #{}\nthis: {:<5.5} wgh: {:<5.5} avg: {:<5.5} min: {:<5.5} max: {:<5.5}",
            self.num_frames,
            self.last_time,
            self.weighted_time,
            self.total_time / (self.num_frames as f64),
            self.min_frame_time,
            self.max_frame_time,
        );
        for counter in self.counters.iter().filter_map(|x| x.as_ref()) {
            println!(" {} ({}:{}) - {:<5.5} {:<5.5}",
                counter.name,
                counter.file,
                counter.line,
                counter.time_this_frame / (counter.calls_this_frame as f64),
                counter.total_time / (counter.total_calls as f64),
            )
        }
    }
}
