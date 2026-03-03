use std::fmt::{self, Display, Formatter};

use bitflags::bitflags;
#[rustfmt::skip]
#[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
use {
    std::io::Cursor,
    glutin::platform::x11::X11VisualInfo,
    winit::icon::RgbaIcon,
    winit::platform::x11::WindowAttributesX11,
};
use winit::cursor::CursorIcon;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;
#[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
use winit::platform::wayland::WindowAttributesWayland;
#[cfg(windows)]
use winit::platform::windows::{WinIcon, WindowAttributesWindows};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{
    ImePurpose, UserAttentionType, Window as WinitWindow, WindowAttributes, WindowId,
};
#[cfg(target_os = "macos")]
use {
    objc2::MainThreadMarker,
    objc2_app_kit::{NSColorSpace, NSView},
    winit::platform::macos::{OptionAsAlt, WindowAttributesMacOS, WindowExtMacOS},
};
#[cfg(not(any(target_os = "macos", windows)))]
use {
    winit::platform::startup_notify::{self, EventLoopExtStartupNotify},
    winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
    winit::window::ActivationToken,
};

use alacritty_terminal::index::Point;

use crate::cli::WindowOptions;
use crate::config::AlacrittyConfig;
use crate::config::window::{Identity, WindowConfig};
use crate::display::SizeInfo;

/// Window icon for `_NET_WM_ICON` property.
#[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
const WINDOW_ICON: &[u8] = include_bytes!("../../extra/logo/compat/alacritty-term.png");

/// This should match the definition of IDI_ICON from `alacritty.rc`.
#[cfg(windows)]
const IDI_ICON: u16 = 0x101;

/// Window errors.
#[derive(Debug)]
pub enum Error {
    /// Error creating the window.
    WindowCreation(winit::error::RequestError),

    /// Error dealing with fonts.
    Font(crossfont::Error),
}

/// Result of fallible operations concerning a Window.
type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::WindowCreation(err) => err.source(),
            Error::Font(err) => err.source(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::WindowCreation(err) => write!(f, "Error creating GL context; {err}"),
            Error::Font(err) => err.fmt(f),
        }
    }
}

impl From<winit::error::RequestError> for Error {
    fn from(val: winit::error::RequestError) -> Self {
        Error::WindowCreation(val)
    }
}

impl From<crossfont::Error> for Error {
    fn from(val: crossfont::Error) -> Self {
        Error::Font(val)
    }
}

/// A window which can be used for displaying the terminal.
///
/// Wraps the underlying windowing library to provide a stable API in Alacritty.
pub struct Window {
    /// Flag tracking that we have a frame we can draw.
    pub has_frame: bool,

    /// Cached scale factor for quickly scaling pixel sizes.
    pub scale_factor: f64,

    /// Flag indicating whether redraw was requested.
    pub requested_redraw: bool,

    /// Hold the window when terminal exits.
    pub hold: bool,

    window: Box<dyn WinitWindow>,

    /// Current window title.
    title: String,

    is_x11: bool,
    current_mouse_cursor: CursorIcon,
    mouse_visible: bool,
    ime_inhibitor: ImeInhibitor,
}

