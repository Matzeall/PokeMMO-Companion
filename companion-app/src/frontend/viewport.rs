#![allow(unused_imports)]

use std::{
    sync::{mpsc::{self, Receiver}, Arc},
    thread,
};

use eframe::Frame;

use egui::Context;
use raw_window_handle::{WindowHandle,DisplayHandle};

use crate::frontend::style;

/// Focused when interacting with any of the windows
/// Unfocused when seeing the overlay but interacting with something underneath
/// Hidden when the whole overlay is hidden and needs to be manually unhidden before seeing anything
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum FocusState {
    Focused,
    Unfocused,
    Hidden,
}

#[allow(dead_code)]
impl FocusState {
    pub fn is_focused(&self) -> bool {
        *self == FocusState::Focused
    }
    pub fn is_unfocused(&self) -> bool {
        *self == FocusState::Unfocused
    }
    pub fn is_hidden(&self) -> bool {
        *self == FocusState::Hidden
    }
}

pub trait ViewportManager {
    fn update_viewport(&mut self, ctx: &Context, frame: &mut Frame); // needs to be called each frame
    fn current_focus_state(&self) -> FocusState;
    fn window_background_color(&self) -> egui::Rgba { egui::Rgba::TRANSPARENT       /*  style::COLOR_BG_NON_OVERLAY.into() */ }
    fn should_draw_gui(&self) -> bool {true}

    // fn setup_focused_mode(&self);   // INFO: not easily possible because communication between
    // fn setup_closed_mode(&self);    // listener thread and main thread is limitied, because of
    // fn setup_click_through_mode(&self); // eframe bug -> no ticks when window is minimized
    //                                    // for now let thread directly handle window state
}
// doesn't manage anything -> viewport stays focused forever
#[derive(Default)]
pub struct DefaultViewportManager {
    initialized: bool ,
}
impl ViewportManager for DefaultViewportManager {
    fn update_viewport(&mut self, _ctx: &Context, _frame: &mut Frame) {
        if !self.initialized {
            // normal window -> make draggable etc for the user
            _ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true)); 
            self.initialized = true;
        }
    }

    fn current_focus_state(&self) -> FocusState {
        FocusState::Focused
    }

    fn window_background_color(&self) -> egui::Rgba {
        style::COLOR_BG_NON_OVERLAY.into()
    }
}

#[cfg(windows)]
pub mod windows {
    use crate::{app::APP_ID, utils::convert_cyrillic_string};

    use super::*;
    // windows only imports

