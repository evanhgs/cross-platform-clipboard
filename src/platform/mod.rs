//! Adaptateurs spécifiques à chaque OS.

mod macos;
mod wayland_portal;
mod windows;
mod x11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutBackend {
    WaylandPortal,
    X11,
    Windows,
    Macos,
}
