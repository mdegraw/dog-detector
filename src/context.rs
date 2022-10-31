use tokio::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DetectionState {
    Running,
    StreamEnd,
    Detected(Instant),
    Paused(Instant),
    Streaming(Instant),
}

#[derive(Debug)]
pub struct Context {
    pub stream_duration: Duration,
    pub state: DetectionState,
    pub detected_count: u8,
    pub detected_threshold: u8,
    pub pause_duration: Duration,
}

impl Context {
    pub fn new(
        stream_duration: Duration,
        pause_duration: Duration,
        detected_threshold: u8,
    ) -> Self {
        Context {
            stream_duration,
            pause_duration,
            detected_threshold,
            detected_count: 0,
            state: DetectionState::Running,
        }
    }

    pub fn is_detected(&self) -> bool {
        self.detected_count >= self.detected_threshold
    }

    pub fn next(&mut self) -> DetectionState {
        match self.state {
            DetectionState::Detected(instant) => {
                self.detected_count += 1;

                if self.detected_count > self.detected_threshold {
                    self.detected_count = 0;
                    self.state = DetectionState::Streaming(instant);
                } else {
                    self.state = DetectionState::Running
                }
            }
            DetectionState::Streaming(detected_at) => {
                let now = Instant::now();

                if now.duration_since(detected_at) > self.stream_duration {
                    self.state = DetectionState::StreamEnd;
                }
            }
            DetectionState::Paused(paused_at) => {
                let now = Instant::now();

                if now.duration_since(paused_at) > self.pause_duration {
                    self.state = DetectionState::Running;
                }
            }
            DetectionState::StreamEnd => {
                let now = Instant::now();
                self.state = DetectionState::Paused(now);
            }
            _ => {}
        }
        self.state
    }
}
