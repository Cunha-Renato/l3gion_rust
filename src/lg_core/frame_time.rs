use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use crate::StdError;

use super::lg_types::units_of_time::{AsLgTime, LgTime};

static FRAME_TIME: OnceLock<Arc<Mutex<FrameTime>>> = OnceLock::new();

/// Time is stored in SECONDS.
#[derive(Debug)]
pub struct FrameTime {
    current: LgTime,
    begin: std::time::Instant,
}
impl FrameTime {
    pub fn elapsed() -> Result<LgTime, StdError> {
        Ok(Self::get()?.current)
    }

    pub(crate) fn init() -> Result<(), StdError> {
        match FRAME_TIME.set(Arc::new(Mutex::new(FrameTime {
            current: 16.6.ms(),
            begin: std::time::Instant::now(),
        }))) {
            Err(_) => Err("Failed to create FrameTime!".into()),
            _ => Ok(())
        }
    }
    pub(crate) fn start() -> Result<(), StdError> {
        Self::get()?.begin = std::time::Instant::now();

        Ok(())
    }
    pub(crate) fn end() -> Result<(), StdError> {
        let mut ft = Self::get()?;
        ft.current = ft.begin.elapsed().as_nanos().ns();
        
        Ok(())
    }
    fn get() -> Result<MutexGuard<'static, FrameTime>, StdError> {
        FRAME_TIME
            .get().ok_or("Failed to get FrameTime! (Maybe it wasn't initialized)")?
            .lock().or(Err("Failed to lock FrameTime".into()))
            
    }
}