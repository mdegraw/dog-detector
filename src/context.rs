use tokio::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Event {
    Acknowledge,
    Stop,
    Pause,
    Start,
    StreamVideo,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PauseState {
    OledStreaming,
    Stopped,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DetectionState {
    Running,
    Stopped,
    Detected(Instant),
    Paused(PauseState),
}

#[derive(Debug)]
pub struct Context {
    pub delay_duration: Duration,
    pub state: DetectionState,
    pub detected_at: Option<Instant>,
    pub detected_count: u8,
    pub detected_threshold: u8,
}

impl Context {
    pub fn new(delay_duration: Duration, detected_threshold: u8) -> Self {
        Context {
            delay_duration,
            detected_threshold,
            detected_count: 0,
            state: DetectionState::Running,
            detected_at: None,
        }
    }

    pub fn next(&mut self, state: DetectionState) -> DetectionState {
        match state {
            DetectionState::Detected(instant) => {
                self.detected_count += 1;
                self.detected_at = Some(instant);

                if self.detected_count > self.detected_threshold {
                    self.state = DetectionState::Paused(PauseState::OledStreaming);
                } else {
                    self.state = DetectionState::Running
                }
            }
            // TODO: change this to stream state
            DetectionState::Paused(pause_state) => match pause_state {
                PauseState::OledStreaming => match self.detected_at {
                    Some(detected_at) => {
                        let now = Instant::now();

                        if now.duration_since(detected_at) > self.delay_duration {
                            self.state = DetectionState::Running;
                            self.detected_at = None;
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {
                self.state = state;
            }
        }
        self.state
    }
}
