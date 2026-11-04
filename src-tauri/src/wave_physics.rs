//! Native Rust Wave Carousel Geometry, Spring Physics, and Color Palette Engine for MicroSwap.
//!
//! This module provides deterministic layout calculations, spring-damper physics simulation,
//! color palette analysis, and polygon clipping geometry for the MicroSwap wave carousel.

use serde::{Deserialize, Serialize};

/// Represents a 2D point in normalized percentage space `[0.0 .. 100.0]`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        let clamped = t.clamp(0.0, 1.0);
        Self {
            x: self.x + (other.x - self.x) * clamped,
            y: self.y + (other.y - self.y) * clamped,
        }
    }

    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Represents a 4-corner sheared polygon used to clip vertical wallpaper slices.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShearedQuad {
    pub top_left: Point2D,
    pub top_right: Point2D,
    pub bottom_right: Point2D,
    pub bottom_left: Point2D,
}

impl ShearedQuad {
    pub const fn rectangle() -> Self {
        Self {
            top_left: Point2D::new(0.0, 0.0),
            top_right: Point2D::new(100.0, 0.0),
            bottom_right: Point2D::new(100.0, 100.0),
            bottom_left: Point2D::new(0.0, 100.0),
        }
    }

    pub const fn left_wing(shear_pct: f64) -> Self {
        Self {
            top_left: Point2D::new(0.0, shear_pct),
            top_right: Point2D::new(100.0, 0.0),
            bottom_right: Point2D::new(100.0, 100.0 - shear_pct),
            bottom_left: Point2D::new(0.0, 100.0),
        }
    }

    pub const fn right_wing(shear_pct: f64) -> Self {
        Self {
            top_left: Point2D::new(0.0, 0.0),
            top_right: Point2D::new(100.0, shear_pct),
            bottom_right: Point2D::new(100.0, 100.0),
            bottom_left: Point2D::new(0.0, 100.0 - shear_pct),
        }
    }

    pub const fn center_blade(shear_pct: f64) -> Self {
        Self {
            top_left: Point2D::new(0.0, shear_pct),
            top_right: Point2D::new(100.0, 0.0),
            bottom_right: Point2D::new(100.0, 100.0 - shear_pct),
            bottom_left: Point2D::new(0.0, 100.0),
        }
    }

    pub fn to_css_polygon(self) -> String {
        format!(
            "polygon({:.1}% {:.1}%, {:.1}% {:.1}%, {:.1}% {:.1}%, {:.1}% {:.1}%)",
            self.top_left.x,
            self.top_left.y,
            self.top_right.x,
            self.top_right.y,
            self.bottom_right.x,
            self.bottom_right.y,
            self.bottom_left.x,
            self.bottom_left.y
        )
    }

    pub fn interpolate(self, target: Self, t: f64) -> Self {
        Self {
            top_left: self.top_left.lerp(target.top_left, t),
            top_right: self.top_right.lerp(target.top_right, t),
            bottom_right: self.bottom_right.lerp(target.bottom_right, t),
            bottom_left: self.bottom_left.lerp(target.bottom_left, t),
        }
    }
}

/// Complete computed visual metrics for a single slice in the wave carousel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedSliceVisuals {
    pub offset: i32,
    pub width: f64,
    pub height: f64,
    pub opacity: f64,
    pub brightness: f64,
    pub translate_y: f64,
    pub scale: f64,
    pub skew_x: f64,
    pub skew_y: f64,
    pub rotate_z: f64,
    pub clip_path: String,
    pub z_index: i32,
}

/// Wave profile table defining base heights and widths from center (`offset = 0`) outward.
#[derive(Debug, Clone)]
pub struct WaveProfile {
    pub heights: [f64; 10],
    pub widths: [f64; 10],
    pub expanded_width: f64,
    pub expanded_height: f64,
    pub base_skew_x: f64,
}

