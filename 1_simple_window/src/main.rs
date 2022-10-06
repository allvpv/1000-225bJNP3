use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::ValidateRect,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class = s!("WinAPI madness");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_HELP)?,
            hInstance: instance,
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(some_window),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class,
            s!("This is a sample window"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        );

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            match DispatchMessageA(&message) {
                LRESULT(0) => {
                    println!("Returning OK");
                }
                LRESULT(_) => {
                    println!("Returning with error");
                    return Err(Error::from(E_FAIL));
                }
            }
        }

        match message.wParam {
            WPARAM(0) => Ok(()),
            WPARAM(_) => Err(Error::from(E_UNEXPECTED)),
        }
    }
}

extern "system" fn some_window(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => {
                println!("WM_CLOSE");
                let confirmation_message = w!("Are you sure?");
                let window_title = w!("Want to quit?");

                match MessageBoxW(None, confirmation_message, window_title, MB_OKCANCEL) {
                    IDOK => match DestroyWindow(window) {
                        BOOL(0) => panic!(),
                        BOOL(_) => LRESULT(0),
                    },
                    IDCANCEL => LRESULT(0),
                    _ => LRESULT(1),
                }
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
