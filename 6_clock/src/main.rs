use chrono::{NaiveTime, Timelike, Duration};
use rand::Rng;
use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::*,
            Direct2D::Common::*,
            Imaging::*,
            Imaging::D2D::*,
            Gdi::*,
        },
        System::{
            LibraryLoader::*,
            Performance::*,
            SystemServices::GENERIC_READ,
            Com::*,
        },
        UI::{
            HiDpi::*,
            WindowsAndMessaging::*,
        }
    },
};

const SECS_IN_DAY: u32 = 60 * 60 * 24;

struct Graphics {
    render_target: ID2D1HwndRenderTarget,
    digits_bitmap: ID2D1Bitmap,
    watch_bitmap: ID2D1Bitmap,
}

impl Graphics {
    fn get_translation_for_bitmap_centering(&self, bmp: &ID2D1Bitmap) -> Matrix3x2 {
        let translation = {
            let scrsize = unsafe { self.render_target.GetSize() };
            let bmpsize = unsafe { bmp.GetSize() };
            (
                (scrsize.width - bmpsize.width) / 2.,
                (scrsize.height - bmpsize.height) / 2.,
            )
        };

        Matrix3x2::translation(translation.0, translation.1)
    }

    fn get_translation_for_bitmap_rotation(&self, bmp: &ID2D1Bitmap, angle: f32) -> Matrix3x2 {
        let bmpsize = unsafe { bmp.GetSize() };
        Matrix3x2::rotation(angle, bmpsize.width / 2., bmpsize.height / 2.)
    }

    fn draw_watch(&self) {
        unsafe {
            self.render_target.DrawBitmap(
                &self.watch_bitmap,
                None,
                1.,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );
        }
    }

    const DIGIT_WIDTH: f32 = 108.;
    const DIGIT_OPACITY: f32 = 0.7;
    const WATCH_VERTICAL_MARGIN: f32 = 104.;
    const WATCH_HORIZONTAL_MARGIN: f32 = 119.;

    fn draw_separator(&self) {
        let digits_width = unsafe { self.digits_bitmap.GetSize().width };
        let digits_height = unsafe { self.digits_bitmap.GetSize().height };

        let source_rect = {
            let separator_begin = digits_width - 100.;
            let separator_end = digits_width;

            D2D_RECT_F {
                top: 0.,
                bottom: digits_height,
                left: separator_begin,
                right: separator_end,
            }
        };

        let target_rect = {
            let horizontal_margin = Self::WATCH_HORIZONTAL_MARGIN + Self::DIGIT_WIDTH * 2.;
            D2D_RECT_F {
                top: Self::WATCH_VERTICAL_MARGIN,
                bottom: Self::WATCH_VERTICAL_MARGIN + digits_height,
                left: horizontal_margin,
                right: horizontal_margin + 100.,
            }
        };

        unsafe {
            self.render_target.DrawBitmap(
                &self.digits_bitmap,
                Some(&target_rect),
                Self::DIGIT_OPACITY,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                Some(&source_rect),
            );
        }
    }

    fn draw_digit(&self, num: i32, pos: i32, separator: bool) {
        let digits_height = unsafe { self.digits_bitmap.GetSize().height };

        let source_rect = {
            let digit_begin = Self::DIGIT_WIDTH * num as f32;
            let digit_end = digit_begin + Self::DIGIT_WIDTH;

            D2D_RECT_F {
                top: 0.,
                bottom: digits_height,
                left: digit_begin,
                right: digit_end,
            }
        };

        let target_rect = {
            let horizontal_margin =
                Self::WATCH_HORIZONTAL_MARGIN + Self::DIGIT_WIDTH * pos as f32 + {
                    if separator {
                        100.
                    } else {
                        0.
                    }
                };

            D2D_RECT_F {
                top: Self::WATCH_VERTICAL_MARGIN,
                bottom: Self::WATCH_VERTICAL_MARGIN + digits_height,
                left: horizontal_margin,
                right: horizontal_margin + Self::DIGIT_WIDTH,
            }
        };

        unsafe {
            self.render_target.DrawBitmap(
                &self.digits_bitmap,
                Some(&target_rect),
                Self::DIGIT_OPACITY,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                Some(&source_rect),
            );
        }
    }