impl Default for WaveProfile {
    fn default() -> Self {
        Self {
            heights: [550.0, 504.0, 448.0, 392.0, 336.0, 282.0, 232.0, 188.0, 152.0, 120.0],
            widths: [152.0, 132.0, 114.0, 98.0, 86.0, 74.0, 64.0, 54.0, 46.0, 38.0],
            expanded_width: 780.0,
            expanded_height: 460.0,
            base_skew_x: -4.5,
        }
    }
}

impl WaveProfile {
    pub fn compute_slice(
        &self,
        offset: i32,
        is_expanded_center: bool,
        is_hovered: bool,
        hovered_offset: Option<i32>,
        is_being_dragged: bool,
    ) -> ComputedSliceVisuals {
        let abs = offset.unsigned_abs() as usize;

        if abs == 0 && is_expanded_center {
            let quad = ShearedQuad::center_blade(2.5);
            return ComputedSliceVisuals {
                offset,
                width: self.expanded_width,
                height: self.expanded_height,
                opacity: 1.0,
                brightness: 1.08,
                translate_y: 0.0,
                scale: 1.03,
                skew_x: -3.0,
                skew_y: -1.5,
                rotate_z: -0.8,
                clip_path: quad.to_css_polygon(),
                z_index: 25,
            };
        }

        let idx = abs.min(self.heights.len() - 1);
        let base_height = self.heights[idx];
        let base_width = self.widths[idx];

        let dist_to_hover = hovered_offset
            .map(|h| (offset - h).unsigned_abs() as usize)
            .unwrap_or(99);

        let hover_height_boost = if is_hovered && abs != 0 {
            42.0
        } else if dist_to_hover == 1 && abs != 0 {
            18.0
        } else {
            0.0
        };

        let hover_width_boost = if is_hovered && abs != 0 {
            18.0
        } else if dist_to_hover == 1 && abs != 0 {
            7.0
        } else {
            0.0
        };

        let height = base_height + hover_height_boost;
        let width = base_width + hover_width_boost;

        let opacity = if is_being_dragged {
            0.92
        } else if abs > 7 {
            0.38
        } else if abs > 5 {
            0.74
        } else {
            1.0
        };

        let brightness = if abs == 0 {
            1.14
        } else if is_hovered {
            1.10
        } else if dist_to_hover == 1 {
            0.98
        } else {
            (1.0 - (abs as f64) * 0.065).max(0.48)
        };

        let translate_y = if is_being_dragged {
            -18.0
        } else if abs == 0 {
            -10.0
        } else if is_hovered {
            -12.0
        } else if dist_to_hover == 1 {
            -5.0
        } else {
            0.0
        };

        let scale = if is_being_dragged {
            1.08
        } else if abs == 0 {
            1.045
        } else if is_hovered {
            1.06
        } else if dist_to_hover == 1 {
            1.02
        } else {
            1.0
        };

        let skew_x = self.base_skew_x;
        let skew_y = if offset == 0 {
            -2.2
        } else if offset < 0 {
            -3.2
        } else {
            2.6
        };

        let rotate_z = if is_hovered {
            (offset as f64) * 0.2 - 0.5
        } else {
            (offset as f64) * 0.35 - 1.2
        };

        let quad = if offset < 0 {
            ShearedQuad::left_wing(5.5)
        } else if offset > 0 {
            ShearedQuad::right_wing(5.5)
        } else {
            ShearedQuad::center_blade(3.5)
        };

        let z_index = if is_being_dragged {
            30
        } else if abs == 0 {
            20
        } else {
            15 - (abs.min(10) as i32)
        };

        ComputedSliceVisuals {
            offset,
            width,
            height,
            opacity,
            brightness,
            translate_y,
            scale,
            skew_x,
            skew_y,
            rotate_z,
            clip_path: quad.to_css_polygon(),
            z_index,
        }
    }