    use winit::window::Window;
    use std::{ffi::OsString, os::windows::ffi::OsStringExt, sync::mpsc::Sender, time::{Duration, Instant}};
    use ::windows::{core::BOOL, Win32::{
        Foundation::{HWND, LPARAM}, Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST}, UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, VK_C, VK_F, VK_V}, WindowsAndMessaging::{
                DispatchMessageW, EnumWindows, GetDesktopWindow, GetMessageW, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, 
                IsWindow, IsWindowVisible, SetForegroundWindow, SetWindowLongW, SetWindowPos, ShowWindow, TranslateMessage, GWL_EXSTYLE,
                MSG, SWP_FRAMECHANGED, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_RESTORE, SW_SHOW, SW_SHOWMAXIMIZED, WM_HOTKEY, WS_EX_LAYERED, WS_EX_TRANSPARENT
            }
        }
    }};

    /// manages the focus state of the main window by calling Win32 native functionality like
    /// RegisterHotKey and the Windows Event Loop
    pub struct NativeViewportManagerWin32 {
        app_focus: FocusState,

        focus_state_rx: Option<Receiver<FocusState>>,
        pokemmo_window_tx: Option<Sender<Option<isize>>>,
        
        overlay_hwnd_int: isize,
        pokemmo_hwnd_int: Option<isize>,
        winit_window: Arc<Window>,

        last_run_window_update: Instant,
    }

    impl NativeViewportManagerWin32 {
        pub fn new(window_handle: WindowHandle<'_>, winit_window: Arc<Window>) -> Self {
            let mut manager = Self {
                app_focus: FocusState::Focused,
                focus_state_rx: None,
                pokemmo_window_tx: None,
                overlay_hwnd_int: 0,
                pokemmo_hwnd_int:None,
                winit_window,
                last_run_window_update: Instant::now(),
            };
            
            match window_handle.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(raw_handle) => {
                    manager.overlay_hwnd_int = raw_handle.hwnd.get(); // isize is thread safe, pointer not
                    manager.pokemmo_hwnd_int = find_pokemmo_window_via_iteration().map(|hwnd| hwnd.0 as isize); 

                    let (focus_rx, pokemmo_hwnd_tx)= manager.spawn_hotkey_listener_thread();

                    manager.focus_state_rx = Some(focus_rx); // focus state update receiver
                    manager.pokemmo_window_tx = Some(pokemmo_hwnd_tx); // hwnd updater
                }
                _ => println!(
                    "Error setting up the Listener-thread (no Win32 window handle). \nHotKeys to bring back focus, will not work!"
                ),
            }

            manager.winit_window.set_decorations(false);

            manager
        }


        fn spawn_hotkey_listener_thread(&self) -> (Receiver<FocusState>, Sender<Option<isize>>) {
            let overlay_hwnd_int = self.overlay_hwnd_int;
            let pokemmo_hwnd_int= self.pokemmo_hwnd_int;

            let (focus_state_tx, focus_state_rx): (Sender<FocusState>, Receiver<FocusState>) =
                mpsc::channel();
            let (pokemmo_window_tx, pokemmo_window_rx): (Sender<Option<isize>>, Receiver<Option<isize>>) =
                mpsc::channel();


            thread::spawn(move || unsafe {
                let overlay_hwnd = HWND(overlay_hwnd_int as *mut _);
                let mut pokemmo_hwnd =  pokemmo_hwnd_int.map(|i| HWND(i as *mut _));

                // register hotkeys on this thread
                RegisterHotKey(None, 1, MOD_ALT, VK_F.0 as u32)
                    .expect("failed to register hotkey for focusing");
                RegisterHotKey(None, 2, MOD_ALT, VK_C.0 as u32)
                    .expect("failed to register hotkey for closing");
                RegisterHotKey(None, 3, MOD_ALT, VK_V.0 as u32)
                    .expect("failed to register hotkey for closing");

                // thread has it's own Event loop only listening to all global HotKeys
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    // receive most recent pokemmo window hwnd
                    while let Ok(hwnd_maybe) = pokemmo_window_rx.try_recv() {
                        pokemmo_hwnd = hwnd_maybe.map(|i|  HWND(i as *mut _));
                        println!("received new pokemmo hwnd: {:?}", pokemmo_hwnd.map(|h| h.0 as isize));
                    }

                    if msg.message == WM_HOTKEY {
                        let key_id = msg.wParam.0;
                        // println!("Pressed HotKey ({key_id})");
                        match key_id {
                            1 => {
                                println!("Focus Overlay");

                                show_window_maximized(overlay_hwnd,true);
                                disable_overlay_click_through(overlay_hwnd);

                                let _ = focus_state_tx.send(FocusState::Focused); // notify main thread 
                            }
                            2 => {
                                println!("Close Overlay");

                                hide_window(overlay_hwnd);

                                let _ = focus_state_tx.send(FocusState::Hidden); // notify main thread 
                            }
                            3 => {
                                println!("Make Overlay non-interactable");

                                show_window_maximized(overlay_hwnd,true);
                                enable_overlay_click_through(overlay_hwnd);
                                if let Some(pokemmo_hwnd) = pokemmo_hwnd {
                                    show_window_maximized(pokemmo_hwnd, false);
                                } 

                                let _ = focus_state_tx.send(FocusState::Unfocused); // notify main thread 
                            }
                            _ => {}
                        }
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            });
            (focus_state_rx,pokemmo_window_tx)
        }
    }

    impl ViewportManager for NativeViewportManagerWin32 {
        fn update_viewport(&mut self, _ctx: &Context, _frame: &mut Frame) {
            // update app focus from thread messages
            if let Some(rx) = &self.focus_state_rx {
                while let Ok(focus_update) = rx.try_recv() {
                    self.app_focus = focus_update;
                }
            }

            // occasionally refresh pokemmo window reference and move overlay to the same monitor
            let now = Instant::now();
            if now.duration_since(self.last_run_window_update)>= Duration::from_secs(1) {
                self.last_run_window_update = now;

                if is_window_alive(self.pokemmo_hwnd_int.map(|i| HWND(i as *mut _))) {
                    let pokemmo_window: HWND = self.pokemmo_hwnd_int.map(|i| HWND(i as *mut _))
                        .expect("pokemmo hwnd was \"None\" although it's window was checked before => not possible");
                    let overlay_window: HWND = HWND(self.overlay_hwnd_int as *mut _);

                    let pokemmo_monitor = unsafe { MonitorFromWindow(pokemmo_window, MONITOR_DEFAULTTONEAREST)};
                    let overlay_monitor = unsafe { MonitorFromWindow(overlay_window, MONITOR_DEFAULTTONEAREST)};

                    if overlay_monitor != pokemmo_monitor {
                        println!("switching overlay monitor to pokemmo monitor ({:?} => {:?})", overlay_monitor, pokemmo_monitor);
                        maximize_on_target_monitor(overlay_window, pokemmo_monitor);
                    }
                } else {
                    let window_opt= find_pokemmo_window_via_iteration().map(|hwnd| hwnd.0 as isize);
                    if window_opt != self.pokemmo_hwnd_int {
                        self.pokemmo_hwnd_int = window_opt;
                        if let Some(tx) = &self.pokemmo_window_tx {
                            let _ = tx.send(window_opt);
                        }
                    }
                }
            }
        }

        fn current_focus_state(&self) -> FocusState {
            self.app_focus
        }
    }

    fn maximize_on_target_monitor(overlay_hwnd: HWND, target_monitor: HMONITOR)   {
        unsafe {
            // needs to be initialized specially
            let mut monitor_info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as _,
                ..Default::default()
            };

            let _ = GetMonitorInfoW(target_monitor, &mut monitor_info);
      
            // this is necessary to clear the windows caches maximize size, 
            // otherwise the last call doesn't work correctly
            let _ = ShowWindow(overlay_hwnd, SW_RESTORE);

            // Move window onto target monitor (keeping size/Z-order)
            let workarea = monitor_info.rcWork;
            // println!("target_monitor workarea: {:?}",workarea);
            let _ = SetWindowPos(
                overlay_hwnd,
                None,
                workarea.left,
                workarea.top,
                workarea.right - workarea.left,
                workarea.bottom - workarea.top,
                SWP_NOSIZE | SWP_NOZORDER| SWP_FRAMECHANGED, // don't change and zorder
            );

            // maximize, because now the window is on the correct monitor
            let _ = ShowWindow(overlay_hwnd, SW_SHOWMAXIMIZED);
        }
    }


    /// Iterating over all top-level windows is the only way, since pokemmo randomely has cyrillic
    /// letters in their title, this way I can filter that out
    fn find_pokemmo_window_via_iteration() -> Option<HWND> {
        let mut found: Option<HWND> = None;

        // weird windows syntax, but it just iterates all top-level windows and handles output
        // through that LPARAM parameter
        unsafe {
            let _ = EnumWindows(
                Some(enum_is_pokemmo_window),
                LPARAM(&mut found as *mut _ as isize),
            );
        }

        if let Some(window)= &found {
            println!("found new pokemmo window: {}", window.0 as isize);
        }else {
            println!("pokemmo window not found...");
        }

        found
    }

    extern "system" fn enum_is_pokemmo_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            // skip hidden windows
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }

            // get the length of the title
            let len = GetWindowTextLengthW(hwnd);
            if len <= 0 {
                return BOOL(1);
            }

            // read into a temporary buffer
            let mut buf = vec![0u16; (len + 1) as usize];
            let read = GetWindowTextW(hwnd, &mut buf);
            if read <= 0 {
                return BOOL(1);
            }

            // convert and lowercase
            let title = OsString::from_wide(&buf[..read as usize])
                .to_string_lossy()
                .to_lowercase();

            // Weird edge case where the game title is in Cyrillic in some moments
            let cleaned_title = convert_cyrillic_string(title.as_str());
            // if cleaned_title.eq_ignore_ascii_case("pokemmo") && !cleaned_title.contains("companion") && !cleaned_title.contains(APP_ID) {
            if cleaned_title.eq_ignore_ascii_case("pokemmo") {
                // store and stop enumeration
                *(lparam.0 as *mut Option<HWND>) = Some(hwnd);
                return BOOL(0);
            }
        }
        // keep searching
        BOOL(1)
    }

    fn is_window_alive(hwnd_optional: Option<HWND>) -> bool {
        unsafe { IsWindow(hwnd_optional).as_bool() }
    }

    fn show_window_maximized(hwnd: HWND, show_maximized: bool) {
        let show_type = if show_maximized {SW_SHOWMAXIMIZED} else {SW_SHOW};
        unsafe {
            let _ = ShowWindow(hwnd, show_type);
            
            let _ = SetForegroundWindow(hwnd);
        }
    }

    fn hide_window(hwnd: HWND) {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }

    fn enable_overlay_click_through(hwnd: HWND) {
        unsafe {
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = (ex_style as u32 | WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0) as i32;
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
            // reset focus to some other window (desktop) to lose text edit keyboard focus e.g.
            let _ = SetForegroundWindow(GetDesktopWindow());
        }
    }
    fn disable_overlay_click_through(hwnd: HWND) {
        unsafe {
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = (ex_style as u32 & !WS_EX_TRANSPARENT.0) as i32;
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }
    }
}

