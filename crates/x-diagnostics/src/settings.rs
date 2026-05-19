use std::sync::atomic;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{ColourChoice, DiagnosticStyle};

static PRETTY_DIAGNOSTICS: AtomicBool = AtomicBool::new(true);
static DIAGNOSTIC_STYLE: atomic::AtomicU8 = atomic::AtomicU8::new(DiagnosticStyle::Rich as u8);
static DIAGNOSTIC_COLOUR: atomic::AtomicU8 = atomic::AtomicU8::new(ColourChoice::Auto as u8);

pub fn set_pretty_diagnostics_enabled(enabled: bool) {
    PRETTY_DIAGNOSTICS.store(enabled, Ordering::Relaxed);
}

pub fn pretty_diagnostics_enabled() -> bool {
    PRETTY_DIAGNOSTICS.load(Ordering::Relaxed)
}

pub fn set_diagnostic_style(style: DiagnosticStyle) {
    DIAGNOSTIC_STYLE.store(style as u8, Ordering::Relaxed);
}

pub fn diagnostic_style() -> DiagnosticStyle {
    match DIAGNOSTIC_STYLE.load(Ordering::Relaxed) {
        value if value == DiagnosticStyle::Compact as u8 => DiagnosticStyle::Compact,
        value if value == DiagnosticStyle::Ascii as u8 => DiagnosticStyle::Ascii,
        value if value == DiagnosticStyle::Unicode as u8 => DiagnosticStyle::Unicode,
        _ => DiagnosticStyle::Rich,
    }
}

pub fn set_diagnostic_colour_choice(choice: ColourChoice) {
    DIAGNOSTIC_COLOUR.store(choice as u8, Ordering::Relaxed);
}

pub fn diagnostic_colour_choice() -> ColourChoice {
    match DIAGNOSTIC_COLOUR.load(Ordering::Relaxed) {
        value if value == ColourChoice::Always as u8 => ColourChoice::Always,
        value if value == ColourChoice::Never as u8 => ColourChoice::Never,
        _ => ColourChoice::Auto,
    }
}