    pub fn compute_band(
        &self,
        radius: i32,
        is_expanded_center: bool,
        hovered_offset: Option<i32>,
        dragged_offset: Option<i32>,
    ) -> Vec<ComputedSliceVisuals> {
        let mut band = Vec::with_capacity((radius * 2 + 1) as usize);
        for offset in -radius..=radius {
            let is_hovered = hovered_offset == Some(offset);
            let is_dragged = dragged_offset == Some(offset);
            band.push(self.compute_slice(
                offset,
                is_expanded_center,
                is_hovered,
                hovered_offset,
                is_dragged,
            ));
        }
        band
    }
}

/// Second-order damped harmonic oscillator for simulating bouncy spring motion in Rust.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpringSolver {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Default for SpringSolver {
    fn default() -> Self {
        Self {
            stiffness: 280.0,
            damping: 18.0,
            mass: 1.0,
        }
    }
}

impl SpringSolver {
    pub const fn bouncy() -> Self {
        Self {
            stiffness: 340.0,
            damping: 15.5,
            mass: 1.0,
        }
    }

    pub const fn smooth_portal() -> Self {
        Self {
            stiffness: 190.0,
            damping: 24.0,
            mass: 1.1,
        }
    }

    /// Evaluates normalized position `[0.0 -> 1.0]` at elapsed time `t_seconds`.
    pub fn sample(&self, t_seconds: f64) -> f64 {
        if t_seconds <= 0.0 {
            return 0.0;
        }
        let omega_n = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());

        if zeta < 1.0 {
            // Under-damped (bouncy overshoot)
            let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
            let envelope = (-zeta * omega_n * t_seconds).exp();
            let cos_part = (omega_d * t_seconds).cos();
            let sin_part = (zeta / (1.0 - zeta * zeta).sqrt()) * (omega_d * t_seconds).sin();
            1.0 - envelope * (cos_part + sin_part)
        } else {
            // Critically damped
            let envelope = (-omega_n * t_seconds).exp();
            1.0 - envelope * (1.0 + omega_n * t_seconds)
        }
    }

    /// Generates a lookup curve of `steps` samples across `duration_ms`.
    pub fn sample_curve(&self, duration_ms: u32, steps: usize) -> Vec<f64> {
        if steps == 0 {
            return Vec::new();
        }
        let total_sec = (duration_ms as f64) / 1000.0;
        (0..=steps)
            .map(|i| {
                let frac = (i as f64) / (steps as f64);
                self.sample(frac * total_sec)
            })
            .collect()
    }
}

/// Supported exit choreography modes when applying a wallpaper from the wave carousel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApplyAnimationStyle {
    Cascade,
    Accordion,
    Laser,
    Shutter,
}

impl ApplyAnimationStyle {
    pub fn from_key(key: &str) -> Self {
        match key.trim().to_ascii_lowercase().as_str() {
            "accordion" | "fold" => Self::Accordion,
            "laser" => Self::Laser,
            "shutter" => Self::Shutter,
            _ => Self::Cascade,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Cascade => "Wave",
            Self::Accordion => "Fold",
            Self::Laser => "Laser",
            Self::Shutter => "Shutter",
        }
    }

    pub fn total_duration_ms(&self) -> u32 {
        match self {
            Self::Cascade => 300,
            Self::Accordion => 290,
            Self::Laser => 280,
            Self::Shutter => 280,
        }
    }

    pub fn os_handoff_delay_ms(&self) -> u32 {
        90
    }
}

/// Computed per-slice exit transform target for the 4 apply animation styles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceExitTrajectory {
    pub slot_index: usize,
    pub offset: i32,
    pub is_selected: bool,
    pub stagger_delay_ms: u32,
    pub duration_ms: u32,
    pub target_translate_x: f64,
    pub target_translate_y: f64,
    pub target_skew_x: f64,
    pub target_skew_y: f64,
    pub target_rotate_z: f64,
    pub target_scale_x: f64,
    pub target_scale_y: f64,
    pub target_opacity: f64,
    pub target_brightness: f64,
    pub css_transform: String,
    pub css_transition: String,
}

