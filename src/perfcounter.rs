#[derive(Debug)]
pub struct PerformanceCounter {
    start_of_frame: std::time::Instant,
    old_point: std::time::Instant,
    text: String,
    old_text: String,
    only_fps: bool,
}

impl PerformanceCounter {
    pub fn new() -> Self {
        Self {
            start_of_frame: std::time::Instant::now(),
            old_point: std::time::Instant::now(),
            text: String::new(),
            old_text: String::new(),
            only_fps: false,
        }
    }

    pub fn only_fps(&mut self, only_fps: bool) {
        self.only_fps = only_fps;
    }

    pub fn get_frametime(&self) -> u64 {
        (std::time::Instant::now() - self.start_of_frame).as_micros() as u64
    }

    pub fn start_of_frame(&mut self) {
        self.start_of_frame = std::time::Instant::now();
        self.old_point = self.start_of_frame;
        self.text.clear();
    }

    pub fn add_measurement(&mut self, label: &str) {
        if self.only_fps {
            return;
        }
        let now = std::time::Instant::now();
        let ms = (now - self.old_point).as_micros();
        self.text += &format!("{}: {}\n", label, ms);
        self.old_point = now;
    }

    pub fn discard_measurement(&mut self) {
        if self.only_fps {
            return;
        }
        self.old_point = std::time::Instant::now();
    }

    pub fn print(&mut self) {
        if !self.only_fps {
            self.text += &format!(
                "total: {}\n",
                (std::time::Instant::now() - self.start_of_frame).as_micros()
            );
        }
        self.text += &format!(
            "fps: {}",
            1_000_000 / (std::time::Instant::now() - self.start_of_frame).as_micros()
        );

        self.old_text = self.text.clone();
    }

    pub fn get_text(&self) -> &str {
        &self.old_text
    }
}
