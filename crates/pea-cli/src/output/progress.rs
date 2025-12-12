//! Progress bar implementations for long-running operations.
//!
//! Provides progress indicators for operations like dependency resolution,
//! package downloads, and file extraction.

use std::time::{Duration, Instant};

/// Simple progress bar for terminal output
pub struct ProgressBar {
    total: u64,
    current: u64,
    start_time: Instant,
    last_update: Instant,
    message: String,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: u64, message: String) -> Self {
        let now = Instant::now();
        Self {
            total,
            current: 0,
            start_time: now,
            last_update: now,
            message,
        }
    }

    /// Update progress
    pub fn update(&mut self, current: u64) {
        self.current = current;
        let now = Instant::now();
        
        // Only update display every 100ms to avoid flickering
        if now.duration_since(self.last_update) > Duration::from_millis(100) {
            self.display();
            self.last_update = now;
        }
    }

    /// Increment progress by 1
    pub fn increment(&mut self) {
        self.update(self.current + 1);
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        self.display();
        println!(); // New line after completion
    }

    /// Display the current progress
    fn display(&self) {
        let percentage = if self.total > 0 {
            (self.current * 100) / self.total
        } else {
            0
        };

        let elapsed = self.start_time.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            self.current / elapsed.as_secs()
        } else {
            0
        };

        print!(
            "\r{} [{}/{}] {}% ({}/s)",
            self.message, self.current, self.total, percentage, rate
        );
        
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }
}