    fn render_time(&self, time: &NaiveTime) {
        let hour = time.hour() as i32;
        let minute = time.minute() as i32;

        self.draw_digit(hour / 10, 0, false);
        self.draw_digit(hour % 10, 1, false);
        self.draw_digit(minute / 10, 2, true);
        self.draw_digit(minute % 10, 3, true);
    }

    fn render(&self, time: &NaiveTime, draw_separator: bool) -> Result<()> {
        self.begin_draw();
        self.clear_screen(0.7, 0.7, 1.);

        let rotation_matrix = self.get_translation_for_bitmap_rotation(&self.watch_bitmap, -7.2);
        let watch_centering = self.get_translation_for_bitmap_centering(&self.watch_bitmap);

        self.draw_watch();
        self.render_time(time);

        if draw_separator {
            self.draw_separator();
        }

        unsafe {
            let final_matrix = rotation_matrix * watch_centering;
            self.render_target.SetTransform(&final_matrix);
        }

        self.end_draw()
    }

    fn load_bitmap_from_file(
        render_target: &ID2D1HwndRenderTarget,
        imaging_factory: &IWICImagingFactory2,
        uri: PCWSTR,
    ) -> Result<ID2D1Bitmap> {
        let decoder = unsafe {
            imaging_factory.CreateDecoderFromFilename(
                uri,
                None,
                GENERIC_READ,
                WICDecodeMetadataCacheOnLoad,
            )?
        };

        let frame_decoder = unsafe { decoder.GetFrame(0)? };
        let format_converter = unsafe { imaging_factory.CreateFormatConverter()? };

        unsafe {
            format_converter.Initialize(
                &frame_decoder,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.,
                WICBitmapPaletteTypeMedianCut,
            )?
        };

        unsafe { render_target.CreateBitmapFromWicBitmap(&format_converter, None) }
    }

    fn new(window: &Window) -> Result<Self> {
        let options = D2D1_FACTORY_OPTIONS::default();

        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options))? };

        let imaging_factory: IWICImagingFactory2 =
            unsafe { CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)? };

        let width = window.client_area_width as u32;
        let height = window.client_area_height as u32;

        let properties = D2D1_RENDER_TARGET_PROPERTIES::default();
        let hwnd_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: window.handle,
            pixelSize: D2D_SIZE_U { width, height },
            ..Default::default()
        };

        let render_target =
            unsafe { factory.CreateHwndRenderTarget(&properties, &hwnd_properties)? };

        let digits_bitmap =
            Self::load_bitmap_from_file(&render_target, &imaging_factory, w!("Digits.png"))?;

        let watch_bitmap =
            Self::load_bitmap_from_file(&render_target, &imaging_factory, w!("Watch.png"))?;

        Ok(Graphics {
            render_target,
            digits_bitmap,
            watch_bitmap,
        })
    }

    fn clear_screen(&self, r: f32, g: f32, b: f32) {
        unsafe {
            self.render_target
                .Clear(Some(&D2D1_COLOR_F { r, g, b, a: 1.0 }));
        }
    }

    fn begin_draw(&self) {
        unsafe {
            self.render_target.BeginDraw();
        }
    }

    fn end_draw(&self) -> Result<()> {
        unsafe { self.render_target.EndDraw(None, None) }
    }
}

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
    }

    let mut window = Window::new()?;
    window.run()
}

struct Timer {
    start_time: i64,
    update_time: i64,
    frequency: i64,
}

impl Timer {
    fn query_performance_frequency() -> Result<i64> {
        let mut frequency = 0;

        unsafe {
            QueryPerformanceFrequency(&mut frequency).ok()?;
        }

        Ok(frequency)
    }

    fn query_performance_counter() -> Result<i64> {
        let mut counter = 0;

        unsafe {
            QueryPerformanceCounter(&mut counter).ok()?;
        }

        Ok(counter)
    }

    fn new() -> Result<Self> {
        let frequency = Self::query_performance_frequency()?;
        let counter = Self::query_performance_counter()?;

        Ok(Timer {
            start_time: counter,
            update_time: counter,
            frequency,
        })
    }

    fn get_time(&self, period_in_seconds: u32) -> f64 {
        let delta = self.update_time.wrapping_sub(self.start_time);
        let period = self.frequency * period_in_seconds as i64;
        let time = delta % period;

        time as f64 / self.frequency as f64
    }

