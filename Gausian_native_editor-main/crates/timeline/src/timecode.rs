/// Professional timecode display system with drop-frame support
/// Phase 1: Timeline Polish & UX - Complete Implementation

use crate::{Frame, Fps};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Timecode format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimecodeFormat {
    /// Non-drop frame (HH:MM:SS:FF)
    NonDropFrame,
    /// Drop frame (HH:MM:SS;FF) - Used for 29.97, 59.94 fps
    DropFrame,
    /// Seconds with decimals (SS.mmm)
    Seconds,
    /// Frames (FFFFF)
    Frames,
}

impl Default for TimecodeFormat {
    fn default() -> Self {
        Self::NonDropFrame
    }
}

/// Timecode representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timecode {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
    pub format: TimecodeFormat,
}

impl Timecode {
    /// Create timecode from frame number
    pub fn from_frame(frame: Frame, fps: Fps, format: TimecodeFormat) -> Self {
        let frame = frame.max(0) as u64;
        let fps_f64 = fps.num as f64 / fps.den as f64;
        let fps_rounded = fps_f64.round() as u64;

        match format {
            TimecodeFormat::DropFrame => {
                // Drop frame calculation for 29.97 and 59.94 fps
                // Drop 2 frames at the start of every minute except every 10th minute
                let is_drop_fps = (fps.num == 30000 && fps.den == 1001)
                    || (fps.num == 60000 && fps.den == 1001);

                if is_drop_fps {
                    let drop_frames = if fps.num == 30000 { 2 } else { 4 };

                    // Calculate total minutes
                    let frames_per_10min = fps_rounded * 60 * 10 - drop_frames * 9;
                    let ten_minute_blocks = frame / frames_per_10min;
                    let remainder = frame % frames_per_10min;

                    // Frames in first minute of each 10-minute block don't have drops
                    let frames_first_minute = fps_rounded * 60;

                    let (minutes_in_block, seconds, frames) = if remainder < frames_first_minute {
                        let seconds = remainder / fps_rounded;
                        let frames = remainder % fps_rounded;
                        (0, seconds as u32, frames as u32)
                    } else {
                        let remainder_after_first_min = remainder - frames_first_minute;
                        let frames_per_normal_min = fps_rounded * 60 - drop_frames;
                        let additional_minutes = 1 + (remainder_after_first_min / frames_per_normal_min);
                        let frames_in_current_min =
                            remainder_after_first_min % frames_per_normal_min;

                        // Add back dropped frames for display
                        let adjusted_frames = frames_in_current_min + drop_frames;
                        let seconds = adjusted_frames / fps_rounded;
                        let frames = adjusted_frames % fps_rounded;

                        (additional_minutes as u32, seconds as u32, frames as u32)
                    };

                    let total_minutes = (ten_minute_blocks * 10 + minutes_in_block as u64) as u32;
                    let hours = total_minutes / 60;
                    let minutes = total_minutes % 60;

                    Self {
                        hours,
                        minutes,
                        seconds,
                        frames,
                        format,
                    }
                } else {
                    // Not a drop frame rate, use normal calculation
                    Self::from_frame_non_drop(frame, fps_rounded, format)
                }
            }
            _ => Self::from_frame_non_drop(frame, fps_rounded, format),
        }
    }

    fn from_frame_non_drop(frame: u64, fps: u64, format: TimecodeFormat) -> Self {
        let total_seconds = frame / fps;
        let frames = (frame % fps) as u32;

        let hours = (total_seconds / 3600) as u32;
        let minutes = ((total_seconds % 3600) / 60) as u32;
        let seconds = (total_seconds % 60) as u32;

        Self {
            hours,
            minutes,
            seconds,
            frames,
            format,
        }
    }

