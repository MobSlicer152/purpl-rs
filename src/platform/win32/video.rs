use log::{debug, info};
use std::mem;
use std::ptr;
use winapi::shared::windef;
use winapi::um::errhandlingapi;
use winapi::um::libloaderapi;
use winapi::um::winuser;

pub struct State {
    window: windef::HWND,
    title: String,
    width: u32,
    height: u32,
    resized: bool,
    focused: bool,
    closed: bool,
}

const IDI_ICON1: u32 = 103;

// static mut WND: windef::HWND = ptr::null_mut();

const WND_CLASS_NAME: &str = "PurplWindow";

// static mut WND_TITLE: String = String::new();
// static mut WND_WIDTH: u32 = 0;
// static mut WND_HEIGHT: u32 = 0;

// static mut WND_RESIZED: bool = false;
// static mut WND_FOCUSED: bool = false;
// static mut WND_CLOSED: bool = false;

unsafe extern "system" fn wndproc(
    state: &mut State,
    msgwnd: windef::HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if state.window == ptr::null_mut() || msgwnd == state.window {
        match msg {
            winuser::WM_SIZE => {
                let mut client_area: windef::RECT = mem::zeroed();

                winuser::GetClientRect(msgwnd, ptr::addr_of_mut!(client_area));
                let new_width = (client_area.right - client_area.left) as u32;
                let new_height = (client_area.bottom - client_area.top) as u32;

                if new_width != state.width || new_height != state.height {
                    state.resized = true;
                    info!(
                        "Window resized from {}x{} to {}x{}",
                        state.width, state.width, new_width, new_height
                    );
                    state.width = new_width;
                    state.height = new_height;
                }

                0
            }
            winuser::WM_ACTIVATEAPP => {
                state.focused = wparam != 0;
                info!(
                    "Window {}",
                    if state.focused {
                        "focused"
                    } else {
                        "unfocused"
                    }
                );
                0
            }
            winuser::WM_DESTROY | winuser::WM_CLOSE => {
                info!("Window closed");
                state.closed = true;
                0
            }
            _ => winuser::DefWindowProcA(msgwnd, msg, wparam, lparam),
        }
    } else {
        winuser::DefWindowProcA(msgwnd, msg, wparam, lparam)
    }
}

unsafe fn register_wndclass() {
    let mut wnd_class: winuser::WNDCLASSEXA = mem::zeroed();
    let base_addr = libloaderapi::GetModuleHandleA(ptr::null_mut());

    debug!("Registering window class");

    wnd_class.cbSize = mem::size_of::<winuser::WNDCLASSEXA>() as u32;
    wnd_class.lpfnWndProc = Some(wndproc);
    wnd_class.hInstance = base_addr;
    wnd_class.hCursor = winuser::LoadCursorA(ptr::null_mut(), winuser::IDC_ARROW as *const i8);
    wnd_class.hIcon = winuser::LoadIconA(base_addr, IDI_ICON1 as *const i8);
    wnd_class.lpszClassName = WND_CLASS_NAME.as_ptr() as *const i8;
    if winuser::RegisterClassExA(ptr::addr_of_mut!(wnd_class)) == 0 {
        let err = errhandlingapi::GetLastError();
        panic!(
            "Failed to register window class: error 0x{:X} ({})",
            err, err
        );
    }

    debug!("Window class registered");
}