#[cfg(unix)]
pub mod unix {
    use super::*;
    // unix only imports
    use std::error::Error;
    use std::io::{self, BufRead, BufReader};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::time::Duration;
    use raw_window_handle::{ RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
    use winit::window::Window;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask, GetKeyboardMappingReply, GrabMode, ModMask};
    use x11rb::protocol::Event;
    use xkeysym::{self, key};

    // use wayland_client::{
    //     Dispatch,
    //     QueueHandle,
    //     protocol::{
    //         wl_compositor::WlCompositor,
    //         wl_surface::WlSurface,
    //         wl_region::WlRegion,
    //         wl_registry::WlRegistry,
    //     },
    //     globals::{registry_queue_init, GlobalListContents},
    // };
    // use wayland_protocols::xdg::shell::client::{
    //     xdg_wm_base::XdgWmBase,
    //     xdg_surface::XdgSurface,
    //     xdg_toplevel::XdgToplevel,
    // };

    pub struct NativeViewportManagerX11 {
        app_focus: FocusState,
        focus_state_rx: Option<Receiver<FocusState>>,
    }

    impl NativeViewportManagerX11 {
        pub fn new(window_handle: WindowHandle<'_>) -> Self {
            let mut manager = Self {
                app_focus: FocusState::Focused,
                focus_state_rx: None,
            };

            match window_handle.as_raw() {
                raw_window_handle::RawWindowHandle::Xlib(_raw_handle) => {
                    // let hwnd_int = raw_handle.hwnd.get(); // isize is thread safe, pointer not
                    // manager.hwnd_int = hwnd_int;
                    match manager.spawn_hotkey_listener_thread() {
                        Ok(receiver) => manager.focus_state_rx = Some(receiver),
                        Err(boxed_err) => println!(
                            "Error during Hotkey registering({boxed_err}).\nNo Listener thread spawned\nHotkeys to bring back focus will not work"
                        ),
                    };
                }
                _ => println!(
                    "Error setting up the Listener-thread (no XLib window handle). \nHotKeys to bring back focus, will not work!"
                ),
            }

            manager
        }

