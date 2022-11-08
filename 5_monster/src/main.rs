use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::Foundation::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
    Win32::Graphics::Gdi::*,
    Win32::System::Com::*,
    Win32::System::Performance::*,
    Win32::UI::HiDpi::*,
    Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState,
    Win32::UI::WindowsAndMessaging::*,
    Win32::{
        System::LibraryLoader::*,
        UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_LBUTTON},
    },
};

const EYE_RADIUS: i32 = 90;
const BALL_RADIUS: i32 = 30;

struct Graphics {
    render_target: ID2D1HwndRenderTarget,
    brush: ID2D1SolidColorBrush,
    outline_path: ID2D1PathGeometry,
    nose_path: ID2D1PathGeometry,
    nosmile_path: ID2D1PathGeometry,
    smile_path: ID2D1PathGeometry,
    nosebrush: ID2D1SolidColorBrush,
    outline_gradient: ID2D1RadialGradientBrush,
    left_eye_gradient: ID2D1RadialGradientBrush,
    right_eye_gradient: ID2D1RadialGradientBrush,
    mouse_pos: (f32, f32),
}

impl Graphics {
    fn draw_monster_nosmile(factory: &ID2D1Factory) -> Result<ID2D1PathGeometry> {
        let geometry = unsafe { factory.CreatePathGeometry()? };
        let sink = unsafe { geometry.Open()? };

        unsafe {
            sink.BeginFigure(
                D2D_POINT_2F {
                    x: 616.00 - 716.00,
                    y: 427.50 - 294.00,
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F {
                    x: 816.00 - 716.00,
                    y: 427.50 - 294.00,
                },
                rotationAngle: 0.,
                size: D2D_SIZE_F {
                    width: 10.,
                    height: 2.,
                },
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);

            sink.Close()?;
        }

        Ok(geometry)
    }

    fn draw_monster_smile(factory: &ID2D1Factory) -> Result<ID2D1PathGeometry> {
        let geometry = unsafe { factory.CreatePathGeometry()? };
        let sink = unsafe { geometry.Open()? };

        unsafe {
            sink.BeginFigure(
                D2D_POINT_2F {
                    x: 616.00 - 716.00,
                    y: 427.50 - 294.00,
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F {
                    x: 816.00 - 716.00,
                    y: 427.50 - 294.00,
                },
                rotationAngle: 0.,
                size: D2D_SIZE_F {
                    width: 10.,
                    height: 7.,
                },
                sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);

            sink.Close()?;
        }

        Ok(geometry)
    }

    fn draw_monster_nose(factory: &ID2D1Factory) -> Result<ID2D1PathGeometry> {
        let geometry = unsafe { factory.CreatePathGeometry()? };
        let sink = unsafe { geometry.Open()? };

        unsafe {
            sink.BeginFigure(
                D2D_POINT_2F {
                    x: 649.95 - 716.00,
                    y: 333.04 - 294.00,
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 649.95 - 716.00,
                    y: 333.04 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 679.40 - 716.00,
                    y: 382.37 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 713.28 - 716.00,
                    y: 383.71 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 752.93 - 716.00,
                    y: 381.21 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 774.51 - 716.00,
                    y: 337.71 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 778.78 - 716.00,
                    y: 329.96 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 778.78 - 716.00,
                    y: 329.96 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 789.32 - 716.00,
                    y: 295.96 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 715.79 - 716.00,
                    y: 291.21 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 635.73 - 716.00,
                    y: 293.71 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 649.95 - 716.00,
                    y: 333.04 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 649.95 - 716.00,
                    y: 333.04 - 294.00,
                },
            });
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);

            sink.Close()?;
        }

        Ok(geometry)
    }

    fn get_eye(x: f32, y: f32) -> D2D1_ELLIPSE {
        D2D1_ELLIPSE {
            point: D2D_POINT_2F { x, y },
            radiusX: EYE_RADIUS as f32,
            radiusY: EYE_RADIUS as f32,
        }
    }

