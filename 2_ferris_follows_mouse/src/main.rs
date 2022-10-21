use core::ffi::c_void;

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        CreatePatternBrush, DeleteObject, GetObjectA, UpdateWindow, ValidateRect, BITMAP, HBITMAP,
        HBRUSH, HGDIOBJ,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*,
};

const TIMER_ID: usize = 1234; /* Arbitrary nIDEvent value for timer */

struct WindowsBitmap(HBITMAP);

impl WindowsBitmap {
    unsafe fn from_file<STRLIKE>(filename: STRLIKE) -> Result<WindowsBitmap>
    where
        STRLIKE: Into<PCSTR>,
    {
        let handle = unsafe { LoadImageA(None, filename, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE)? };
        debug_assert!(handle != HANDLE(0)); /* I suppose that case is handled by Result type being
                                            returned from LoadImageA */
        Ok(WindowsBitmap(HBITMAP(handle.0)))
    }

    fn get(&self) -> HBITMAP {
        self.0
    }

    fn info(&self) -> Result<BITMAP> {
        let mut bitmap_info = BITMAP::default();
        let bitmap_info_ptr = &mut bitmap_info as *mut BITMAP;

        let bitmap_info_size: i32 = std::mem::size_of::<BITMAP>().try_into().unwrap();
        match unsafe {
            GetObjectA(
                self.0,
                bitmap_info_size,
                Some(bitmap_info_ptr as *mut c_void),
            )
        } {
            0 => Err(Error::new(
                E_FAIL,
                HSTRING::from("Cannot get info from bitmap"),
            )),
            _ => Ok(bitmap_info),
        }
    }
}

impl Drop for WindowsBitmap {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(HGDIOBJ::from(self.get()));
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        /* Load bitmap */
        let bitmap = WindowsBitmap::from_file(s!("ferris.bmp"))?;
        /* Get handle to the current process .exe file. */
        let instance = GetModuleHandleA(None)?;
        let window_class_name = s!("ferris");

        let bitmap_info = bitmap.info()?;
        println!("{}, {}", bitmap_info.bmWidth, bitmap_info.bmHeight);

        let brush = CreatePatternBrush(bitmap.get());
        assert_ne!(brush, HBRUSH(0));

        let wc = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(follow_mouse),
            hInstance: instance,
            hbrBackground: brush,
            lpszClassName: window_class_name,
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        assert!(atom != 0);

        let window_ex_style = WS_EX_TOPMOST | WS_EX_LAYERED;

        let window_handle = CreateWindowExA(
            window_ex_style,       /* window extended style */
            window_class_name,     /* - */
            s!("Mouse follower"),  /* window title */
            WS_POPUP | WS_VISIBLE, /* window style */
            CW_USEDEFAULT,         /* horizontal position */
            CW_USEDEFAULT,         /* vertical position */
            0,                     /* width */
            0,                     /* height */
            None,                  /* parent of window */
            None,                  /* handle to a menu */
            instance,              /* module associated w/ window */
            None,                  /* WM_CREATE window message */
        );

        ShowWindow(window_handle, SW_SHOW);
        UpdateWindow(window_handle);
        SetWindowPos(
            window_handle,
            HWND_TOPMOST,
            0,
            0,
            bitmap_info.bmWidth,
            bitmap_info.bmHeight,
            SWP_SHOWWINDOW,
        );

        SetLayeredWindowAttributes(window_handle, COLORREF(0), 255, LWA_ALPHA | LWA_COLORKEY);
        SetTimer(window_handle, TIMER_ID, 50, None);

        let mut message = MSG::default();

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            TranslateMessage(&message);
            match DispatchMessageA(&message) {
                LRESULT(0) => (),
                LRESULT(_) => return Err(Error::from(E_FAIL)),
            }
        }

        match message.wParam {
            WPARAM(0) => Ok(()),
            WPARAM(_) => Err(Error::from(E_UNEXPECTED)),
        }
    }
}

unsafe fn get_cursor_position() -> Result<POINT> {
    let mut point = POINT::default();
    match GetCursorPos(&mut point) {
        BOOL(0) => Err(Error::new(
            E_FAIL,
            HSTRING::from("Cannot get cursor position"),
        )),
        BOOL(_) => Ok(point),
    }
}

unsafe fn get_window_rectangle(hwnd: HWND) -> Result<RECT> {
    let mut rectangle = RECT::default();
    match GetWindowRect(hwnd, &mut rectangle) {
        BOOL(0) => Err(Error::new(
            E_FAIL,
            HSTRING::from("Cannot get window rectangle (position)"),
        )),
        BOOL(_) => Ok(rectangle),
    }
}

extern "system" fn follow_mouse(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                /* Post quit message with status 0 to main process loop */
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => match DestroyWindow(window) {
                BOOL(0) => panic!(),
                BOOL(_) => LRESULT(0),
            },
            WM_TIMER => {
                let cursor_position = get_cursor_position().unwrap();
                let window_rectangle = get_window_rectangle(window).unwrap();

                let width = window_rectangle.right - window_rectangle.left;
                let height = window_rectangle.bottom - window_rectangle.top;

                let newposx = cursor_position.x - width;
                let newposy = cursor_position.y - height;

                match SetWindowPos(
                    window,
                    HWND_TOPMOST,
                    (newposx + 9 * window_rectangle.left) / 10,
                    (newposy + 9 * window_rectangle.top) / 10,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER,
                ) {
                    BOOL(0) => LRESULT(1),
                    BOOL(_) => LRESULT(0),
                }
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