        fn spawn_hotkey_listener_thread(&self) -> Result<Receiver<FocusState>, Box<dyn Error>> {
            // Connect to the X server
            let (conn, screen_num) = x11rb::connect(None)?;

            let screen = &conn.setup().roots[screen_num];
            let root = screen.root;
            println!("Root window = 0x{:X}", root);

            // Get KeyCodes for Alt+F/C/V
            let f_kc = keysym_to_keycode(&conn, key::f)?;
            let c_kc = keysym_to_keycode(&conn, key::c)?;
            let v_kc = keysym_to_keycode(&conn, key::v)?;
            let alt = ModMask::M1;

            // to notice a grabbed key I need to grab all variations with other mod keys that could
            // be simultaneously pressed (e.g. NumLock could be always on etc)
            let lock_masks = [
                ModMask::default(),          // no lock
                ModMask::LOCK,               // CapsLock
                ModMask::M2,                 // NumLock
                ModMask::LOCK | ModMask::M2, // both
            ];
            // Grab the keys
            for kc in [f_kc, c_kc, v_kc] {
                for &lock in &lock_masks {
                    println!(
                        "Keycode = {kc}  , ModMask = {:?} (bits = 0x{:X})",
                        ModMask::M1 | lock,
                        (ModMask::M1 | lock).bits()
                    );
                    match conn.grab_key(
                        false,
                        root,
                        alt | lock,
                        kc,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    ) {
                        Ok(cookie) => cookie.check(),
                        Err(err) => return Err(Box::new(err)),
                    }?;
                }
            }
            conn.change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::KEY_PRESS),
            )?
            .check()?;