/// Computes deterministic exit trajectories for every slice in the carousel
/// when a wallpaper transition is triggered.
#[derive(Debug, Clone)]
pub struct ApplyChoreographyEngine {
    pub style: ApplyAnimationStyle,
    pub wave_profile: WaveProfile,
}

impl ApplyChoreographyEngine {
    pub fn new(style: ApplyAnimationStyle) -> Self {
        Self {
            style,
            wave_profile: WaveProfile::default(),
        }
    }

    pub fn compute_slice_exit(
        &self,
        slot_index: usize,
        selected_index: usize,
        expanded_center: bool,
    ) -> SliceExitTrajectory {
        let offset = (slot_index as i32) - (selected_index as i32);
        let abs_offset = offset.unsigned_abs();
        let is_selected = offset == 0;
        let base = self
            .wave_profile
            .compute_slice(offset, expanded_center, false, None, false);

        if is_selected {
            let (tx, ty, sx, sy, rz, scx, scy, br) = match self.style {
                ApplyAnimationStyle::Cascade => (0.0, 18.0, -2.0, -1.0, -0.4, 0.96, 0.96, 1.05),
                ApplyAnimationStyle::Accordion => (0.0, 0.0, 0.0, -18.0, 0.0, 0.03, 0.92, 1.40),
                ApplyAnimationStyle::Laser => (0.0, -14.0, 0.0, 0.0, 0.0, 0.01, 1.65, 2.50),
                ApplyAnimationStyle::Shutter => (0.0, -14.0, 0.0, 0.0, 0.0, 1.04, 0.04, 1.50),
            };
            let duration_ms = self.style.total_duration_ms();
            let css_transform = format!(
                "translateX({:.1}px) translateY({:.1}px) skewX({:.2}deg) skewY({:.2}deg) rotateZ({:.2}deg) scaleX({:.3}) scaleY({:.3})",
                tx, ty, sx, sy, rz, scx, scy
            );
            let css_transition = format!(
                "transform {}ms cubic-bezier(0.22, 1, 0.36, 1), opacity {}ms ease, filter {}ms ease",
                duration_ms, duration_ms, duration_ms
            );

            return SliceExitTrajectory {
                slot_index,
                offset,
                is_selected: true,
                stagger_delay_ms: 0,
                duration_ms,
                target_translate_x: tx,
                target_translate_y: ty,
                target_skew_x: sx,
                target_skew_y: sy,
                target_rotate_z: rz,
                target_scale_x: scx,
                target_scale_y: scy,
                target_opacity: 0.0,
                target_brightness: br,
                css_transform,
                css_transition,
            };
        }

        let stagger_delay_ms = match self.style {
            ApplyAnimationStyle::Cascade => (abs_offset * 22).min(110),
            ApplyAnimationStyle::Accordion => (abs_offset * 16).min(80),
            ApplyAnimationStyle::Laser => (abs_offset * 14).min(70),
            ApplyAnimationStyle::Shutter => (abs_offset * 12).min(60),
        };

        let (tx, ty, sx, sy, rz, scx, scy, br) = match self.style {
            ApplyAnimationStyle::Cascade => {
                let domino_tilt = if offset < 0 { -9.0 } else { 9.0 };
                (
                    0.0,
                    base.translate_y + 135.0,
                    base.skew_x,
                    base.skew_y,
                    base.rotate_z + domino_tilt,
                    base.scale * 0.68,
                    base.scale * 0.68,
                    base.brightness,
                )
            }
            ApplyAnimationStyle::Accordion => {
                let pleat_dir = if abs_offset % 2 == 0 { 22.0 } else { -22.0 };
                let pull_in_x = if offset < 0 { 28.0 } else { -28.0 };
                (
                    pull_in_x,
                    base.translate_y,
                    0.0,
                    pleat_dir,
                    0.0,
                    0.05,
                    base.scale * 0.92,
                    base.brightness * 1.15,
                )
            }
            ApplyAnimationStyle::Laser => (
                0.0,
                base.translate_y - 12.0,
                0.0,
                0.0,
                0.0,
                0.03,
                base.scale * 1.45,
                2.20,
            ),
            ApplyAnimationStyle::Shutter => {
                let zipper_dir = if slot_index % 2 == 0 { -210.0 } else { 210.0 };
                (
                    0.0,
                    base.translate_y + zipper_dir,
                    base.skew_x,
                    base.skew_y,
                    base.rotate_z,
                    base.scale * 0.88,
                    base.scale * 0.55,
                    base.brightness,
                )
            }
        };

        let css_transform = format!(
            "translateX({:.1}px) translateY({:.1}px) skewX({:.2}deg) skewY({:.2}deg) rotateZ({:.2}deg) scaleX({:.3}) scaleY({:.3})",
            tx, ty, sx, sy, rz, scx, scy
        );
        let css_transition = format!(
            "transform 210ms cubic-bezier(0.32, 0, 0.2, 1) {}ms, opacity 195ms ease {}ms, filter 180ms ease {}ms",
            stagger_delay_ms, stagger_delay_ms, stagger_delay_ms
        );

        SliceExitTrajectory {
            slot_index,
            offset,
            is_selected: false,
            stagger_delay_ms,
            duration_ms: 210,
            target_translate_x: tx,
            target_translate_y: ty,
            target_skew_x: sx,
            target_skew_y: sy,
            target_rotate_z: rz,
            target_scale_x: scx,
            target_scale_y: scy,
            target_opacity: 0.0,
            target_brightness: br,
            css_transform,
            css_transition,
        }
    }

