use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, MouseEvent};

struct Particle {
    x: f64,
    y: f64,
    size: f64,
    speed_x: f64,
    speed_y: f64,
}

impl Particle {
    fn new(w: f64, h: f64) -> Self {
        Self {
            x: js_sys::Math::random() * w,
            y: js_sys::Math::random() * h,
            size: js_sys::Math::random() * 1.5 + 0.5,
            speed_x: (js_sys::Math::random() - 0.5) * 0.5,
            speed_y: (js_sys::Math::random() - 0.5) * 0.5,
        }
    }
}

struct State {
    particles: Vec<Particle>,
    mouse_x: f64,
    mouse_y: f64,
    width: f64,
    height: f64,
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let glow: HtmlElement = document
    .get_element_by_id("mouse-glow")
    .unwrap()
    .dyn_into()?;

    let canvas: HtmlCanvasElement = document
    .get_element_by_id("particleCanvas")
    .unwrap()
    .dyn_into()?;

    let ctx: CanvasRenderingContext2d = canvas
    .get_context("2d")?
    .unwrap()
    .dyn_into()?;

    let width = window.inner_width()?.as_f64().unwrap();
    let height = window.inner_height()?.as_f64().unwrap();
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let count = ((width * height) / 15_000.0) as usize;
    let mut particles = Vec::with_capacity(count);
    for _ in 0..count {
        particles.push(Particle::new(width, height));
    }

    let state = Rc::new(RefCell::new(State {
        particles,
        mouse_x: -1000.0,
        mouse_y: -1000.0,
        width,
        height,
    }));

    // Mouse Move Event
    let state_mouse = state.clone();
    let mouse_cb = Closure::wrap(Box::new(move |event: MouseEvent| {
        let x = event.client_x() as f64;
        let y = event.client_y() as f64;
        let mut s = state_mouse.borrow_mut();
        s.mouse_x = x;
        s.mouse_y = y;
        glow.style().set_property("--mouse-x", &format!("{}px", x)).unwrap();
        glow.style().set_property("--mouse-y", &format!("{}px", y)).unwrap();
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback("mousemove", mouse_cb.as_ref().unchecked_ref())?;
    mouse_cb.forget();

    // Resize Event
    let state_resize = state.clone();
    let canvas_clone = canvas.clone();
    let window_clone = window.clone();
    let resize_cb = Closure::wrap(Box::new(move || {
        let w = window_clone.inner_width().unwrap().as_f64().unwrap();
        let h = window_clone.inner_height().unwrap().as_f64().unwrap();
        canvas_clone.set_width(w as u32);
        canvas_clone.set_height(h as u32);
        let mut s = state_resize.borrow_mut();
        s.width = w;
        s.height = h;
    }) as Box<dyn FnMut()>);
    window.add_event_listener_with_callback("resize", resize_cb.as_ref().unchecked_ref())?;
    resize_cb.forget();

    // Animation Loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let state_anim = state.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut s = state_anim.borrow_mut();
        ctx.clear_rect(0.0, 0.0, s.width, s.height);

        // Extract values to prevent borrow checker conflicts
        let width = s.width;
        let height = s.height;
        let mouse_x = s.mouse_x;
        let mouse_y = s.mouse_y;

        // Update and draw particles
        for p in &mut s.particles {
            p.x += p.speed_x;
            p.y += p.speed_y;

            if p.x > width { p.x = 0.0; }
            if p.x < 0.0 { p.x = width; }
            if p.y > height { p.y = 0.0; }
            if p.y < 0.0 { p.y = height; }

            let dx = p.x - mouse_x;
            let dy = p.y - mouse_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 150.0 {
                let force = (150.0 - dist) / 150.0;
                p.x += (dx / dist) * force * 3.0;
                p.y += (dy / dist) * force * 3.0;
            }

            ctx.begin_path();
            ctx.arc(p.x, p.y, p.size, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.set_fill_style_str("rgba(5, 163, 47, 0.6)");
            ctx.fill();
        }

        // Connect particles
        for a in 0..s.particles.len() {
            for b in a..s.particles.len() {
                let pa = &s.particles[a];
                let pb = &s.particles[b];
                let dx = pa.x - pb.x;
                let dy = pa.y - pb.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < 150.0 {
                    let opacity = 1.0 - (dist / 150.0);
                    ctx.set_stroke_style_str(&format!("rgba(5, 163, 47, {})", opacity * 0.15));
                    ctx.set_line_width(1.0);
                    ctx.begin_path();
                    ctx.move_to(pa.x, pa.y);
                    ctx.line_to(pb.x, pb.y);
                    ctx.stroke();
                }
            }
        }

        web_sys::window()
        .unwrap()
        .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window()
    .unwrap()
    .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
    .unwrap();

    Ok(())
}
