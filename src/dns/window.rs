use crate::dns::types::WindowState;
use dioxus::desktop::tao::monitor::MonitorHandle;
use dioxus::desktop::tao::window::Window;

/// Capture current window state.
/// Position is stored in physical pixels, size in logical pixels.
/// Returns None if position cannot be determined (e.g., minimized).
pub fn capture_window_state(window: &Window) -> Option<WindowState> {
    let scale = window.scale_factor();
    let position = window.outer_position().ok()?;
    let size = window.inner_size().to_logical::<u32>(scale);
    let maximized = window.is_maximized();

    Some(WindowState {
        x: position.x,
        y: position.y,
        width: size.width.max(WindowState::MIN_WIDTH),
        height: size.height.max(WindowState::MIN_HEIGHT),
        maximized,
    })
}

/// Validate window state against available monitors.
/// Returns corrected state that is guaranteed to be visible.
/// Position comparison uses physical coordinates.
pub fn validate_window_state(
    state: &WindowState,
    monitors: &[MonitorHandle],
    primary_monitor: Option<&MonitorHandle>,
) -> WindowState {
    let width = state.width.max(WindowState::MIN_WIDTH);
    let height = state.height.max(WindowState::MIN_HEIGHT);

    let is_visible = monitors.iter().any(|m| {
        let scale = m.scale_factor();
        let pos = m.position();
        let msize = m.size();
        let left = pos.x;
        let top = pos.y;
        let right = left + msize.width as i32;
        let bottom = top + msize.height as i32;

        let physical_width = (width as f64 * scale) as i32;
        let physical_height = (height as f64 * scale) as i32;

        state.x < right
            && (state.x + physical_width) > left
            && state.y < bottom
            && (state.y + physical_height) > top
    });

    if is_visible {
        WindowState {
            x: state.x,
            y: state.y,
            width,
            height,
            maximized: state.maximized,
        }
    } else {
        let fallback_monitor = primary_monitor.or_else(|| monitors.first());

        if let Some(monitor) = fallback_monitor {
            let scale = monitor.scale_factor();
            let mpos = monitor.position();
            let msize = monitor.size();

            let physical_width = (width as f64 * scale) as i32;
            let physical_height = (height as f64 * scale) as i32;

            let x = mpos.x + ((msize.width as i32 - physical_width) / 2);
            let y = mpos.y + ((msize.height as i32 - physical_height) / 2);

            WindowState {
                x,
                y,
                width,
                height,
                maximized: false,
            }
        } else {
            WindowState::default()
        }
    }
}