impl Window {
    /// Create a new window.
    ///
    /// This creates a window and fully initializes a window.
    pub fn new(
        event_loop: &dyn ActiveEventLoop,
        config: &AlacrittyConfig,
        identity: &Identity,
        options: &mut WindowOptions,
        #[rustfmt::skip]
        #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
        x11_visual: Option<X11VisualInfo>,
    ) -> Result<Window> {
        let mut window_attributes = Window::get_platform_window_attributes(
            #[cfg(not(any(target_os = "macos", windows)))]
            event_loop,
            identity,
            &config.window,
            #[cfg(not(any(target_os = "macos", windows)))]
            options.activation_token.take(),
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            x11_visual,
        );

        window_attributes = window_attributes
            .with_title(&identity.title)
            .with_visible(false)
            .with_transparent(true)
            .with_blur(config.window.blur)
            .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            .with_decorations(false);

        // #[cfg(not(target_os = "macos"))]
        // explicitly setting this can prevent x11 from ignoring the later set_position command
        {
            window_attributes =
                window_attributes.with_position(PhysicalPosition::new(0, 0)).with_active(true);
        }

        let window = event_loop.create_window(window_attributes)?;

        // Text cursor.
        let current_mouse_cursor = CursorIcon::Text;
        window.set_cursor(current_mouse_cursor.into());

        // Enable IME.
        #[allow(deprecated)]
        window.set_ime_allowed(true);
        #[allow(deprecated)]
        window.set_ime_purpose(ImePurpose::Terminal);

        // Set initial transparency hint.
        window.set_transparent(config.window_opacity() < 1.);

        if let Err(e) = Self::initialize_platform_window_handle(
            &*window,
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            event_loop,
        ) {
            log::error!("Failed to set raw window attributes: {e}")
        }

        #[cfg(target_os = "macos")]
        use_srgb_color_space(window.as_ref());

        let scale_factor = window.scale_factor();
        log::info!("Window scale factor: {scale_factor}");
        let is_x11 = matches!(window.window_handle().unwrap().as_raw(), RawWindowHandle::Xlib(_));

        Ok(Self {
            hold: options.terminal_options.hold,
            requested_redraw: false,
            title: identity.title.clone(),
            current_mouse_cursor,
            mouse_visible: true,
            has_frame: true,
            scale_factor,
            window,
            is_x11,
            ime_inhibitor: Default::default(),
        })
    }

    pub fn initialize_platform_window_handle(
        window: &dyn WinitWindow,
        #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
        event_loop: &dyn ActiveEventLoop,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let window_handle = window.window_handle().expect("Failed to get window handle").as_raw();

        match window_handle {
            // --- macOS (AppKit) ---
            #[cfg(target_os = "macos")]
            RawWindowHandle::AppKit(handle) => {
                use objc2::rc::Retained;
                use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

                //
                let view = unsafe {
                    assert!(MainThreadMarker::new().is_some());
                    handle.ns_view.cast::<NSView>().as_ref()
                };
                let ns_window: Retained<NSWindow> = view.window().expect("NSView has no window");

                let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::Stationary
                    | NSWindowCollectionBehavior::Transient;

                ns_window.setCollectionBehavior(behavior);
                // This prevents rendering artifacts from showing up when the window is transparent.
                ns_window.setHasShadow(false);
            },

            // --- Windows (Win32) ---
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(h) => {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    GWL_EXSTYLE, GetWindowLongW, SetWindowLongW, WS_EX_TOOLWINDOW,
                };

                unsafe {
                    let hwnd = HWND(h.hwnd.get() as _);

                    // WS_EX_TOOLWINDOW is the standard Win32 way to make a window
                    // appear on all virtual desktops automatically.
                    // Setting WS_EX_TOOLWINDOW makes the window stay on all virtual desktops
                    // in most Windows 10/11 versions, though it removes it from the taskbar.
                    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                    SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TOOLWINDOW.0 as i32);
                }
            },

            // --- X11 (Xlib Backend) ---
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            RawWindowHandle::Xlib(h) => {
                use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, PropMode};