    /// Convert timecode back to frame number
    pub fn to_frame(&self, fps: Fps) -> Frame {
        let fps_f64 = fps.num as f64 / fps.den as f64;
        let fps_rounded = fps_f64.round() as i64;

        match self.format {
            TimecodeFormat::DropFrame => {
                let is_drop_fps = (fps.num == 30000 && fps.den == 1001)
                    || (fps.num == 60000 && fps.den == 1001);

                if is_drop_fps {
                    let drop_frames = if fps.num == 30000 { 2 } else { 4 };

                    let total_minutes = self.hours as i64 * 60 + self.minutes as i64;
                    let ten_minute_blocks = total_minutes / 10;
                    let remaining_minutes = total_minutes % 10;

                    // Calculate frames
                    let mut frame = ten_minute_blocks * (fps_rounded * 600 - drop_frames * 9);

                    if remaining_minutes > 0 {
                        // First minute of the block (no drop)
                        frame += fps_rounded * 60;

                        // Remaining minutes (with drops)
                        if remaining_minutes > 1 {
                            frame += (remaining_minutes - 1) * (fps_rounded * 60 - drop_frames);
                        }
                    }

                    // Add seconds and frames
                    frame += self.seconds as i64 * fps_rounded;
                    frame += self.frames as i64;

                    frame
                } else {
                    self.to_frame_non_drop(fps_rounded)
                }
            }
            _ => self.to_frame_non_drop(fps_rounded),
        }
    }

    fn to_frame_non_drop(&self, fps: i64) -> Frame {
        let total_seconds = self.hours as i64 * 3600 + self.minutes as i64 * 60 + self.seconds as i64;
        total_seconds * fps + self.frames as i64
    }

    /// Parse timecode string (HH:MM:SS:FF or HH:MM:SS;FF)
    pub fn parse(s: &str, fps: Fps) -> Result<Self, String> {
        let is_drop = s.contains(';');
        let format = if is_drop {
            TimecodeFormat::DropFrame
        } else {
            TimecodeFormat::NonDropFrame
        };

        let parts: Vec<&str> = s.split(&[':', ';'][..]).collect();

        if parts.len() != 4 {
            return Err("Invalid timecode format. Expected HH:MM:SS:FF".to_string());
        }

        let hours = parts[0]
            .parse::<u32>()
            .map_err(|_| "Invalid hours".to_string())?;
        let minutes = parts[1]
            .parse::<u32>()
            .map_err(|_| "Invalid minutes".to_string())?;
        let seconds = parts[2]
            .parse::<u32>()
            .map_err(|_| "Invalid seconds".to_string())?;
        let frames = parts[3]
            .parse::<u32>()
            .map_err(|_| "Invalid frames".to_string())?;

        if minutes >= 60 || seconds >= 60 {
            return Err("Minutes and seconds must be < 60".to_string());
        }

        let fps_val = (fps.num as f64 / fps.den as f64).round() as u32;
        if frames >= fps_val {
            return Err(format!("Frames must be < {}", fps_val));
        }

        Ok(Self {
            hours,
            minutes,
            seconds,
            frames,
            format,
        })
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        match self.format {
            TimecodeFormat::NonDropFrame => {
                format!(
                    "{:02}:{:02}:{:02}:{:02}",
                    self.hours, self.minutes, self.seconds, self.frames
                )
            }
            TimecodeFormat::DropFrame => {
                format!(
                    "{:02}:{:02}:{:02};{:02}",
                    self.hours, self.minutes, self.seconds, self.frames
                )
            }
            TimecodeFormat::Seconds => {
                let total_seconds =
                    self.hours as f64 * 3600.0 + self.minutes as f64 * 60.0 + self.seconds as f64;
                format!("{:.3}", total_seconds)
            }
            TimecodeFormat::Frames => {
                format!("{:05}", self.frames)
            }
        }
    }
}

impl fmt::Display for Timecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Timecode calculator for timeline operations
pub struct TimecodeCalculator {
    pub fps: Fps,
    pub format: TimecodeFormat,
}

