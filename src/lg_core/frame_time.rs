use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use crate::{profile_function, StdError};
use super::{lg_types::units_of_time::{AsLgTime, LgTime}, timer::LgTimer};

static FRAME_TIME: OnceLock<Arc<Mutex<FrameTime>>> = OnceLock::new();

/// Time is stored in SECONDS.
pub struct FrameTime {
    current: LgTime,
    timer: LgTimer
}
// Public
impl FrameTime {
    pub fn value() -> Result<LgTime, StdError> {
        Ok(Self::get_locked()?.current)
    }
}
// Public(crate)
impl FrameTime {
    pub(crate) fn init() -> Result<(), StdError> {
        match FRAME_TIME.set(Arc::new(Mutex::new(FrameTime {
            current: 16.6.ms(),
            timer: LgTimer::new(),
        }))) {
            Err(_) => Err("Failed to create FrameTime!".into()),
            _ => Ok(())
        }
    }
    pub(crate) fn start() -> Result<(), StdError> {
        profile_function!();
        Self::get_locked()?.timer.restart();

        Ok(())
    }
    pub(crate) fn end() -> Result<(), StdError> {
        profile_function!();
        let mut ft = Self::get_locked()?;
        ft.current = ft.timer.elapsed();
        
        Ok(())
    }
}
// Private
impl FrameTime {
    // Btw I don't like this ok, but I want a singleton because passing them to every other object is a pain and ugly
    fn get_locked() -> Result<MutexGuard<'static, FrameTime>, StdError> {
        FRAME_TIME
            .get().ok_or("Failed to get FrameTime! (Maybe it wasn't initialized)")?
            .lock().or(Err("Failed to lock FrameTime".into()))
    }
}