impl State {
    fn init_wnd() -> Self {
        let mut client_area: windef::RECT = mem::zeroed();
        let base_addr = libloaderapi::GetModuleHandleA(ptr::null_mut());

        client_area.left = 0;
        client_area.right = (winuser::GetSystemMetrics(winuser::SM_CXSCREEN) as f32 / 1.5) as i32;
        client_area.top = 0;
        client_area.bottom = (winuser::GetSystemMetrics(winuser::SM_CYSCREEN) as f32 / 1.5) as i32;
        winuser::AdjustWindowRect(
            ptr::addr_of_mut!(client_area),
            winuser::WS_OVERLAPPEDWINDOW,
            false as i32,
        );
        let width = (client_area.right - client_area.left) as u32;
        let height = (client_area.bottom - client_area.top) as u32;

        let title = std::format!(
            "{} v{}.{}.{} by {}",
            crate::GAME_NAME,
            crate::GAME_VERSION_MAJOR,
            crate::GAME_VERSION_MINOR,
            crate::GAME_VERSION_PATCH,
            crate::GAME_ORGANIZATION_NAME
        );
        debug!("Creating {width}x{height} window titled {title}");

        let window = winuser::CreateWindowExA(
            0,
            WND_CLASS_NAME.as_ptr() as *const i8,
            title.as_ptr() as *const i8,
            winuser::WS_OVERLAPPEDWINDOW,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            width as i32,
            height as i32,
            ptr::null_mut(),
            ptr::null_mut(),
            base_addr,
            ptr::null_mut(),
        );
        if window == ptr::null_mut() {
            let err = errhandlingapi::GetLastError();
            panic!("Failed to create window: error 0x{err:X} {err}");
        }

        winuser::GetClientRect(window, ptr::addr_of_mut!(client_area));
        let width = (client_area.right - client_area.left) as u32;
        let height = (client_area.bottom - client_area.top) as u32;

        debug!(
            "Successfully created window with handle 0x{:X}",
            window as usize
        );

        Self {
            window,
            title,
            width,
            height,
            resized: false,
            focused: true,
            closed: false,
        }
    }
    pub fn init() -> Self {
        info!("Windows video initialization started");

        register_wndclass();
        let state = Self::init_wnd();

        debug!("Showing window");
        winuser::ShowWindow(state.window, winuser::SW_SHOW);

        info!("Windows video initialization succeeded");

        state
    }

    pub fn update(&mut self) -> bool {
        let mut msg: winuser::MSG = mem::zeroed();

        while winuser::PeekMessageA(
            ptr::addr_of_mut!(msg),
            ptr::null_mut(),
            0,
            0,
            winuser::PM_REMOVE,
        ) != 0
        {
            winuser::TranslateMessage(ptr::addr_of_mut!(msg));
            winuser::DispatchMessageA(ptr::addr_of_mut!(msg));
        }

        !self.closed
    }
    pub fn shutdown(&self) {
        info!("Windows video shutdown started");

        debug!("Destroying window");
        winuser::DestroyWindow(self.window);

        info!("Windows video shutdown succeeded");
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn resized(&mut self) -> bool {
        let ret = self.resized;
        self.resized = false;
        ret
    }

    pub fn focused(&self) -> bool {
        self.resized
    }
}

// pub unsafe fn init() {
//     info!("Windows video initialization started");

//     register_wndclass();
//     init_wnd();

//     debug!("Showing window");
//     winuser::ShowWindow(WND, winuser::SW_SHOW);

//     info!("Windows video initialization succeeded");
// }

// pub unsafe fn update() -> bool {
//     let mut msg: winuser::MSG = mem::zeroed();

//     while winuser::PeekMessageA(
//         ptr::addr_of_mut!(msg),
//         ptr::null_mut(),
//         0,
//         0,
//         winuser::PM_REMOVE,
//     ) != 0
//     {
//         winuser::TranslateMessage(ptr::addr_of_mut!(msg));
//         winuser::DispatchMessageA(ptr::addr_of_mut!(msg));
//     }

//     !WND_CLOSED
// }

// pub unsafe fn shutdown() {
//     info!("Windows video shutdown started");

//     debug!("Destroying window");
//     winuser::DestroyWindow(WND);

//     info!("Windows video shutdown succeeded");
// }

// pub unsafe fn get_size(mut width: &u32, mut height: &u32) {
//     width = &WND_WIDTH;
//     height = &WND_HEIGHT;
// }

// pub unsafe fn resized() -> bool {
//     let ret = WND_RESIZED;
//     WND_RESIZED = false;
//     ret
// }

// pub unsafe fn focused() -> bool {
//     WND_FOCUSED
// }