impl TimecodeCalculator {
    pub fn new(fps: Fps, format: TimecodeFormat) -> Self {
        Self { fps, format }
    }

    /// Convert frame to timecode string
    pub fn frame_to_timecode(&self, frame: Frame) -> String {
        Timecode::from_frame(frame, self.fps, self.format).to_string()
    }

    /// Convert timecode string to frame
    pub fn timecode_to_frame(&self, timecode: &str) -> Result<Frame, String> {
        let tc = Timecode::parse(timecode, self.fps)?;
        Ok(tc.to_frame(self.fps))
    }

    /// Get duration string
    pub fn duration_string(&self, frames: Frame) -> String {
        self.frame_to_timecode(frames)
    }

    /// Check if fps requires drop frame
    pub fn should_use_drop_frame(&self) -> bool {
        (self.fps.num == 30000 && self.fps.den == 1001)
            || (self.fps.num == 60000 && self.fps.den == 1001)
    }

    /// Get recommended format for current fps
    pub fn recommended_format(&self) -> TimecodeFormat {
        if self.should_use_drop_frame() {
            TimecodeFormat::DropFrame
        } else {
            TimecodeFormat::NonDropFrame
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timecode_non_drop_24fps() {
        let fps = Fps { num: 24, den: 1 };
        let tc = Timecode::from_frame(0, fps, TimecodeFormat::NonDropFrame);
        assert_eq!(tc.to_string(), "00:00:00:00");

        let tc = Timecode::from_frame(24, fps, TimecodeFormat::NonDropFrame);
        assert_eq!(tc.to_string(), "00:00:01:00");

        let tc = Timecode::from_frame(1440, fps, TimecodeFormat::NonDropFrame); // 60 seconds
        assert_eq!(tc.to_string(), "00:01:00:00");

        let tc = Timecode::from_frame(86400, fps, TimecodeFormat::NonDropFrame); // 1 hour
        assert_eq!(tc.to_string(), "01:00:00:00");
    }

    #[test]
    fn test_timecode_drop_frame_2997() {
        let fps = Fps {
            num: 30000,
            den: 1001,
        };

        // Frame 0
        let tc = Timecode::from_frame(0, fps, TimecodeFormat::DropFrame);
        assert_eq!(tc.to_string(), "00:00:00;00");

        // Frame 1800 (should be around 1 minute in drop frame)
        let tc = Timecode::from_frame(1800, fps, TimecodeFormat::DropFrame);
        // In drop frame, we skip frames 0 and 1 at the start of each minute (except every 10th)
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 1);
    }

    #[test]
    fn test_timecode_parse() {
        let fps = Fps { num: 24, den: 1 };

        let tc = Timecode::parse("00:00:01:00", fps).unwrap();
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 0);
        assert_eq!(tc.seconds, 1);
        assert_eq!(tc.frames, 0);

        let tc = Timecode::parse("01:23:45:12", fps).unwrap();
        assert_eq!(tc.hours, 1);
        assert_eq!(tc.minutes, 23);
        assert_eq!(tc.seconds, 45);
        assert_eq!(tc.frames, 12);
    }

    #[test]
    fn test_timecode_round_trip() {
        let fps = Fps { num: 24, den: 1 };
        let frame = 12345;

        let tc = Timecode::from_frame(frame, fps, TimecodeFormat::NonDropFrame);
        let back = tc.to_frame(fps);

        assert_eq!(frame as i64, back);
    }

    #[test]
    fn test_timecode_calculator() {
        let fps = Fps { num: 24, den: 1 };
        let calc = TimecodeCalculator::new(fps, TimecodeFormat::NonDropFrame);

        assert_eq!(calc.frame_to_timecode(0), "00:00:00:00");
        assert_eq!(calc.frame_to_timecode(24), "00:00:01:00");

        let frame = calc.timecode_to_frame("00:00:01:00").unwrap();
        assert_eq!(frame, 24);
    }
}
