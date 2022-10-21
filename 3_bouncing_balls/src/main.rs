use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::{
        Graphics::Gdi::{
            BeginPaint, CreatePen, CreateSolidBrush, Ellipse, EndPaint, FillRect, InvalidateRect,
            SelectObject, HBRUSH, HDC, PAINTSTRUCT, PS_SOLID,
        },
        System::LibraryLoader::GetModuleHandleA,
    },
};

const TIMER_ID: usize = 1337; /* Arbitrary nIDEvent value for timer */
const FPS: u32 = 60;

const DRAWING_PARAMS: AnimationParams = AnimationParams {
    width: 960,
    height: 720,
    speed: 1.,
    foreground: rgb::<100, 255, 100>(),
    background: rgb::<0, 0, 0>(),
    balls_count: 8,
    ground_fraction: 10,
    margin_fraction: 8,
    raise_fraction: 3,
    shuffle: 8. / 5.,
};

struct AnimationParams {
    width: i32,
    height: i32,
    speed: f64,
    foreground: COLORREF,
    background: COLORREF,
    balls_count: i32,
    ground_fraction: i32,
    margin_fraction: i32,
    raise_fraction: i32,
    shuffle: f64,
}

const fn rgb<const R: u8, const G: u8, const B: u8>() -> COLORREF {
    COLORREF(((B as u32) << 16) | ((G as u32) << 8) | (R as u32))
}

fn draw_straight_horizontal_line(brush: HBRUSH, hdc: HDC, y: i32, width: i32) {
    let rect = RECT {
        top: y,
        left: 0,
        right: width,
        bottom: y + 1,
    };

    unsafe {
        FillRect(hdc, &rect, brush);
    }
}

fn paint_ground(ps: &PAINTSTRUCT, brush: HBRUSH, width: i32, height: i32) -> i32 {
    let groundline_y = height - (height / DRAWING_PARAMS.ground_fraction);

    let mut y = groundline_y;
    let mut step = 1;

    while y < height {
        draw_straight_horizontal_line(brush, ps.hdc, y, width);
        y += step;
        step += (step + 1) / 2;
    }

    groundline_y
}

// x in [-1, 1]
fn y_circle(x: f64) -> f64 {
    (1. - x * x).sqrt()
}

fn get_width_height(window: HWND) -> (i32, i32) {
    let mut rect = RECT::default();

    unsafe {
        GetClientRect(window, &mut rect);
    }

    return (rect.right - rect.left, rect.bottom - rect.top);
}

fn paint_animation(window: HWND, frame: u32) {
    let (width, height) = get_width_height(window);

    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(window, &mut ps) };
    let background_brush = unsafe { CreateSolidBrush(DRAWING_PARAMS.background) };
    let foreground_brush = unsafe { CreateSolidBrush(DRAWING_PARAMS.foreground) };
    let pen = unsafe { CreatePen(PS_SOLID, 1, DRAWING_PARAMS.foreground) };

    let groundline = paint_ground(&ps, foreground_brush, width, height);

    unsafe {
        SelectObject(ps.hdc, background_brush);
        SelectObject(ps.hdc, pen);
    }

    let circles = DRAWING_PARAMS.balls_count;
    let margin = width / DRAWING_PARAMS.margin_fraction;
    let ellipse_diameter = (width - 2 * margin) / circles;

    let mut next_ellipse_start = margin;

    let frames = (FPS as f64 / DRAWING_PARAMS.speed) as u32;
    let frame_no = frame % frames;

    for i in 0..circles {
        let circle_shift = i as f64 / (circles - 1) as f64;
        let animation_shift = frame_no as f64 / (frames - 1) as f64;
        let vertical_shift = (circle_shift * DRAWING_PARAMS.shuffle + animation_shift) % 1.;

        let max_raise_pixels = height / DRAWING_PARAMS.raise_fraction;
        let raise_factor = y_circle(vertical_shift * 2. - 1.0);
        let raise_pixels = (max_raise_pixels as f64 * raise_factor) as i32;
        let rectangle_bottom = groundline - raise_pixels;

        let left = next_ellipse_start;
        let top = rectangle_bottom - ellipse_diameter;
        let right = next_ellipse_start + ellipse_diameter;
        let bottom = rectangle_bottom;

        unsafe {
            Ellipse(hdc, left, top, right, bottom);
        }

        next_ellipse_start = right;
    }

    unsafe {
        EndPaint(window, &ps);
    }
}

struct AnimWinState {
    current_frame_num: u32,
}

static mut ANIM_WIN_STATE: AnimWinState = AnimWinState {
    current_frame_num: 0,
};

extern "system" fn animation_window(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                paint_animation(window, ANIM_WIN_STATE.current_frame_num);
                LRESULT(0)
            }
            WM_DESTROY => {
                if KillTimer(window, TIMER_ID) == BOOL(0) {
                    panic!("Cannot kill timer");
                }
                /* Post quit message with status 0 to main process loop */
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_CLOSE => match DestroyWindow(window) {
                BOOL(0) => panic!(),
                BOOL(_) => LRESULT(0),
            },
            WM_TIMER => {
                ANIM_WIN_STATE.current_frame_num += 1;
                InvalidateRect(window, None, true);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        /* Get handle to the current process .exe file. */
        let instance = GetModuleHandleA(None)?;
        let window_class_name = s!("GDI animation");

        let wc = WNDCLASSA {
            hInstance: instance,
            lpszClassName: window_class_name,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(animation_window),
            hbrBackground: CreateSolidBrush(DRAWING_PARAMS.background),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        assert!(atom != 0);

        let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE & !WS_THICKFRAME;
        let ex_style = WINDOW_EX_STYLE::default();
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: DRAWING_PARAMS.width,
            bottom: DRAWING_PARAMS.height,
        };

        AdjustWindowRectEx(&mut rect, style, false, ex_style);

        let window_handle = CreateWindowExA(
            ex_style,
            window_class_name,
            s!("GDI Animation"),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            instance,
            None,
        );

        SetTimer(window_handle, TIMER_ID, 1000 / FPS, None);

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