    pub fn compute_batch(
        &self,
        total_slices: usize,
        selected_index: usize,
        expanded_center: bool,
    ) -> Vec<SliceExitTrajectory> {
        (0..total_slices)
            .map(|idx| self.compute_slice_exit(idx, selected_index, expanded_center))
            .collect()
    }
}

/// Offline procedural acoustic synthesizer for generating ultra-low-latency
/// mechanical detent clicks and sub-bass confirmation envelopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TactileAcousticSynthesizer {
    pub sample_rate: u32,
    pub master_gain: f64,
    pub lowpass_cutoff_hz: f64,
}

impl Default for TactileAcousticSynthesizer {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            master_gain: 0.06,
            lowpass_cutoff_hz: 920.0,
        }
    }
}

impl TactileAcousticSynthesizer {
    /// Computes the starting pitch frequency in Hz for a given slot index across the wave.
    pub fn tick_start_frequency_hz(&self, slot_index: usize, total_slots: usize) -> f64 {
        let ratio = (slot_index as f64) / (total_slots.max(1) as f64);
        215.0 + ratio * 80.0
    }

    /// Synthesizes a 14ms mono PCM waveform buffer (`f32` in `[-1.0, 1.0]`) for a mechanical scroll detent click.
    pub fn synthesize_tick_pcm(&self, slot_index: usize, total_slots: usize) -> Vec<f32> {
        let duration_sec = 0.014;
        let num_samples = ((self.sample_rate as f64) * duration_sec).round() as usize;
        let f_start = self.tick_start_frequency_hz(slot_index, total_slots);
        let f_end = 72.0;

        let mut raw = Vec::with_capacity(num_samples);
        let mut phase = 0.0_f64;

        for i in 0..num_samples {
            let t = (i as f64) / (self.sample_rate as f64);
            let frac = (t / duration_sec).clamp(0.0, 1.0);
            let freq = f_start * (f_end / f_start).powf(frac);
            phase += freq / (self.sample_rate as f64);
            let tri = 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0;
            let env = self.master_gain * (0.0001_f64 / self.master_gain).powf(frac);
            raw.push((tri * env) as f32);
        }

        self.apply_single_pole_lowpass(&raw, self.lowpass_cutoff_hz)
    }

