use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*, Win32::Graphics::Gdi::*, Win32::System::Com::*,
    Win32::System::LibraryLoader::*, Win32::System::Performance::*, Win32::UI::HiDpi::*,
    Win32::UI::WindowsAndMessaging::*,
};

struct DrawingParams {
    function: fn(f64, f64) -> f64,
    x_points: usize,
    y_points: usize,
    spread: f64,
    x_shift: f64,
    y_shift: f64,
}

const PARAMS: DrawingParams = DrawingParams {
    function: |x: f64, y: f64| -> f64 { (10. * (x * x + y * y).sqrt()).cos() / 4. },
    x_points: 51,
    y_points: 51,
    spread: 200.,
    x_shift: 400.,
    y_shift: 350.,
};

type PlotArray = [[(f64, f64, f64); PARAMS.y_points]; PARAMS.x_points];

struct Plot {
    // Box to prevent stack overflow when x_points, y_points >~ 100
    array: Box<PlotArray>,
}

impl Plot {
    pub fn get_initial_plot<F>(function: F) -> Plot
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut array = Box::new([[(0., 0., 0.); PARAMS.y_points]; PARAMS.x_points]);

        for (i, row) in array.iter_mut().enumerate() {
            // map [0; Y_POINTS) -> [-1, 1]
            let y =
                ((i as i64 - (PARAMS.y_points / 2) as i64) as f64) / ((PARAMS.y_points / 2) as f64);

            for (j, value) in row.iter_mut().enumerate() {
                // map [0; X_POINTS) -> [-1, 1]
                let x = ((j as i64 - (PARAMS.x_points / 2) as i64) as f64)
                    / ((PARAMS.x_points / 2) as f64);
                let z = function(x, y);
                let (x_prim, y_prim) = Self::rotate_around_z(x, y, std::f64::consts::PI / 4.);
                *value = (x_prim, y_prim, z);
            }
        }

        Plot { array }
    }

    fn rotate_around_z(x: f64, y: f64, alpha: f64) -> (f64, f64) /* x, y */ {
        let x_prim = x * alpha.cos() - y * alpha.sin();
        let y_prim = x * alpha.sin() + y * alpha.cos();
        (x_prim, y_prim)
    }

    fn project_onto_plane(y: f64, z: f64, alpha: f64) -> f64 /* y */ {
        y * alpha.cos() - z * alpha.sin()
    }

    pub fn get_pixel_value(&self, i: usize, j: usize, alpha: f64) -> (f32, f32) {
        let (x, y_orig, z) = self.array[i][j];
        let y = Self::project_onto_plane(y_orig, z, alpha);

        let x_pixel = x * PARAMS.spread + PARAMS.x_shift;
        let y_pixel = y * PARAMS.spread + PARAMS.y_shift;

        (x_pixel as f32, y_pixel as f32)
    }
}

struct Graphics {
    render_target: ID2D1HwndRenderTarget,
    plot: Plot,
    brush: ID2D1SolidColorBrush,
}

impl Graphics {
    fn render(&self, alpha: f64) -> Result<()> {
        self.begin_draw();
        self.clear_screen(0., 0., 0.);

        for i in 0..PARAMS.y_points {
            let mut previous_point = self.plot.get_pixel_value(i, 0, alpha);

            for j in 1..PARAMS.x_points {
                let next_point = self.plot.get_pixel_value(i, j, alpha);
                self.draw_line(previous_point, next_point);
                previous_point = next_point;
            }
        }

        for j in 0..PARAMS.x_points {
            let mut previous_point = self.plot.get_pixel_value(0, j, alpha);

            for i in 1..PARAMS.y_points {
                let next_point = self.plot.get_pixel_value(i, j, alpha);
                self.draw_line(previous_point, next_point);
                previous_point = next_point;
            }
        }

        self.end_draw()
    }

    fn new(window: &Window) -> Result<Self> {
        let options = D2D1_FACTORY_OPTIONS::default();

        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options))? };

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

        let brush = unsafe {
            render_target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.,
                    g: 1.,
                    b: 1.,
                    a: 1.0,
                },
                None,
            )?
        };

        Ok(Graphics {
            render_target,
            brush,
            plot: Plot::get_initial_plot(PARAMS.function),
        })
    }

    fn clear_screen(&self, r: f32, g: f32, b: f32) {
        unsafe {
            self.render_target
                .Clear(Some(&D2D1_COLOR_F { r, g, b, a: 1.0 }));
        }
    }

    fn draw_line(&self, p1: (f32, f32), p2: (f32, f32)) {
        let point0 = D2D_POINT_2F { x: p1.0, y: p1.1 };
        let point1 = D2D_POINT_2F { x: p2.0, y: p2.1 };

        unsafe {
            self.render_target
                .DrawLine(point0, point1, &self.brush, 1., None);
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
    previous_call_to_update: i64,
    current_call_to_update: i64,
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
            previous_call_to_update: counter,
            current_call_to_update: counter,
            frequency,
        })
    }

    fn get_time(&self) -> f64 {
        let delta = (self.current_call_to_update - self.start_time) as f64;
        delta / (self.frequency as f64)
    }

    fn reset(&mut self) -> Result<()> {
        self.previous_call_to_update = self.current_call_to_update;
        self.current_call_to_update = Self::query_performance_counter()?;
        Ok(())
    }
}

struct Window {
    handle: HWND,
    visible: bool,
    timer: Timer,
    client_area_width: i32,
    client_area_height: i32,
    graphics: Option<Graphics>,
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
        })
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    BeginPaint(self.handle, &mut ps);

                    if let Some(graphics) = &self.graphics {
                        graphics.render(self.timer.get_time()).unwrap();
                    }

                    EndPaint(self.handle, &ps);
                    LRESULT(0)
                }

                WM_SIZE | WM_DISPLAYCHANGE => {
                    if let Some(graphics) = &self.graphics {
                        graphics.render(self.timer.get_time()).unwrap();
                    }
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
                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);
            let window_class = s!("AnimationWave");

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
                s!("Animation Wave"),
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
                right: 800,
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
                if let Some(graphics) = &self.graphics {
                    self.timer.reset()?;
                    println!("t: {}", self.timer.get_time());
                    graphics.render(self.timer.get_time())?;
                }

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
