use std::collections::VecDeque;

pub const DEFAULT_HISTORY_SECONDS: u32 = 5;
pub const MIN_HISTORY_SECONDS: u32 = 1;
pub const MAX_HISTORY_SECONDS: u32 = 30;
pub const TVC_FRAMES_PER_SECOND: usize = 50;
const THUMBNAIL_MAX_WIDTH: usize = 160;

#[derive(Clone)]
pub struct FrameThumbnail {
    pub pixels: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

impl FrameThumbnail {
    pub fn from_framebuffer(pixels: &[u32], width: usize, height: usize) -> Self {
        if width == 0 || height == 0 || pixels.len() < width.saturating_mul(height) {
            return Self {
                pixels: Vec::new(),
                width: 0,
                height: 0,
            };
        }

        let step = width.div_ceil(THUMBNAIL_MAX_WIDTH).max(1);
        let thumbnail_width = width.div_ceil(step);
        let thumbnail_height = height.div_ceil(step);
        let mut thumbnail = Vec::with_capacity(thumbnail_width * thumbnail_height);
        for y in (0..height).step_by(step) {
            for x in (0..width).step_by(step) {
                thumbnail.push(pixels[y * width + x]);
            }
        }
        Self {
            pixels: thumbnail,
            width: thumbnail_width,
            height: thumbnail_height,
        }
    }

    fn byte_len(&self) -> usize {
        self.pixels.len() * std::mem::size_of::<u32>()
    }
}

pub struct FrameRecord {
    pub id: u64,
    pub snapshot: Vec<u8>,
    pub thumbnail: FrameThumbnail,
    pub frame_number: u64,
    pub pc: u16,
}

impl FrameRecord {
    fn byte_len(&self) -> usize {
        self.snapshot.len() + self.thumbnail.byte_len()
    }
}

pub struct FrameHistory {
    frames: VecDeque<FrameRecord>,
    selected: Option<usize>,
    recording: bool,
    duration_seconds: u32,
    next_id: u64,
    next_frame_number: u64,
}

impl Default for FrameHistory {
    fn default() -> Self {
        Self {
            frames: VecDeque::new(),
            selected: None,
            recording: false,
            duration_seconds: DEFAULT_HISTORY_SECONDS,
            next_id: 1,
            next_frame_number: 0,
        }
    }
}

impl FrameHistory {
    pub fn start(&mut self) {
        self.clear();
        self.recording = true;
    }

    pub fn stop(&mut self) {
        self.recording = false;
    }