    fn get_ball(&self, eye: (f32, f32), translation: (f32, f32)) -> D2D1_ELLIPSE {
        let (curbias_x, curbias_y) = self.mouse_pos;
        let (translation_x, translation_y) = translation;
        let (cursor_x, cursor_y) = (curbias_x - translation_x, curbias_y - translation_y);
        let (eye_x, eye_y) = eye;
        let (currel_x, currel_y) = (cursor_x - eye_x, cursor_y - eye_y);

        const BALL_ORBIT: i32 = EYE_RADIUS - BALL_RADIUS;
        const BALL_ORBIT_SQUARE: i32 = BALL_ORBIT * BALL_ORBIT;

        let cursor_formula = currel_x * currel_x + currel_y * currel_y;

        let (ball_x, ball_y) = {
            if cursor_formula <= BALL_ORBIT_SQUARE as f32 {
                (cursor_x, cursor_y)
            } else {
                let factor = BALL_ORBIT as f32 / cursor_formula.sqrt();
                (currel_x * factor + eye_x, currel_y * factor + eye_y)
            }
        };

        D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: ball_x,
                y: ball_y,
            },
            radiusX: BALL_RADIUS as f32,
            radiusY: BALL_RADIUS as f32,
        }
    }

    fn draw_monster_outline(factory: &ID2D1Factory) -> Result<ID2D1PathGeometry> {
        let geometry = unsafe { factory.CreatePathGeometry()? };
        let sink = unsafe { geometry.Open()? };

        unsafe {
            sink.BeginFigure(
                D2D_POINT_2F {
                    x: 837.50 - 716.00,
                    y: 83.06 - 294.00,
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 837.50 - 716.00,
                    y: 83.06 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 888.50 - 716.00,
                    y: -9. - 294.0001,
                },
                point3: D2D_POINT_2F {
                    x: 965.50 - 716.00,
                    y: 54.54 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 965.50 - 716.00,
                    y: 54.54 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 1035.00 - 716.00,
                    y: 115.09 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 911.50 - 716.00,
                    y: 205.15 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 898.50 - 716.00,
                    y: 231.17 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 1028.50 - 716.00,
                    y: 415.81 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 963.50 - 716.00,
                    y: 513.38 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 929.00 - 716.00,
                    y: 563.92 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 859.50 - 716.00,
                    y: 585.94 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 726.50 - 716.00,
                    y: 586.44 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 610.50 - 716.00,
                    y: 584.43 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 493.50 - 716.00,
                    y: 587.44 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 454.50 - 716.00,
                    y: 491.37 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 422.31 - 716.00,
                    y: 419.50 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 492.00 - 716.00,
                    y: 295.22 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 516.00 - 716.00,
                    y: 205.15 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 379.50 - 716.00,
                    y: 99.07 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 474.52 - 716.00,
                    y: 44.37 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 473.50 - 716.00,
                    y: 45.03 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 545.33 - 716.00,
                    y: 2.17 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 585.22 - 716.00,
                    y: 77.69 - 294.00,
                },
                point3: D2D_POINT_2F {
                    x: 587.50 - 716.00,
                    y: 82.56 - 294.00,
                },
            });
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F {
                    x: 587.50 - 716.00,
                    y: 82.56 - 294.00,
                },
                point2: D2D_POINT_2F {
                    x: 702.17 - 716.00,
                    y: -21. - 294.0035,
                },
                point3: D2D_POINT_2F {
                    x: 837.50 - 716.00,
                    y: 83.06 - 294.00,
                },
            });
            sink.EndFigure(D2D1_FIGURE_END_CLOSED);

            sink.Close()?;
        }

        Ok(geometry)
    }

    fn create_outline_gradient_brush(
        target: &ID2D1HwndRenderTarget,
    ) -> Result<ID2D1RadialGradientBrush> {
        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.,
                color: D2D1_COLOR_F {
                    r: 0.8,
                    g: 1.,
                    b: 0.8,
                    a: 1.,
                },
            },
            D2D1_GRADIENT_STOP {
                position: 0.5,
                color: D2D1_COLOR_F {
                    r: 0.,
                    g: 0.9,
                    b: 0.,
                    a: 1.,
                },
            },
            D2D1_GRADIENT_STOP {
                position: 0.8,
                color: D2D1_COLOR_F {
                    r: 0.1,
                    g: 0.6,
                    b: 0.1,
                    a: 1.,
                },
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: D2D1_COLOR_F {
                    r: 0.3,
                    g: 0.4,
                    b: 0.3,
                    a: 1.,
                },
            },
        ];

        let gradient_collection = unsafe {
            target.CreateGradientStopCollection(
                &gradient_stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?
        };

        let ellipse_center = D2D_POINT_2F { x: 0.00, y: 30.00 };

        unsafe {
            target.CreateRadialGradientBrush(
                &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                    center: ellipse_center,
                    radiusX: 340.,
                    radiusY: 380.,
                    gradientOriginOffset: D2D_POINT_2F { x: 0., y: 0. },
                },
                None,
                &gradient_collection,
            )
        }
    }

    fn create_left_eye_gradient_brush(
        target: &ID2D1HwndRenderTarget,
    ) -> Result<ID2D1RadialGradientBrush> {
        Self::create_eye_gradient_brush(target, 588. - 716.)
    }

    fn create_right_eye_gradient_brush(
        target: &ID2D1HwndRenderTarget,
    ) -> Result<ID2D1RadialGradientBrush> {
        Self::create_eye_gradient_brush(target, 840. - 716.)
    }

    fn create_eye_gradient_brush(
        target: &ID2D1HwndRenderTarget,
        x: f32,
    ) -> Result<ID2D1RadialGradientBrush> {
        let gradient_stops = [
            D2D1_GRADIENT_STOP {
                position: 0.7,
                color: D2D1_COLOR_F {
                    r: 1.,
                    g: 1.,
                    b: 1.,
                    a: 1.,
                },
            },
            D2D1_GRADIENT_STOP {
                position: 1.0,
                color: D2D1_COLOR_F {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                    a: 1.,
                },
            },
        ];

        let gradient_collection = unsafe {
            target.CreateGradientStopCollection(
                &gradient_stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?
        };

        let ellipse_center = D2D_POINT_2F { x, y: 210. - 294. };

        unsafe {
            target.CreateRadialGradientBrush(
                &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                    center: ellipse_center,
                    radiusX: EYE_RADIUS as f32,
                    radiusY: EYE_RADIUS as f32,
                    gradientOriginOffset: D2D_POINT_2F { x: 0., y: 0. },
                },
                None,
                &gradient_collection,
            )
        }
    }

    fn render(&self, alpha: f64, lbutton_up: bool) -> Result<()> {
        self.begin_draw();
        self.clear_screen(0.7, 0.7, 1.);

        let translation = {
            let size = unsafe { self.render_target.GetSize() };
            (size.width / 2., size.height / 2.)
        };

        let translation_matrix = Matrix3x2::translation(translation.0, translation.1);
        let rotation_matrix = Matrix3x2::rotation(alpha as f32, 0., 0.) * translation_matrix;

        let eye_y = 210. - 294.;
        let (eye_l_x, eye_r_x) = (588. - 716., 840. - 716.);

        let left_eye = Self::get_eye(eye_l_x, eye_y);
        let right_eye = Self::get_eye(eye_r_x, eye_y);

        let left_ball = self.get_ball((eye_l_x, eye_y), translation);
        let right_ball = self.get_ball((eye_r_x, eye_y), translation);

        unsafe {
            self.render_target.SetTransform(&translation_matrix);
            self.render_target
                .FillGeometry(&self.outline_path, &self.outline_gradient, None);
            self.render_target
                .DrawGeometry(&self.outline_path, &self.brush, 5., None);

            self.render_target
                .FillEllipse(&left_eye, &self.left_eye_gradient);
            self.render_target
                .FillEllipse(&right_eye, &self.right_eye_gradient);

            self.render_target
                .DrawEllipse(&left_eye, &self.brush, 2., None);
            self.render_target
                .DrawEllipse(&right_eye, &self.brush, 2., None);

            self.render_target.FillEllipse(&left_ball, &self.brush);
            self.render_target.FillEllipse(&right_ball, &self.brush);

            self.render_target.SetTransform(&rotation_matrix);
            self.render_target
                .FillGeometry(&self.nose_path, &self.nosebrush, None);
            self.render_target
                .DrawGeometry(&self.nose_path, &self.brush, 2., None);

            let mouth = match lbutton_up {
                false => &self.nosmile_path,
                true => &self.smile_path,
            };

            self.render_target
                .DrawGeometry(mouth, &self.brush, 7., None);
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
                    r: 0.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                },
                None,
            )?
        };

        let nosebrush = unsafe {
            render_target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.25,
                    g: 0.25,
                    b: 0.25,
                    a: 1.,
                },
                None,
            )?
        };

        let outline_path = Self::draw_monster_outline(&factory)?;
        let nose_path = Self::draw_monster_nose(&factory)?;
        let nosmile_path = Self::draw_monster_nosmile(&factory)?;
        let smile_path = Self::draw_monster_smile(&factory)?;

        let outline_gradient = Self::create_outline_gradient_brush(&render_target)?;
        let left_eye_gradient = Self::create_left_eye_gradient_brush(&render_target)?;
        let right_eye_gradient = Self::create_right_eye_gradient_brush(&render_target)?;

        Ok(Graphics {
            render_target,
            brush,
            nosebrush,
            outline_path,
            nose_path,
            nosmile_path,
            smile_path,
            outline_gradient,
            left_eye_gradient,
            right_eye_gradient,
            mouse_pos: (0., 0.),
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

    fn on_mouse_move(&mut self, pixel_x: f32, pixel_y: f32) {
        self.mouse_pos = (pixel_x, pixel_y);
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

struct Window {
    handle: HWND,
    visible: bool,
    timer: Timer,
    client_area_width: i32,
    client_area_height: i32,
    graphics: Option<Graphics>,
    alpha: f64,
    lbutton_up: bool,
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
            alpha: 0.,
            lbutton_up: false,
        })
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    BeginPaint(self.handle, &mut ps);

                    if let Some(graphics) = &self.graphics {
                        graphics.render(self.alpha, self.lbutton_up).unwrap();
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
                WM_MOUSEMOVE => {
                    let x_pos = ((lparam.0 as u32) & 0xFFFF) as u16 as i32;
                    let y_pos = (((lparam.0 as u32) >> 16) & 0xFFFF) as u16 as i32;

                    let dpi = GetDpiForWindow(self.handle) as i32;

                    let x = (x_pos * 96) as f32 / dpi as f32;
                    let y = (y_pos * 96) as f32 / dpi as f32;

                    if self.graphics.is_some() {
                        self.graphics.as_mut().unwrap().on_mouse_move(x, y);
                    }

                    LRESULT(0)
                }
                WM_SIZE => {
                    let mut rc = RECT::default();

                    if GetClientRect(self.handle, &mut rc).into() && self.graphics.is_some() {
                        let size = D2D_SIZE_U {
                            width: rc.right as u32,
                            height: rc.bottom as u32,
                        };

                        self.graphics.as_mut().unwrap().render_target.Resize(&size).unwrap();

                        LRESULT(0)
                    } else {
                        LRESULT(1)
                    }

                }
                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }

    fn key_up(vkey: VIRTUAL_KEY) -> bool {
        let state = unsafe { GetAsyncKeyState(vkey.0.into()) };
        (state as u16 & 0x8000) != 0
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
                s!("Monster likes when you click the mouse button"),
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
                if let Some(graphics) = &self.graphics {
                    let time = self.timer.get_time(2) * std::f64::consts::PI;
                    self.alpha = time.sin() * 10.;
                    self.lbutton_up = Self::key_up(VK_LBUTTON);

                    graphics.render(self.alpha, self.lbutton_up)?;
                    self.timer.update()?;
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