    /// Synthesizes a 70ms sub-bass confirmation pulse (`135 Hz -> 48 Hz`) when applying a wallpaper.
    pub fn synthesize_apply_pcm(&self) -> Vec<f32> {
        let duration_sec = 0.070;
        let num_samples = ((self.sample_rate as f64) * duration_sec).round() as usize;
        let f_start = 135.0_f64;
        let f_end = 48.0_f64;
        let peak_gain = 0.09_f64;

        let mut out = Vec::with_capacity(num_samples);
        let mut phase = 0.0_f64;

        for i in 0..num_samples {
            let t = (i as f64) / (self.sample_rate as f64);
            let frac = (t / duration_sec).clamp(0.0, 1.0);
            let freq = f_start * (f_end / f_start).powf(frac);
            phase += freq / (self.sample_rate as f64);
            let sample = (phase * std::f64::consts::TAU).sin();
            let env = peak_gain * (0.0001_f64 / peak_gain).powf(frac);
            out.push((sample * env) as f32);
        }

        out
    }

    fn apply_single_pole_lowpass(&self, input: &[f32], cutoff_hz: f64) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        let dt = 1.0 / (self.sample_rate as f64);
        let rc = 1.0 / (std::f64::consts::TAU * cutoff_hz.max(20.0));
        let alpha = (dt / (rc + dt)) as f32;

        let mut output = Vec::with_capacity(input.len());
        let mut prev = input[0];
        output.push(prev);

        for &x in &input[1..] {
            let y = prev + alpha * (x - prev);
            output.push(y);
            prev = y;
        }
        output
    }
}

/// Perceptual Oklab color & luminance analyzer for wallpaper slices.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OklabColor {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl OklabColor {
    fn srgb_to_linear(channel: u8) -> f64 {
        let c = (channel as f64) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let lr = Self::srgb_to_linear(r);
        let lg = Self::srgb_to_linear(g);
        let lb = Self::srgb_to_linear(b);

        let l = (0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb).cbrt();
        let m = (0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb).cbrt();
        let s = (0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb).cbrt();

        Self {
            l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        }
    }

    pub fn chroma(&self) -> f64 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    pub fn hue_degrees(&self) -> f64 {
        self.b.atan2(self.a).to_degrees().rem_euclid(360.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_profile_symmetry() {
        let profile = WaveProfile::default();
        let left = profile.compute_slice(-3, false, false, None, false);
        let right = profile.compute_slice(3, false, false, None, false);
        assert_eq!(left.height, right.height);
        assert_eq!(left.width, right.width);
    }

    #[test]
    fn test_spring_solver_convergence() {
        let spring = SpringSolver::bouncy();
        let start = spring.sample(0.0);
        let end = spring.sample(2.0);
        assert!((start - 0.0).abs() < 1e-5);
        assert!((end - 1.0).abs() < 1e-2);
    }

    #[test]
    fn test_apply_choreography_styles() {
        for key in ["cascade", "accordion", "laser", "shutter"] {
            let style = ApplyAnimationStyle::from_key(key);
            let engine = ApplyChoreographyEngine::new(style);
            let batch = engine.compute_batch(8, 3, false);
            assert_eq!(batch.len(), 8);
            assert!(batch[3].is_selected);
            assert_eq!(batch[3].stagger_delay_ms, 0);
        }
    }

    #[test]
    fn test_tactile_acoustic_synthesis() {
        let synth = TactileAcousticSynthesizer::default();
        let tick = synth.synthesize_tick_pcm(2, 10);
        let apply = synth.synthesize_apply_pcm();
        assert!(!tick.is_empty());
        assert!(!apply.is_empty());
    }

    #[test]
    fn test_oklab_luminance_bounds() {
        let black = OklabColor::from_rgb(0, 0, 0);
        let white = OklabColor::from_rgb(255, 255, 255);
        assert!(black.l < 0.01);
        assert!((white.l - 1.0).abs() < 0.01);
    }
}
