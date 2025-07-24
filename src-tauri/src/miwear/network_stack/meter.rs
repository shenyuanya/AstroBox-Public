use std::{collections::VecDeque, sync::{Arc, Mutex}, time::{Duration, Instant}};

#[derive(Clone)]
pub struct BandwidthMeter {
    window: Duration,
    write_events: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    read_events:  Arc<Mutex<VecDeque<(Instant, u64)>>>,
}

impl BandwidthMeter {

    fn evict_old(&self, q: &mut VecDeque<(Instant, u64)>, now: Instant) {
        while let Some(&(t, _)) = q.front() {
            if now - t > self.window {
                q.pop_front();
            } else {
                break;
            }
        }
    }

    fn push_event(&self, que: &Arc<Mutex<VecDeque<(Instant, u64)>>>, length: u64) {
        let now = Instant::now();
        let mut que = que.lock().unwrap();
        que.push_back((now, length));
        self.evict_old(&mut que, now);
    }

    pub fn new(window: Duration) -> Self {
        Self {
            window,
            write_events: Arc::new(Mutex::new(VecDeque::new())),
            read_events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn speed_inner(&self, que: &Arc<Mutex<VecDeque<(Instant, u64)>>>) -> f64 {
        let now = Instant::now();
        let mut que = que.lock().unwrap();
        self.evict_old(&mut que, now);
        let total: u64 = que.iter().map(|(_, b)| b).sum();
        if total == 0 {
            0.0
        } else if let Some(&(first_t, _)) = que.front() {
            let actual = (now - first_t).as_secs_f64().max(0.001);
            total as f64 / actual
        } else {
            0.0
        }
    }

    pub fn add_written(&self, length: usize) {
        self.push_event(&self.write_events, length as u64);
    }

    pub fn add_read(&self, length: usize) {
        self.push_event(&self.read_events, length as u64);
    }

    pub fn write_speed(&self) -> f64 {
        self.speed_inner(&self.write_events)
    }

    pub fn read_speed(&self) -> f64 {
        self.speed_inner(&self.read_events)
    }

}