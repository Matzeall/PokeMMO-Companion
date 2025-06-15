use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use eframe::Frame;
use egui::Context;
use raw_window_handle::WindowHandle;
use windows::Win32::{
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
    // fn setup_focused_mode(&self);   // INFO: not easily possible because communication between
    // fn setup_closed_mode(&self);    // listener thread and main thread is limitied, because of
    // fn setup_click_through_mode(&self); // eframe bug -> no ticks when window is minimized
    //                                    // for now let thread directly handle window state
}
// doesn't manage anything -> viewport stays focused forever
#[derive(Default)]
pub struct DefaultViewportManager {}
impl ViewportManager for DefaultViewportManager {
    fn update_viewport(&mut self, _ctx: &Context, _frame: &mut Frame) {}

    fn current_focus_state(&self) -> FocusState {
        FocusState::Focused
    }
}

/// manages the focus state of the main window by calling Win32 native functionality like
/// RegisterHotKey and the Windows Event Loop
pub struct NativeViewportManagerWin32 {
    app_focus: FocusState,
    hwnd_int: isize,
    focus_state_rx: Option<Receiver<FocusState>>,
}

impl NativeViewportManagerWin32 {
    pub fn new(window_handle: WindowHandle<'_>) -> Self {
        let mut manager = Self {
            app_focus: FocusState::Focused,
            hwnd_int: 0,
            focus_state_rx: None,
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

        manager
    }

    fn spawn_hotkey_listener_thread(&self) -> Receiver<FocusState> {
        // let egui_ctx = cc.egui_ctx.clone();
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

                            let _ = focus_state_tx.send(FocusState::Hidden); // notify main thread 

                            // egui_ctx
                            //     .send_viewport_cmd(egui::ViewportCommand::Visible(false));
                            // egui_ctx.request_repaint();
                        }
                        3 => {
                            println!("Make Overlay non-interactable");

                            enable_overlay_click_through(hwnd);

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