            conn.flush()?;

            let (tx, rx) = mpsc::channel();

            println!("starting event loop thread");
            thread::spawn(move || {
                // X11 event loop
                loop {
                    println!("Event Loop: start waiting for something");
                    let event = conn.wait_for_event().unwrap();
                    println!("event : {:?}", event);
                    if let Event::KeyPress(kp) = event {
                        println!("key press: {:?}", kp);
                        match kp.detail {
                            d if d == f_kc => {
                                // focus window maximized
                                println!("Focus Overlay");

                                let _ = tx.send(FocusState::Focused);
                            }
                            d if d == c_kc => {
                                // close window
                                println!("Close Overlay");

                                let _ = tx.send(FocusState::Hidden);
                            }
                            d if d == v_kc => {
                                // focus window maximized
                                println!("Make Overlay non-interactive");

                                let _ = tx.send(FocusState::Unfocused);
                            }
                            _ => {}
                        }
                    }
                }
            });
            Ok(rx)
        }
    }

    impl ViewportManager for NativeViewportManagerX11 {
        fn update_viewport(&mut self, _ctx: &Context, _frame: &mut Frame) {
            // update app focus from thread messages
            if let Some(rx) = &self.focus_state_rx {
                while let Ok(focus_update) = rx.try_recv() {
                    self.app_focus = focus_update;
                }
            }
        }

        fn current_focus_state(&self) -> FocusState {
            self.app_focus
        }
    }

    // helper function to get the first x11 keycode matching a given keysym
    fn keysym_to_keycode<C: Connection>(conn: &C, keysym: u32) -> Result<u8, Box<dyn Error>> {
        // Query server’s keycode range
        let setup = conn.setup();
        let min_kc = setup.min_keycode;
        let max_kc = setup.max_keycode;
        let count = max_kc - min_kc + 1;

        // Fetch the entire keycode→keysym mapping
        let reply: GetKeyboardMappingReply = conn.get_keyboard_mapping(min_kc, count)?.reply()?;
        let per_code = reply.keysyms_per_keycode as usize;

        // Scan each chunk for the keysym
        for (i, chunk) in reply.keysyms.chunks(per_code).enumerate() {
            if chunk.contains(&keysym) {
                let keycode = min_kc + i as u8;
                println!("found keycode ({keycode}) for keysym ({keysym})");
                return Ok(keycode);
            }
        }

        Err(format!("No keycode found for keysym 0x{:X}", keysym).into())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // WAYLAND
    //////////////////////////////////////////////////////////////////////////////////////////

    pub struct NativeViewportManagerWayland {
        app_focus: FocusState,
        focus_state_rx: Option<Receiver<FocusState>>,
        hotkey_daemon_handle: Option<Child>, // used for later shutdown

        winit_window: Arc<Window>,

        // wayland specifics
        // _conn: wayland_client::Connection,
        // _qh: QueueHandle<WaylandEventQueue>,
        // compositor: WlCompositor,
        // surface:   WlSurface,
        // toplevel:  XdgToplevel,
    }

    // messages that will be received from the hotkey daemon
    const DAEMON_FOCUS_EVENT: &str = "focus";
    const DAEMON_CLOSE_EVENT: &str = "close";
    const DAEMON_VISIBLE_EVENT: &str = "visible";


    impl NativeViewportManagerWayland {
        /// if xdg-desktop-portal has extension global hotkeys -> register & listen to them
        /// else ask user if he wants to start sudo hotkey-daemon
        ///     -> granted : pkexec ./hotkey-daemon
        ///     -> canceled: return DefaultViewportManager
        pub fn try_new(window_handle: WindowHandle<'_>, display_handle: DisplayHandle, winit_window: Arc<Window>) -> Box<dyn ViewportManager> {
            let mut manager: Box<dyn ViewportManager> = Box::new(DefaultViewportManager::default());

            // try to make a valid native wayland overlay viewport -> maybe fallback to default
            let mut native_manager = Self {
                app_focus: FocusState::Focused,
                focus_state_rx: None,
                hotkey_daemon_handle: None,
                winit_window
            };
            native_manager.winit_window.set_decorations(true); // window needs to top bar in any case

            let _surface_ptr = match window_handle.as_raw() {
                RawWindowHandle::Wayland(WaylandWindowHandle { surface, .. }) => {
                    surface.as_ptr()  
                }
                _ => return manager,
            };

            let _display_ptr = match display_handle.as_raw() {
                RawDisplayHandle::Wayland(WaylandDisplayHandle { display, .. }) => {
                    display.as_ptr()
                }
                _ => return manager,
            };

            
            // if let Some(xdg_toplevel) = self.winit_window.xdg_toplevel() {
            //     xdg_toplevel
            // }

            // INFO: display handle and window handle won't get me what I want, because eframe and
            // winit completely abstract the wayland connection away, and there can only be one
            // connection to the wayland server (which is held by winit). 
            // I also cannot reconstruct the connection in any way.
            // The only way to manipulate the wayland connection is to go through eframe and winit,
            // both of which have issues preventing me from doing that.
            // -> only solution is to manually patch eframe to either:
            //      - expose winit window
            //      - fix update vs. tick issue, so I can go the viewport cmd approach


            // TODO: implement xdg-desktop-portal approach for hotkeys

            // TODO: hotkey-daemon approach for hotkeys (refactor into funciton)

            // first notify user blocking, why the hotkey-daemon is requesting sudo rights
            if let Err(e) = Command::new("zenity").stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null())
                .arg("--info")
                .arg("--text=On some linux desktop environments it is not possible to register global hotkeys. Wayland was detected as your DisplayCompositor, which prohibits global-hotkeys in general. One user-level approach would be to use the xdg-desktop-portal global hotkeys, but your desktop environment also didn't support it (e.g. GNOME DE supports it after version 48 (Ubuntu 25.04)). \nNow the only solution left for global hotkeys is a sudo key-reader (such programs can be security risk (key-loggers), so read my open source code and verify yourself). You will be prompted for sudo-privileges next. You can still deny the upcoming privileges request and use the app in a non-overlay way.")
                .status() 
            { println!("Something went wrong, trying to inform the user of the upcoming sudo privileges prompt, because : {e}")};

            
            match daemon_path() {
                Ok(daemon) => {
                    let uid = users::get_current_uid(); // needed for drop privileges
                    let gid = users::get_current_gid(); // needed for drop privileges


                    println!(
                        "Trying to start the hotkey-daemon (Path = {}) with (UID = {}, GID = {})",
                        daemon.display(),uid, gid
                    );

                    let start_daemon_in_bg_sh = format!(
                        "setsid {} --drop-uid {} --drop-gid {} \
                            </dev/null >/dev/null 2>&1 &",
                        daemon.display(), uid, gid
                    );

                    // build cmd line, pkexec asks for sudo privileges in a gui popup
                    let process = Command::new("pkexec")
                        .arg("sh")
                        .arg("-c")
                        .arg(start_daemon_in_bg_sh)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn();

                    match process {
                        Ok(mut handle) => {
                            // wait for pkexec to finished to see if sudo was granted
                            if let Ok(status) = handle.wait() {
                                if !status.success() {
                                    // user has denied sudo privileges
                                    // -> fallback to non-overlay
                                    println!(
                                        "User has not given sudo-privileges. Falling back to non-overlay window"
                                     );
                                    return manager; // fallback to DefaultViewportManager
                                }
                            }

                            native_manager.hotkey_daemon_handle = Some(handle); // save daemon process for later shutdown

                            match native_manager.spawn_hotkey_listener_thread() {
                                Ok(receiver) => {
                                    println!(
                                        "Listener thread was successfully setup. Waiting for hotkey socket now ..."
                                    );
                                    native_manager.focus_state_rx = Some(receiver);
                                    manager = Box::new(native_manager);

                                }
                                Err(e) => {
                                    println!(
                                        "Couldn't start thread for listening to hotkey-daemon, because : {e}"
                                    );
                                }
                            }
                        }
                        Err(e) => println!(
                            "Failed to start daemon: {}\nFalling back to non-overlay window",
                            e
                        ),
                    }
                }
                Err(e) => println!(
                    "Daemon path could not be resolved, because : {e}\nFalling back to non-overlay window"
                ),
            }

            manager
        }

        fn spawn_hotkey_listener_thread(&self) -> io::Result<Receiver<FocusState>> {
            let (focus_update_tx, focus_update_rx) = mpsc::channel();

            let winit_window = self.winit_window.clone();
            let socket_path = socket_path()?;

            thread::spawn(move || {
                // wait until socket is valid -> hotkey-daemon is running
                let socket_connection;
                loop {
                    if let Ok(conn) = UnixStream::connect(&socket_path) {
                        println!(
                            "\nHotkey-daemom must be running correctly. Found hotkey-daemon socket. Start listening now ..."
                        );
                        socket_connection = conn;
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
                }

                // connect to daamon, which is running now
                let reader = BufReader::new(socket_connection);
                for line in reader.lines() {
                    match line {
                        Ok(msg) => match msg.as_str() {
                            DAEMON_FOCUS_EVENT => {
                                println!("Received: Focus Overlay");
                                let _ = winit_window.set_cursor_hittest(true);
                                // hint is ignored by GNOME
                                // winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);

                                let _ = focus_update_tx.send(FocusState::Focused);
                            }
                            DAEMON_CLOSE_EVENT => {
                                println!("Received: Close Overlay");
                                let _ = winit_window.set_cursor_hittest(false);

                                let _ = focus_update_tx.send(FocusState::Hidden);
                            }
                            DAEMON_VISIBLE_EVENT => {
                                println!("Received: Make Overlay non-interactive (not supported)");
                                let _ = winit_window.set_cursor_hittest(false);
                                // hint is ignored by GNOME
                                // winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                                let _ = focus_update_tx.send(FocusState::Unfocused);
                            }
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("Socket read error: {}", e);
                            break;
                        }
                    }
                }
            });
            Ok(focus_update_rx)
        }
    }

    impl ViewportManager for NativeViewportManagerWayland {
        fn update_viewport(&mut self, _ctx: &Context, _frame: &mut Frame) {
            // update app focus from thread messages
            if let Some(rx) = &self.focus_state_rx {
                while let Ok(focus_update) = rx.try_recv() {
                    self.app_focus = focus_update;
                }
            }

        
            
            if self.winit_window.is_maximized() {
                self.winit_window.set_maximized(false);
            } else {
                // let monitor = self.winit_window
                //     .current_monitor()
                //     .or_else(|| self.winit_window.primary_monitor())
                //     .expect("no monitor");
                // // raw physical size of monitor
                // let size_px = monitor.size(); 
                //
                // // DPI scale
                // let scale = monitor.scale_factor();
                //
                // let window_bar_height = 100.; // no way to get the actual info
                // let _ = self.winit_window.request_inner_size(LogicalSize {
                //     width : size_px.width  as f64 / scale,
                //     height: size_px.height as f64 / scale - window_bar_height,
                // });
                // Not supported by wayland -> user has to position it manually...
                // self.winit_window.set_outer_position(monitor.position());
            }
        }

        fn current_focus_state(&self) -> FocusState {
            self.app_focus
        }

        fn should_draw_gui(&self) -> bool {
            self.app_focus != FocusState::Hidden
        }
    }

    impl Drop for NativeViewportManagerWayland {
        fn drop(&mut self) {
            if let Some(running_daemon) = &mut self.hotkey_daemon_handle {
                let id = running_daemon.id();
                println!("Shutting down running hotkey-daemon ({id}) ... ");
                match running_daemon.kill() {
                    Ok(()) => {
                        println!("successfully shut down.")
                    }
                    Err(e) => {
                        println!("couldn't shutdown daemon, because : {e}")
                    }
                }
            }
        }
    }

    fn daemon_path() -> io::Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| io::Error::other("Executable has no parent directory"))?;
        Ok(exe_dir.join("./hotkey-daemon"))
    }

    fn socket_path() -> io::Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| io::Error::other("Executable has no parent directory"))?;
        Ok(exe_dir.join("hotkeys.sock"))
    }
}
