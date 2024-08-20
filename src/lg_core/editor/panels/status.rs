use crate::lg_core::frame_time::FrameTime;

const RESOLUTION: usize = 75;

pub(crate) struct FrameTimeTracker {
    pub(crate) average: f64,
    pub(crate) highest: f64,
    pub(crate) current: f64,

    frame_times: [f64; RESOLUTION],
    index: usize,
}
impl FrameTimeTracker {
    pub(crate) const fn new() -> Self {
        Self {
            average: 0.0,
            highest: 0.0,
            current: 0.0,

            frame_times: [16.6; RESOLUTION],
            index: 0,
        }
    }
    
    pub(crate) fn new_frame(&mut self, frame_time: f64) {
        self.current = frame_time;

        if self.index < RESOLUTION {
            self.frame_times[self.index] = frame_time;

            self.index += 1;
        }
        else {
            self.calculate();
            self.index = 0;
        }
    }
    
    fn calculate(&mut self) {
        let mut highest = 0.0;
        self.average = 0.0;

        for frame in &self.frame_times {
            if *frame > highest {
                highest = *frame;
            }
            
            self.average += *frame;                
        }
        self.highest = highest;
        self.average /= self.index as f64;
    }
}

static mut FRAME_TIME_TRACKER: FrameTimeTracker = FrameTimeTracker::new();

pub(crate) fn imgui_status_panel(ui: &mut imgui::Ui) {
    ui.window("Status").size([100.0, 100.0], imgui::Condition::FirstUseEver).build(|| {
        imgui_frame_time(ui);
    });
}

fn imgui_frame_time(ui: &imgui::Ui) {
    let frame_time = FrameTime::value().unwrap();
    let ft = frame_time.ms().value();
    let a_ft;
    let h_ft;

    let fps;
    let a_fps;
    let l_fps;

    unsafe {
        FRAME_TIME_TRACKER.new_frame(ft);

        a_ft = FRAME_TIME_TRACKER.average;
        h_ft = FRAME_TIME_TRACKER.highest;
        
        fps = (1000.0 / ft) as usize;
        a_fps = (1000.0 / a_ft) as usize;
        l_fps = (1000.0 / h_ft) as usize;
    }

    if ui.collapsing_header("Frame Time", imgui::TreeNodeFlags::DEFAULT_OPEN) {
        ui.text(std::format!("FPS"));
        ui.text(std::format!("Current: {fps}"));
        ui.text(std::format!("Average: {a_fps}"));
        ui.text(std::format!("Lowest:  {l_fps}"));
        ui.spacing();
        ui.separator();
        ui.spacing();
        ui.text(std::format!("FRAME TIME"));
        ui.text(std::format!("Current: {ft:.1} ms"));
        ui.text(std::format!("Average: {a_ft:.1} ms"));
        ui.text(std::format!("Highest: {h_ft:.1} ms"));
    }
}