//! Clock-injected step playback for B - Estela (RF-36).

use std::time::Duration;

/// One observation every 700 ms, not a simulation of movement speed.
const STEP: Duration = Duration::from_millis(700);

/// Session-only player. The caller pauses it when the map is hidden or the guild changes.
#[derive(Debug, Default)]
pub struct Playback {
    pub selected: usize,
    pub playing: bool,
    last_tick: Option<Duration>,
}

impl Playback {
    /// Selects the latest observation of a new guild and stops playback.
    pub fn reset(&mut self, count: usize) {
        self.selected = count.saturating_sub(1);
        self.pause();
    }

    /// Pauses without losing the selected claim or counting elapsed time while hidden.
    pub fn pause(&mut self) {
        self.playing = false;
        self.last_tick = None;
    }

    /// Selects an existing step, clamping after history eviction, and pauses.
    pub fn choose(&mut self, index: usize, count: usize) {
        self.selected = index.min(count.saturating_sub(1));
        self.pause();
    }

    /// Starts/resumes, or pauses. Play at the end restarts at the first observation.
    pub fn toggle(&mut self, now: Duration, count: usize) {
        if self.playing || count < 2 {
            self.pause();
            return;
        }
        if self.selected >= count - 1 {
            self.selected = 0;
        }
        self.playing = true;
        self.last_tick = Some(now);
    }

    /// Advances with an injected monotonic clock, preserving sub-step remainder.
    pub fn tick(&mut self, now: Duration, count: usize) {
        if count < 2 {
            self.reset(count);
            return;
        }
        self.selected = self.selected.min(count - 1);
        let Some(last) = self.last_tick else {
            return;
        };
        let steps = (now.saturating_sub(last).as_millis() / STEP.as_millis())
            .min((count - 1 - self.selected) as u128) as usize;
        self.selected += steps;
        self.last_tick = Some(last + STEP * steps as u32);
        if self.selected == count - 1 {
            self.pause();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rf36_estela_playback_advances_at_700ms_and_stops_at_the_end() {
        let mut p = Playback::default();
        p.reset(4);
        assert_eq!(p.selected, 3);
        p.toggle(Duration::ZERO, 4);
        assert_eq!(p.selected, 0);
        p.tick(Duration::from_millis(699), 4);
        assert_eq!(p.selected, 0);
        p.tick(Duration::from_millis(700), 4);
        assert_eq!(p.selected, 1);
        p.tick(Duration::from_millis(2100), 4);
        assert_eq!(p.selected, 3);
        assert!(!p.playing);
        p.toggle(Duration::from_secs(3), 4);
        assert_eq!(p.selected, 0);
    }

    #[test]
    fn rf36_estela_pause_resume_and_scrubbing_do_not_count_hidden_time() {
        let mut p = Playback::default();
        p.reset(5);
        p.toggle(Duration::ZERO, 5);
        p.tick(Duration::from_millis(700), 5);
        p.pause();
        p.tick(Duration::from_secs(100), 5);
        assert_eq!(p.selected, 1);
        p.toggle(Duration::from_secs(100), 5);
        p.tick(Duration::from_millis(100699), 5);
        assert_eq!(p.selected, 1);
        p.tick(Duration::from_millis(100700), 5);
        assert_eq!(p.selected, 2);
        p.choose(usize::MAX, 5);
        assert_eq!(p.selected, 4);
        assert!(!p.playing);
        p.reset(0);
        p.toggle(Duration::from_secs(101), 0);
        p.tick(Duration::MAX, 0);
        assert_eq!(p.selected, 0);
        assert!(!p.playing);
        p.reset(1);
        p.toggle(Duration::ZERO, 1);
        assert!(!p.playing);
    }
}