    pub fn clear(&mut self) {
        self.frames.clear();
        self.selected = None;
        self.next_frame_number = 0;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn duration_seconds(&self) -> u32 {
        self.duration_seconds
    }

    pub fn set_duration_seconds(&mut self, seconds: u32) {
        self.duration_seconds = seconds.clamp(MIN_HISTORY_SECONDS, MAX_HISTORY_SECONDS);
        self.enforce_capacity();
    }

    pub fn capacity(&self) -> usize {
        self.duration_seconds as usize * TVC_FRAMES_PER_SECOND
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn frames(&self) -> &VecDeque<FrameRecord> {
        &self.frames
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected(&self) -> Option<&FrameRecord> {
        self.selected.and_then(|index| self.frames.get(index))
    }

    pub fn selected_offset(&self) -> Option<isize> {
        let selected = self.selected?;
        Some(selected as isize - self.frames.len() as isize + 1)
    }

    pub fn memory_bytes(&self) -> usize {
        self.frames.iter().map(FrameRecord::byte_len).sum()
    }

    pub fn record_frame(&mut self, snapshot: Vec<u8>, thumbnail: FrameThumbnail, pc: u16) -> bool {
        if !self.recording {
            return false;
        }

        self.truncate_future();
        let record = FrameRecord {
            id: self.next_id,
            snapshot,
            thumbnail,
            frame_number: self.next_frame_number,
            pc,
        };
        self.next_id = self.next_id.wrapping_add(1);
        self.next_frame_number = self.next_frame_number.wrapping_add(1);
        self.frames.push_back(record);
        self.selected = Some(self.frames.len() - 1);
        self.enforce_capacity();
        true
    }

    pub fn select_previous(&mut self) -> bool {
        let Some(current) = self.selected.or_else(|| self.frames.len().checked_sub(1)) else {
            return false;
        };
        if current == 0 {
            return false;
        }
        self.selected = Some(current - 1);
        true
    }

    pub fn select_next(&mut self) -> bool {
        let Some(current) = self.selected else {
            return false;
        };
        if current + 1 >= self.frames.len() {
            return false;
        }
        self.selected = Some(current + 1);
        true
    }

    pub fn select_latest(&mut self) -> bool {
        let Some(latest) = self.frames.len().checked_sub(1) else {
            return false;
        };
        let changed = self.selected != Some(latest);
        self.selected = Some(latest);
        changed
    }

    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.frames.len() || self.selected == Some(index) {
            return false;
        }
        self.selected = Some(index);
        true
    }

    pub fn branch_from_selected(&mut self) -> bool {
        let old_len = self.frames.len();
        self.truncate_future();
        if !self.frames.is_empty() {
            self.selected = Some(self.frames.len() - 1);
        }
        self.frames.len() != old_len
    }

    fn truncate_future(&mut self) {
        if let Some(selected) = self.selected {
            self.frames.truncate(selected + 1);
        }
    }

    fn enforce_capacity(&mut self) {
        let excess = self.frames.len().saturating_sub(self.capacity());
        if excess == 0 {
            return;
        }
        self.frames.drain(..excess);
        self.selected = self
            .selected
            .map(|selected| selected.saturating_sub(excess));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(history: &mut FrameHistory, value: u8) {
        assert!(history.record_frame(
            vec![value],
            FrameThumbnail {
                pixels: vec![value as u32],
                width: 1,
                height: 1,
            },
            value as u16,
        ));
    }

    #[test]
    fn recording_is_bounded_and_evicts_oldest_frames() {
        let mut history = FrameHistory::default();
        history.start();
        history.duration_seconds = 1;
        for value in 0..=TVC_FRAMES_PER_SECOND as u8 {
            record(&mut history, value);
        }

        assert_eq!(history.len(), TVC_FRAMES_PER_SECOND);
        assert_eq!(history.frames.front().unwrap().snapshot, vec![1]);
        assert_eq!(history.selected_offset(), Some(0));
    }

    #[test]
    fn navigation_reports_offsets_from_latest_frame() {
        let mut history = FrameHistory::default();
        history.start();
        for value in 0..3 {
            record(&mut history, value);
        }

        assert_eq!(history.selected_offset(), Some(0));
        assert!(history.select_previous());
        assert_eq!(history.selected_offset(), Some(-1));
        assert!(history.select_previous());
        assert_eq!(history.selected_offset(), Some(-2));
        assert!(!history.select_previous());
        assert!(history.select_next());
        assert!(history.select_latest());
        assert_eq!(history.selected_offset(), Some(0));
    }

    #[test]
    fn recording_after_rewind_discards_future_branch() {
        let mut history = FrameHistory::default();
        history.start();
        for value in 0..4 {
            record(&mut history, value);
        }
        assert!(history.select_previous());
        assert!(history.select_previous());

        record(&mut history, 9);

        let values: Vec<u8> = history
            .frames()
            .iter()
            .map(|frame| frame.snapshot[0])
            .collect();
        assert_eq!(values, vec![0, 1, 9]);
        assert_eq!(history.selected_offset(), Some(0));
    }

    #[test]
    fn explicit_branch_discards_future_before_instruction_step() {
        let mut history = FrameHistory::default();
        history.start();
        for value in 0..4 {
            record(&mut history, value);
        }
        assert!(history.select_previous());
        assert!(history.select_previous());

        assert!(history.branch_from_selected());

        assert_eq!(history.len(), 2);
        assert_eq!(history.selected_offset(), Some(0));
        assert_eq!(history.selected().unwrap().snapshot, vec![1]);
    }

    #[test]
    fn duration_resize_trims_existing_history() {
        let mut history = FrameHistory::default();
        history.start();
        for value in 0..75 {
            record(&mut history, value);
        }

        history.set_duration_seconds(1);

        assert_eq!(history.len(), 50);
        assert_eq!(history.frames.front().unwrap().snapshot, vec![25]);
    }

    #[test]
    fn thumbnail_downsamples_without_changing_pixel_order() {
        let width = 320;
        let height = 4;
        let pixels: Vec<u32> = (0..width * height).map(|value| value as u32).collect();

        let thumbnail = FrameThumbnail::from_framebuffer(&pixels, width, height);

        assert_eq!((thumbnail.width, thumbnail.height), (160, 2));
        assert_eq!(thumbnail.pixels[0], 0);
        assert_eq!(thumbnail.pixels[1], 2);
        assert_eq!(thumbnail.pixels[160], 640);
    }
}