    fn update(&mut self) -> Result<()> {
        self.update_time = Self::query_performance_counter()?;
        Ok(())
    }
}

fn get_random_time() -> NaiveTime {
    let secs = rand::thread_rng().gen_range(0..SECS_IN_DAY);
    NaiveTime::from_num_seconds_from_midnight_opt(secs, 0).unwrap()
}

struct Window {
    handle: HWND,
    visible: bool,
    timer: Timer,
    client_area_width: i32,
    client_area_height: i32,
    graphics: Option<Graphics>,
    time: NaiveTime,
}


impl Window {
    fn new() -> Result<Self> {
        Ok(Window {
            handle: HWND(0),
            visible: false,
            timer: Timer::new()?,
            client_area_width: 0,
            client_area_height: 0,
            graphics: None,
            time: get_random_time(),
        })
    }

    fn call_render(&mut self) -> Result<()> {
        let timer_time = self.timer.get_time(SECS_IN_DAY / 4);
        let additional_seconds = (timer_time * 4.) as i64;
        let additional_duration = Duration::seconds(additional_seconds);
        let (total_time, _) = self.time.overflowing_add_signed(additional_duration);

        let draw_separator = additional_seconds % 2 == 0;

        if let Some(graphics) = &self.graphics {
            graphics.render(&total_time, draw_separator)?;
            self.timer.update()?;
        }

        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    BeginPaint(self.handle, &mut ps);

                    if self.graphics.is_some() {
                        self.call_render().ok();
                    }

                    EndPaint(self.handle, &ps);
                    LRESULT(0)
                }
                WM_ACTIVATE => {
                    self.visible = true; // TODO: unpack !HIWORD(wparam);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_SIZE => {
                    let mut rc = RECT::default();

                    if GetClientRect(self.handle, &mut rc).into() && self.graphics.is_some() {
                        let size = D2D_SIZE_U {
                            width: rc.right as u32,
                            height: rc.bottom as u32,
                        };

                        self.graphics
                            .as_mut()
                            .unwrap()
                            .render_target
                            .Resize(&size)
                            .unwrap();

                        LRESULT(0)
                    } else {
                        LRESULT(1)
                    }
                }
                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);
            let window_class = s!("Monster");

            let wc = WNDCLASSA {
                hCursor: LoadCursorW(None, IDC_HAND)?,
                hInstance: instance,
                lpszClassName: window_class,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            let handle = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                s!("Clock"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                Some(self as *mut _ as _),
            );

            assert!(handle.0 != 0);
            assert!(handle == self.handle);

            let mut client_rect = RECT {
                left: 0,
                top: 0,
                right: 1400,
                bottom: 600,
            };

            AdjustWindowRectEx(
                &mut client_rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WINDOW_EX_STYLE::default(),
            );

            let dpi = GetDpiForWindow(handle) as i32;

            self.client_area_width =
                ((((client_rect.right - client_rect.left) * dpi) as f32) / 96.0).ceil() as i32;
            self.client_area_height =
                ((((client_rect.bottom - client_rect.top) * dpi) as f32) / 96.0).ceil() as i32;

            SetWindowPos(
                handle,
                None,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                self.client_area_width,
                self.client_area_height,
                SWP_NOMOVE,
            );

            self.graphics = Some(Graphics::new(self)?);

            let mut message = MSG::default();

            loop {
                self.call_render()?;

                match PeekMessageA(&mut message, None, 0, 0, PM_REMOVE) {
                    BOOL(0) => continue,
                    BOOL(_) => (),
                }

                match message.message {
                    WM_QUIT => return Ok(()),
                    _ => DispatchMessageA(&message),
                };
            }
        }
    }

    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match message {
                WM_NCCREATE => {
                    let this = {
                        let cs = lparam.0 as *const CREATESTRUCTA;
                        (*cs).lpCreateParams as *mut Self
                    };

                    (*this).handle = window;
                    SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
                }
                _ => {
                    let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

                    if !this.is_null() {
                        return (*this).message_handler(message, wparam, lparam);
                    }
                }
            }

            DefWindowProcA(window, message, wparam, lparam)
        }
    }
}
