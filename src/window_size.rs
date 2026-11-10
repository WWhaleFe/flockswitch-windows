//! Keeps a window at its size in points when it moves to a monitor with other scaling.
//!
//! Why this is needed: eframe (egui-winit's `apply_viewport_builder_to_window`) turns
//! the window's min and max inner size into *physical pixels* with the scaling of the
//! monitor the window opens on, and winit keeps them as pixels. When the window is moved
//! to a monitor with other scaling, Windows sends WM_DPICHANGED and winit resizes the
//! window to keep its size in points, but its WM_GETMINMAXINFO still answers with the old
//! pixel limits. Opened at 150% and moved to 100%, the 760-point minimum is still 1140
//! pixels, now 1140 points, so the window jumps far too large (and the other way round
//! the old maximum can make it too small).
//!
//! The fix, decided here and carried out by the settings window: when the scaling
//! changes, send the min and max size again (egui converts them with the new scaling) and
//! put the window back to the size it had, a few times until it sticks. Only sizes the
//! user made while the scaling stayed the same are remembered.

/// A size in points, [width, height].
pub type Size = [f32; 2];

/// Frames in which to ask again for the size, while the window settles.
const TRIES: u8 = 12;
/// Closer than this (points) is the size asked for.
const TOLERANCE: f32 = 2.0;

/// What to tell the window this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fix {
    /// Send the min and max size again: the scaling changed.
    pub limits: bool,
    /// Ask for this inner size.
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct SizeKeeper {
    min: Size,
    max: Size,
    /// The monitor scaling (native pixels per point) last seen.
    scale: Option<f32>,
    /// The size last seen while the scaling stayed the same.
    good: Option<Size>,
    /// The size being put back, and the frames left to try.
    restoring: Option<(Size, u8)>,
}

impl SizeKeeper {
    pub fn new(min: Size, max: Size) -> SizeKeeper {
        SizeKeeper { min, max, scale: None, good: None, restoring: None }
    }

    /// A window that keeps one size.
    pub fn fixed(size: Size) -> SizeKeeper {
        SizeKeeper::new(size, size)
    }

    /// The smallest and largest size.
    pub fn limits(&self) -> (Size, Size) {
        (self.min, self.max)
    }

    pub fn clamp(&self, size: Size) -> Size {
        [size[0].clamp(self.min[0], self.max[0]), size[1].clamp(self.min[1], self.max[1])]
    }

    fn within(&self, size: Size) -> bool {
        (0..2).all(|i| size[i] >= self.min[i] - TOLERANCE && size[i] <= self.max[i] + TOLERANCE)
    }

    /// Still putting the size back (the window should be redrawn soon).
    pub fn busy(&self) -> bool {
        self.restoring.is_some()
    }

    /// Called every frame with the monitor scaling and the window's inner size in points.
    pub fn step(&mut self, scale: f32, size: Size) -> Option<Fix> {
        let changed = self.scale.is_some_and(|s| (s - scale).abs() > 0.001);
        self.scale = Some(scale);
        if changed {
            let target = self.clamp(self.good.unwrap_or(size));
            self.restoring = Some((target, TRIES));
            return Some(Fix { limits: true, size: target });
        }
        if let Some((target, left)) = self.restoring {
            if close(size, target) || left == 0 {
                self.restoring = None;
                if close(size, target) {
                    self.good = Some(size);
                }
                return None;
            }
            self.restoring = Some((target, left - 1));
            return Some(Fix { limits: false, size: target });
        }
        if !self.within(size) {
            // Outside the limits without a change of scaling: back inside them.
            let target = self.clamp(size);
            self.restoring = Some((target, TRIES));
            return Some(Fix { limits: false, size: target });
        }
        self.good = Some(size);
        None
    }
}

/// The inner height a window `height` tall should grow to so that `overflow` more points
/// of its page show, at most `cap`; None when it should stay as it is: everything shows,
/// it cannot grow, or that height (or more) was already asked for (`asked`), so it is
/// asked for once rather than every frame. Never smaller than it is.
pub fn grown_height(height: f32, overflow: f32, cap: f32, asked: Option<f32>) -> Option<f32> {
    if overflow <= 1.0 {
        return None;
    }
    let want = (height + overflow.ceil() + 2.0).min(cap);
    if want <= height + 1.0 || asked.is_some_and(|h| h >= want - 1.0) {
        return None;
    }
    Some(want)
}