                let RawDisplayHandle::Xlib(_display_h) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    unreachable!()
                };

                let (conn, _) = x11rb::connect(None)?;
                use x11rb::connection::Connection;

                let window_id = h.window as u32;
                let reply = conn.intern_atom(false, b"_NET_WM_DESKTOP").unwrap().reply()?;

                conn.change_property(
                    PropMode::REPLACE,
                    window_id,
                    reply.atom,
                    AtomEnum::CARDINAL,
                    32,
                    1,
                    &0xFFFFFFFFu32.to_ne_bytes(),
                )?;
                conn.get_input_focus()?.reply()?; // force a server round-trip otherwise handle gets dropped too early
                conn.flush()?;
            },

            // --- X11 (XCB Backend) ---
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            RawWindowHandle::Xcb(h) => {
                use x11rb::connection::Connection;
                use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, PropMode};

                let RawDisplayHandle::Xcb(_display_h) =
                    event_loop.display_handle().unwrap().as_raw()
                else {
                    unreachable!()
                };

                let (conn, _) = x11rb::connect(None)?;

                let window_id = h.window.get();
                let reply = conn.intern_atom(false, b"_NET_WM_DESKTOP").unwrap().reply()?;

                conn.change_property(
                    PropMode::REPLACE,
                    window_id,
                    reply.atom,
                    AtomEnum::CARDINAL,
                    32,
                    1,
                    &0xFFFFFFFFu32.to_ne_bytes(),
                )?;
                conn.get_input_focus()?.reply()?; // force a server round-trip otherwise handle gets dropped too early
                conn.flush()?;
            },

            // --- Wayland ---
            #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
            RawWindowHandle::Wayland(_h) => {
                // Wayland is more restrictive. Most compositors do not allow a client
                // to decide to be on all workspaces for security and design reasons.
                // Specialized components usually require the Layer Shell protocol.
            },

            _ => {
                log::error!(
                    "Sticky behavior not implemented or supported for this platform/backend."
                );
            },
        }
        Ok(())
    }

    #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
    fn apply_sticky_x11(
        raw_window: RawWindowHandle,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::{
            ClientMessageData, ClientMessageEvent, ConnectionExt, EventMask,
        };

        let window_id = match raw_window {
            RawWindowHandle::Xlib(h) => h.window as u32,
            RawWindowHandle::Xcb(h) => h.window.get(),
            _ => return Ok(()),
        };

        let (conn, screen_num) = x11rb::connect(None)?;
        let setup = conn.setup();
        let root = setup.roots[screen_num].root;

        // 1. Intern all necessary Atoms
        let net_wm_state = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
        let net_wm_state_sticky = conn.intern_atom(false, b"_NET_WM_STATE_STICKY")?.reply()?.atom;

        // This asks the WM to actively toggle the 'sticky' state.
        // data.l[0] = 1 (Action: _NET_WM_STATE_ADD)
        // data.l[1] = The atom we want to add
        // data.l[3] = 1 (Source Indication: Application)
        let data = ClientMessageData::from([
            1, // _NET_WM_STATE_ADD
            net_wm_state_sticky,
            0, // Second property (unused)
            1, // Source: Normal Application
            0, // Unused
        ]);

        let event = ClientMessageEvent {
            response_type: 33, // ClientMessage
            format: 32,
            sequence: 0,
            window: window_id,
            type_: net_wm_state,
            data,
        };

        conn.send_event(
            false,
            root, // Must be sent to the Root window
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;

        // Force a round-trip to ensure the server processes these before we drop 'conn'
        conn.get_input_focus()?.reply()?;
        conn.flush()?;

        Ok(())
    }

    #[inline]
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        self.window.window_handle().unwrap().as_raw()
    }

    #[inline]
    pub fn request_inner_size(&self, size: PhysicalSize<u32>) {
        let _ = self.window.request_surface_size(size.into());
    }

    #[inline]
    pub fn set_position(&self, position: impl Into<winit::dpi::Position>) {
        self.window.set_outer_position(position.into())
    }

    #[allow(dead_code)]
    #[inline]
    pub fn outer_position(&self) -> Option<PhysicalPosition<i32>> {
        self.window.outer_position().ok()
    }

    #[inline]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.window.surface_size()
    }

    #[inline]
    pub fn set_visible(&self, visibility: bool) {
        self.window.set_visible(visibility);
        // todo: check if this should be universally set in ::Occluded
        if visibility {
            self.window.focus_window();
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            {
                use cli_boilerplate_automation::bait::ResultExt;

                let window_handle =
                    self.window.window_handle().expect("Failed to get window handle").as_raw();
                Window::apply_sticky_x11(window_handle)._elog();
            }
        }
    }

    /// Set the window title.
    #[inline]
    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.window.set_title(&self.title);
    }

    #[inline]
    pub fn focus(&self) {
        self.window.focus_window();
    }

    #[allow(dead_code)]
    #[inline]
    pub fn has_focus(&self) -> bool {
        self.window.has_focus()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn is_visible(&self) -> Option<bool> {
        self.window.is_visible()
    }

    /// Get the window title.
    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn request_redraw(&mut self) {
        if !self.requested_redraw {
            self.requested_redraw = true;
            self.window.request_redraw();
        }
    }

    #[inline]
    pub fn set_mouse_cursor(&mut self, cursor: CursorIcon) {
        if cursor != self.current_mouse_cursor {
            self.current_mouse_cursor = cursor;
            self.window.set_cursor(cursor.into());
        }
    }

    /// Set mouse cursor visible.
    pub fn set_mouse_visible(&mut self, visible: bool) {
        if visible != self.mouse_visible {
            self.mouse_visible = visible;
            self.window.set_cursor_visible(visible);
        }
    }

    #[inline]
    pub fn mouse_visible(&self) -> bool {
        self.mouse_visible
    }

    #[cfg(not(any(target_os = "macos", windows)))]
    pub fn get_platform_window_attributes(
        event_loop: &dyn ActiveEventLoop,
        identity: &Identity,
        window_config: &WindowConfig,
        activation_token: Option<String>,
        #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))] x11_visual: Option<
            X11VisualInfo,
        >,
    ) -> WindowAttributes {
        let activation_token = activation_token
            .map(ActivationToken::from_raw)
            .or_else(|| event_loop.read_token_from_env())
            .map(|activation_token| {
                log::debug!("Activating window with token: {activation_token:?}");
                // Remove the token from the env.
                startup_notify::reset_activation_token_env();
                activation_token
            });

        match event_loop.display_handle().unwrap().as_raw() {
            #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
            RawDisplayHandle::Xlib(_) | RawDisplayHandle::Xcb(_) => {
                Self::get_x11_window_attributes(
                    identity,
                    window_config,
                    activation_token,
                    x11_visual,
                )
            },
            #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
            RawDisplayHandle::Wayland(_) => {
                Self::get_wayland_window_attributes(identity, window_config, activation_token)
            },
            _ => unreachable!(),
        }
    }

    #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
    fn get_wayland_window_attributes(
        identity: &Identity,
        _window_config: &WindowConfig,
        activation_token: Option<ActivationToken>,
    ) -> WindowAttributes {
        let mut attrs = WindowAttributesWayland::default()
            .with_name(&identity.class.general, &identity.class.instance);

        if let Some(activation_token) = activation_token {
            attrs = attrs.with_activation_token(activation_token);
        }

        WindowAttributes::default().with_platform_attributes(Box::new(attrs))
    }

    #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
    fn get_x11_window_attributes(
        identity: &Identity,
        _window_config: &WindowConfig,
        activation_token: Option<ActivationToken>,
        x11_visual: Option<X11VisualInfo>,
    ) -> WindowAttributes {
        use winit::platform::x11::WindowType;

        let mut decoder = png::Decoder::new(Cursor::new(WINDOW_ICON));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder.read_info().expect("invalid embedded icon");
        let mut buf = vec![0; reader.output_buffer_size()];
        let _ = reader.next_frame(&mut buf);
        let reader_info = reader.info();
        let icon = RgbaIcon::new(buf, reader_info.width, reader_info.height)
            .expect("invalid embedded icon format")
            .into();

        let mut attrs = WindowAttributesX11::default()
            .with_x11_visual(x11_visual.unwrap().visual_id() as _)
            .with_name(&identity.class.general, &identity.class.instance)
            .with_x11_window_type(vec![WindowType::Utility])
            // .with_override_redirect(true) // this keeps it above in case we want that
            ;

        if let Some(activation_token) = activation_token {
            attrs = attrs.with_activation_token(activation_token);
        }

        WindowAttributes::default()
            .with_window_icon(Some(icon))
            .with_platform_attributes(Box::new(attrs))
    }

    #[cfg(windows)]
    pub fn get_platform_window_attributes(
        _: &Identity,
        window_config: &WindowConfig,
    ) -> WindowAttributes {
        let icon = WinIcon::from_resource(IDI_ICON, None).ok().map(Into::into);
        let attrs = WindowAttributesWindows::default().with_taskbar_icon(icon.clone());

        WindowAttributes::default()
            .with_decorations(window_config.decorations != Decorations::None)
            .with_window_icon(icon)
            .with_platform_attributes(Box::new(attrs))
    }

    #[cfg(target_os = "macos")]
    pub fn get_platform_window_attributes(
        _: &Identity,
        window_config: &WindowConfig,
    ) -> WindowAttributes {
        let attrs = WindowAttributesMacOS::default()
            .with_option_as_alt(window_config.option_as_alt())
            .with_titlebar_hidden(true)
            .with_panel(true);

        WindowAttributes::default().with_platform_attributes(Box::new(attrs))
    }

    pub fn set_urgent(&self, is_urgent: bool) {
        let attention = if is_urgent { Some(UserAttentionType::Critical) } else { None };

        self.window.request_user_attention(attention);
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn set_transparent(&self, transparent: bool) {
        self.window.set_transparent(transparent);
    }

    pub fn set_blur(&self, blur: bool) {
        self.window.set_blur(blur);
    }

    pub fn set_resize_increments(&self, increments: PhysicalSize<f32>) {
        self.window.set_surface_resize_increments(Some(increments.into()));
    }

    pub fn set_maximized(&self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    pub fn set_minimized(&self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    // /// Toggle the window's fullscreen state.
    // pub fn toggle_fullscreen(&self) {
    //     self.set_fullscreen(self.window.fullscreen().is_none());
    // }

    /// Toggle the window's maximized state.
    pub fn toggle_maximized(&self) {
        self.set_maximized(!self.window.is_maximized());
    }

    // pub fn set_fullscreen(&self, fullscreen: bool) {
    //     if fullscreen {
    //         self.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
    //     } else {
    //         self.window.set_fullscreen(None);
    //     }
    // }

    /// Inform windowing system about presenting to the window.
    ///
    /// Should be called right before presenting to the window with e.g. `eglSwapBuffers`.
    pub fn pre_present_notify(&self) {
        self.window.pre_present_notify();
    }

    #[cfg(target_os = "macos")]
    pub fn set_option_as_alt(&self, option_as_alt: OptionAsAlt) {
        self.window.set_option_as_alt(option_as_alt);
    }

    pub fn current_monitor(&self) -> Option<MonitorHandle> {
        self.window.current_monitor()
    }

    /// Set IME inhibitor state and disable IME while any are present.
    ///
    /// IME is re-enabled once all inhibitors are unset.
    pub fn set_ime_inhibitor(&mut self, inhibitor: ImeInhibitor, inhibit: bool) {
        if self.ime_inhibitor.contains(inhibitor) != inhibit {
            self.ime_inhibitor.set(inhibitor, inhibit);
            #[allow(deprecated)]
            self.window.set_ime_allowed(self.ime_inhibitor.is_empty());
        }
    }

    /// Adjust the IME editor position according to the new location of the cursor.
    pub fn update_ime_position(&self, point: Point<usize>, size: &SizeInfo) {
        // NOTE: X11 doesn't support cursor area, so we need to offset manually to not obscure
        // the text.
        let offset = if self.is_x11 { 1 } else { 0 };
        let nspot_x = f64::from(size.padding_x() + point.column.0 as f32 * size.cell_width());
        let nspot_y =
            f64::from(size.padding_y() + (point.line + offset) as f32 * size.cell_height());

        // NOTE: some compositors don't like excluding too much and try to render popup at the
        // bottom right corner of the provided area, so exclude just the full-width char to not
        // obscure the cursor and not render popup at the end of the window.
        let width = size.cell_width() as f64 * 2.;
        let height = size.cell_height as f64;

        #[allow(deprecated)]
        self.window.set_ime_cursor_area(
            PhysicalPosition::new(nspot_x, nspot_y).into(),
            PhysicalSize::new(width, height).into(),
        );
    }
}

bitflags! {
    /// IME inhibition sources.
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ImeInhibitor: u8 {
        const FOCUS = 1;
        const TOUCH = 1 << 1;
        const VI    = 1 << 2;
    }
}

#[cfg(target_os = "macos")]
fn use_srgb_color_space(window: &dyn WinitWindow) {
    let view = match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::AppKit(handle) => {
            assert!(MainThreadMarker::new().is_some());
            unsafe { handle.ns_view.cast::<NSView>().as_ref() }
        },
        _ => return,
    };

    view.window().unwrap().setColorSpace(Some(&NSColorSpace::sRGBColorSpace()));
}
