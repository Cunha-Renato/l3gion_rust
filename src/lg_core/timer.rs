use super::lg_types::units_of_time::{AsLgTime, LgTime};

pub struct Timer {
    begin: std::time::Instant,
}
impl Timer {
    pub fn new() -> Self {
        Self {
            begin: std::time::Instant::now(),
        }
    }
    pub fn restart(&mut self) {
        self.begin = std::time::Instant::now();
    }
    pub fn elapsed(&self) -> LgTime {
        self.begin.elapsed().as_nanos().ns()
    }
}