fn close(a: Size, b: Size) -> bool {
    (a[0] - b[0]).abs() <= TOLERANCE && (a[1] - b[1]).abs() <= TOLERANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: Size = [760.0, 640.0];
    const MAX: Size = [1200.0, 1100.0];

    #[test]
    fn steady_sizes_are_remembered_and_left_alone() {
        let mut k = SizeKeeper::new(MIN, MAX);
        assert_eq!(k.step(1.5, [840.0, 870.0]), None);
        // The user drags it larger: nothing to fix.
        assert_eq!(k.step(1.5, [900.0, 900.0]), None);
        assert!(!k.busy());
    }

    #[test]
    fn moving_to_a_monitor_with_less_scaling_puts_the_size_back() {
        let mut k = SizeKeeper::new(MIN, MAX);
        k.step(1.5, [900.0, 900.0]);
        // At 100% the stale 1140×960-pixel minimum made it 1140×960 points.
        let fix = k.step(1.0, [1140.0, 960.0]);
        assert_eq!(fix, Some(Fix { limits: true, size: [900.0, 900.0] }));
        // Not there yet: asked again, without the limits.
        assert_eq!(k.step(1.0, [1140.0, 960.0]), Some(Fix { limits: false, size: [900.0, 900.0] }));
        // There: remembered, and left alone from then on.
        assert_eq!(k.step(1.0, [900.5, 899.0]), None);
        assert!(!k.busy());
        assert_eq!(k.step(1.0, [900.5, 899.0]), None);
        // And back to the first monitor.
        assert_eq!(k.step(1.5, [900.0, 733.0]), Some(Fix { limits: true, size: [900.5, 899.0] }));
    }

    #[test]
    fn a_size_beyond_the_limits_is_brought_inside() {
        let mut k = SizeKeeper::new(MIN, MAX);
        // The first size seen is already too large (no earlier size to go back to).
        assert_eq!(k.step(1.0, [1300.0, 600.0]), Some(Fix { limits: false, size: [1200.0, 640.0] }));
        assert_eq!(k.step(1.0, [1200.0, 640.0]), None);
        let mut k = SizeKeeper::new(MIN, MAX);
        k.step(2.0, [700.0, 500.0]);
        assert!(k.busy());
        // A change of scaling before any good size was seen: the current size, clamped.
        let mut k = SizeKeeper::new(MIN, MAX);
        k.scale = Some(2.0);
        assert_eq!(k.step(1.25, [1500.0, 700.0]), Some(Fix { limits: true, size: [1200.0, 700.0] }));
    }

    #[test]
    fn gives_up_when_the_size_does_not_stick() {
        let mut k = SizeKeeper::new(MIN, MAX);
        k.step(1.0, [840.0, 870.0]);
        k.step(1.5, [1000.0, 1000.0]);
        let tries = (0..100).take_while(|_| k.step(1.5, [1000.0, 1000.0]).is_some()).count();
        assert_eq!(tries, usize::from(TRIES));
        // Then the size it ended at is accepted as it is.
        assert_eq!(k.step(1.5, [1000.0, 1000.0]), None);
        assert!(!k.busy());
    }

    #[test]
    fn grows_once_to_fit_the_page_within_the_screen() {
        // Everything shows: nothing to do.
        assert_eq!(grown_height(1020.0, 0.0, 1200.0, None), None);
        assert_eq!(grown_height(1020.0, -300.0, 1200.0, None), None);
        // 60.4 points hidden: that much taller, with a little room.
        assert_eq!(grown_height(1020.0, 60.4, 1200.0, None), Some(1083.0));
        // Asked for already: not again on the next frames, nor for less.
        assert_eq!(grown_height(1020.0, 60.4, 1200.0, Some(1083.0)), None);
        assert_eq!(grown_height(1020.0, 30.0, 1200.0, Some(1083.0)), None);
        // The page got longer still: once more.
        assert_eq!(grown_height(1083.0, 50.0, 1200.0, Some(1083.0)), Some(1135.0));
        // No taller than the screen allows, and nothing once there.
        assert_eq!(grown_height(1020.0, 400.0, 1200.0, None), Some(1200.0));
        assert_eq!(grown_height(1200.0, 220.0, 1200.0, Some(1200.0)), None);
        assert_eq!(grown_height(1200.0, 220.0, 1200.0, None), None);
        // A window the user made taller than the cap is never shrunk.
        assert_eq!(grown_height(1250.0, 40.0, 1200.0, None), None);
        // The grown size is within the limits, so the keeper takes it as the good size.
        let mut k = SizeKeeper::new(MIN, [1200.0, 1300.0]);
        assert_eq!(k.step(1.25, [840.0, 1020.0]), None);
        assert_eq!(k.step(1.25, [840.0, 1135.0]), None);
        assert_eq!(k.step(1.0, [840.0, 1135.0]), Some(Fix { limits: true, size: [840.0, 1135.0] }));
    }

    #[test]
    fn a_fixed_window_keeps_its_one_size() {
        let mut k = SizeKeeper::fixed([620.0, 440.0]);
        assert_eq!(k.step(1.0, [620.0, 440.0]), None);
        assert_eq!(k.step(1.75, [620.0, 440.0]), Some(Fix { limits: true, size: [620.0, 440.0] }));
        assert_eq!(k.step(1.75, [620.0, 440.0]), None);
        assert_eq!(k.step(1.75, [700.0, 500.0]), Some(Fix { limits: false, size: [620.0, 440.0] }));
    }
}
