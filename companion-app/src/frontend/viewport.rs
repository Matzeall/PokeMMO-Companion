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
    use super::*;
    // windows only imports

    use winit::window::Window;
    use std::sync::mpsc::Sender;
    use ::windows::Win32::{
        Foundation::HWND,
        UI::{
            Input::KeyboardAndMouse::{MOD_ALT, RegisterHotKey, VK_C, VK_F, VK_V},
            WindowsAndMessaging::{
                DispatchMessageW, GWL_EXSTYLE, GetDesktopWindow, GetMessageW, GetWindowLongW, MSG,
                SW_HIDE, SW_SHOWMAXIMIZED, SetForegroundWindow, SetWindowLongW, ShowWindow,
                TranslateMessage, WM_HOTKEY, WS_EX_LAYERED, WS_EX_TRANSPARENT,
            },
        },
    };

    /// manages the focus state of the main window by calling Win32 native functionality like
    /// RegisterHotKey and the Windows Event Loop
    pub struct NativeViewportManagerWin32 {
        app_focus: FocusState,
        focus_state_rx: Option<Receiver<FocusState>>,
        hwnd_int: isize,
        winit_window: Arc<Window>,
    }

    impl NativeViewportManagerWin32 {
        pub fn new(window_handle: WindowHandle<'_>, winit_window: Arc<Window>) -> Self {
            let mut manager = Self {
                app_focus: FocusState::Focused,
                focus_state_rx: None,
                hwnd_int: 0,
                winit_window,
            };

            match window_handle.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(raw_handle) => {
                    let hwnd_int = raw_handle.hwnd.get(); // isize is thread safe, pointer not
                    manager.hwnd_int = hwnd_int;
                    manager.focus_state_rx = Some(manager.spawn_hotkey_listener_thread());
                }
                _ => println!(
                    "Error setting up the Listener-thread (no Win32 window handle). \nHotKeys to bring back focus, will not work!"
                ),
            }

            manager.winit_window.set_decorations(false);

            manager
        }

        fn spawn_hotkey_listener_thread(&self) -> Receiver<FocusState> {
            // let egui_ctx = cc.egui_ctx.clone();
            // let winit_window = self.winit_window.clone();
            let hwnd_int = self.hwnd_int;
            let (focus_state_tx, focus_state_rx): (Sender<FocusState>, Receiver<FocusState>) =
                mpsc::channel();

            thread::spawn(move || unsafe {
                let hwnd = HWND(hwnd_int as *mut _);

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
                    if msg.message == WM_HOTKEY {
                        let key_id = msg.wParam.0;
                        // println!("Pressed HotKey ({key_id})");
                        match key_id {
                            1 => {
                                println!("Focus Overlay");

                                show_window_maximized(hwnd);

                                // winit_window.set_visible(true);
                                // winit_window.set_maximized(true);
                                // winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                                // winit_window.set_transparent(false);
                                // winit_window.set_cursor_hittest(true);
                                // winit_window.focus_window();
                                // winit_window.request_redraw();

                                let _ = focus_state_tx.send(FocusState::Focused); // notify main thread 

                                // egui_ctx
                                //     .send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                                // egui_ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                                // egui_ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                //     egui::WindowLevel::AlwaysOnTop,
                                // ));
                                // egui_ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(
                                //     false,
                                // ));
                                // egui_ctx.send_viewport_cmd(
                                //     egui::ViewportCommand::MousePassthrough(false),
                                // );
                                // egui_ctx.request_repaint();
                            }
                            2 => {
                                println!("Close Overlay");

                                hide_window(hwnd);

                                // winit_window.set_visible(false);
                                // winit_window.request_redraw();

                                let _ = focus_state_tx.send(FocusState::Hidden); // notify main thread 

                                // egui_ctx
                                //     .send_viewport_cmd(egui::ViewportCommand::Visible(false));
                                // egui_ctx.request_repaint();
                            }
                            3 => {
                                println!("Make Overlay non-interactable");

                                enable_overlay_click_through(hwnd);

                                // winit_window.set_visible(true);
                                // winit_window.set_maximized(true);
                                // winit_window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                                // winit_window.set_transparent(true);
                                // winit_window.set_cursor_hittest(false);
                                // winit_window.focus_window();
                                // winit_window.request_redraw();

                                let _ = focus_state_tx.send(FocusState::Unfocused); // notify main thread 

                                // egui_ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(
                                //     true,
                                // ));
                                // egui_ctx.send_viewport_cmd(
                                //     egui::ViewportCommand::MousePassthrough(true),
                                // );
                            }
                            _ => {}
                        }
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            });
            focus_state_rx
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
            //TODO: once eframe bug is resolved, implement singular means of switching between
            // focus_states to avoid desyncs, but since switch must be done partially on the thread for
            // now only update the result
        }

        fn current_focus_state(&self) -> FocusState {
            self.app_focus
        }
    }

    fn show_window_maximized(hwnd: HWND) {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED);
            let _ = SetForegroundWindow(hwnd);
            disable_overlay_click_through(hwnd);
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
            // TODO: maybe even search for the PokeMMO binary running and focus that?
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
