use macroquad::prelude::*;

const CONTROLPOINT_RADIUS: f32 = 10.0;
#[derive(Clone, Copy)]
struct Point {
    pub pos: Vec2,
    pub color: Color
}

impl Point {
    pub fn new(pos: Vec2, color: Color) -> Self { Self { pos, color } }

    pub fn draw_control(&self) {
        draw_circle(self.pos.x, self.pos.y, CONTROLPOINT_RADIUS, self.color);
    }

    pub fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, 1.0, 1.0, self.color);
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Point {
        let pos = self.pos.lerp(other.pos, t);
        let color = Color::from_vec(self.color.to_vec().lerp(other.color.to_vec(), t));

        Self { pos , color }
    }
}

// Calculate B(t) using De Casteljau's algorithm
fn decasteljau(points: &[Point], t: f32) -> Point {
    let a = points[0];
    let b = points[1];
    let c = points[2];
    let d = points[3];

    let ab = a.lerp(&b, t);
    let bc = b.lerp(&c, t);
    let cd = c.lerp(&d, t);

    let abc = ab.lerp(&bc, t);
    let bcd = bc.lerp(&cd, t);

    abc.lerp(&bcd, t)
}

// Calculate B(t) using Bernstein's Polynomial form
fn cubic_bezier(t: f32, points: &[Point]) -> Vec2 {
    (points[0].pos * (-t.powi(3) + 3.*t.powi(2) - 3. * t + 1.))  +
    (points[1].pos * (3. * t.powi(3) - 6. * t.powi(2) + 3. * t)) +
    (points[2].pos * (-3. * t.powi(3) + 3. * t.powi(2)))         +
    (points[3].pos * t.powi(3))
}

// B'(t) - First derivative of Bernstein Polynomial - used to get tangent and normal
fn velocity(points: [Point; 4], t: f32) -> Vec2 {
    (points[0].pos * (-3.*t.powi(2) + 6. * t - 3.))   +
    (points[1].pos * (9. * t.powi(2) - 12. * t + 3.)) +
    (points[2].pos * (-9. * t.powi(2) + 6. * t))      +
    (points[3].pos * 3. * t.powi(2))
}

// B''(t) - Second derivative of Bernstein Polynomial - used on curvature formula
fn acceleration(points: [Point; 4], t: f32) -> Vec2 {
    points[0].pos * (-6. * t + 6.)  +
    points[1].pos * (18. * t - 12.) +
    points[2].pos * (-18. * t + 6.) +
    points[3].pos * (6. * t)
}

// Used by tight bounding box to solve for each axis' derivative
fn solve_quadratic(xs: [f32; 4]) -> Option<(f32, f32)> {
    let [x0, x1, x2, x3] = xs;

    let a = (-3. * x0) + (9. * x1) - (9. * x2) + (3. * x3);
    let b = (6. * x0) - (12. * x1) + (6. * x2);
    let c =( -3. * x0) + 3. * x1;
    let delta = b*b - (4.*a*c);

    if delta >= 0. {
        let x0 = (-b + delta.sqrt()) / (2. * a);
        let x1 = (-b - delta.sqrt()) / (2. * a);

        return Some((x0, x1));
    }

    None
}

const BOUNDING_BOX_COLOR: Color = BLUE;
struct BoundingBox {
    point_min: Vec2,
    point_max: Vec2,
    point_color: Color,
    outline_color: Color
}

impl BoundingBox {
    fn draw(&self) {
        let (pmin, pmax) = (self.point_min, self.point_max);

        draw_circle(pmin.x, pmin.y, 5.0, self.point_color);
        draw_circle(pmax.x, pmax.y, 5.0, self.point_color);


        draw_line(pmin.x, pmin.y, pmin.x, pmax.y, 1., self.outline_color);
        draw_line(pmax.x, pmin.y, pmax.x, pmax.y, 1., self.outline_color);

        draw_line(pmin.x, pmin.y, pmax.x, pmin.y, 1., self.outline_color);
        draw_line(pmin.x, pmax.y, pmax.x, pmax.y, 1., self.outline_color);
    }
}

#[derive(Default)]
struct Curve {
    control: Vec<Point>,
    rendered: Vec<Point>,
    boxes: Vec<BoundingBox>,
    modified: bool
}

impl Curve {
    fn bounding_box(points: &[Point]) -> [Vec2; 2] {
        let (mut min_x, mut min_y) = (f32::MAX, f32::MAX);
        let (mut max_x, mut max_y) = (f32::MIN, f32::MIN);

        for p in points {
            min_x = min_x.min(p.pos.x);
            max_x = max_x.max(p.pos.x);

            min_y = min_y.min(p.pos.y);
            max_y = max_y.max(p.pos.y);
        }

        [vec2(min_x, min_y), vec2(max_x, max_y)]
    }

    /// Take the derivative on each eaxis then build by comparing with start_anchor and end_anchor points
    fn tight_bounding_box(points: &[Point]) -> [Vec2; 2] {
        let [mut pmin, mut pmax] = Curve::bounding_box(&[points[0], points[3]]);

        let xs = [points[0].pos.x, points[1].pos.x, points[2].pos.x, points[3].pos.x];
        if let Some((tx0, tx1)) = solve_quadratic(xs) {
            if (0.0..1.0).contains(&tx0) {
                let candidate = cubic_bezier(tx0, points);
                draw_circle(candidate.x, candidate.y, 5.0, RED);
                draw_text("x0", candidate.x, candidate.y, 20.0, YELLOW);

                // info!("x0 candidate: {}", candidate);
                let candidate_x = candidate.x;
                pmin.x = pmin.x.min(candidate_x);
                pmax.x = pmax.x.max(candidate_x);
            }

            if (0.0..1.0).contains(&tx1) {
                let candidate = cubic_bezier(tx1, points);
                draw_circle(candidate.x, candidate.y, 5.0, RED);
                draw_text("x1", candidate.x, candidate.y, 20.0, YELLOW);

                // info!("x1 candidate: {}", candidate);
                let candidate_x = candidate.x;
                pmin.x = pmin.x.min(candidate_x);
                pmax.x = pmax.x.max(candidate_x);
            }
        }

        let ys = [points[0].pos.y, points[1].pos.y, points[2].pos.y, points[3].pos.y];
        if let Some((ty0, ty1)) = solve_quadratic(ys) {
            if (0.0..1.0).contains(&ty0) {
                let candidate = cubic_bezier(ty0, points);
                draw_circle(candidate.x, candidate.y, 5.0, RED);
                draw_text("y0", candidate.x, candidate.y, 20.0, YELLOW);

                // info!("y0 candidate: {}", candidate);
                let candidate_y = candidate.y;
                pmin.y = pmin.y.min(candidate_y);
                pmax.y = pmax.y.max(candidate_y);
            }

            if (0.0..1.0).contains(&ty1) {
                let candidate = cubic_bezier(ty1, points);
                draw_circle(candidate.x, candidate.y, 5.0, RED);
                draw_text("y1", candidate.x, candidate.y, 20.0, YELLOW);

                // info!("y1 candidate: {}", candidate);
                let candidate_y = candidate.y;
                pmin.y = pmin.y.min(candidate_y);
                pmax.y = pmax.y.max(candidate_y);
            }
        }

        [pmin, pmax]
    }

    fn render(&mut self, use_casteljau: bool) {
        info!("Rendering new curve!");
        let bezier = match use_casteljau {
            true => decasteljau,
            false => |points: &[Point], t| {
                let (start, end) = (points[0], points[3]);
                let color = Color::from_vec(start.color.to_vec().lerp(end.color.to_vec(), t));
                Point::new(cubic_bezier(t, points), color)
            },
        };

        let control = &mut self.control;

        for control_window in control.windows(4).step_by(3) {
            let a = &control_window[0];
            let b = &control_window[1];

            let c = &control_window[2];
            let d = &control_window[3];

            for t in (0..=2000).map(|t| t as f32*0.0005) {

                // self.rendered.push(bp);
                let new_point = bezier(control_window, t);
                self.rendered.push(new_point);


                // Uncomment to draw normals and curvature
                // if t % 0.5 < 0.000001 {
                //     let vel = Curve::velocity(control_window, t);
                //     let acc = Curve::acceleration(control_window, t);

                //     // Tangent, normal
                //     let t = vel.normalize();
                //     let n = t.perp();

                //     let pn = bp.pos - n*50.;
                //     let pp = bp.pos + n*50.;
                //     draw_line(pn.x, pn.y, pp.x, pp.y, 2.0, VIOLET);

                //     // Curvature
                //     // let k = Mat2::from_cols(vel, acc).determinant() / vel.length().powi(3);

                //     // let r = 1.0/k;
                //     // let center = bp.pos + (n*(r+10.));
                //     // draw_circle_lines(center.x, center.y, r, 2.0, PURPLE);
                // }
            }

            let [point_min, point_max] = Curve::bounding_box(control_window);
            self.boxes.push(BoundingBox {
                point_min,
                point_max,
                point_color: BOUNDING_BOX_COLOR,
                outline_color: BOUNDING_BOX_COLOR
            });

            let [point_min, point_max] = Curve::tight_bounding_box(control_window);
            self.boxes.push(BoundingBox {
                point_min,
                point_max,
                point_color: RED,
                outline_color: GOLD
            });

            draw_text("a", a.pos.x, a.pos.y, 42.0, YELLOW);
            draw_text("b", b.pos.x, b.pos.y, 42.0, YELLOW);
            draw_text("c", c.pos.x, c.pos.y, 42.0, YELLOW);
            draw_text("d", d.pos.x, d.pos.y, 42.0, YELLOW);
        }

        self.modified = false;
    }

    fn draw(&mut self, draw_bounding: bool, use_casteljau: bool) {
        if self.control.len() < 4 { return };
        if self.modified {
            self.rendered.clear();
            self.boxes.clear();
            self.render(use_casteljau);
        }

        for point in &self.rendered {
            point.draw();
        }

        if draw_bounding {
            for bbox in &self.boxes {
                bbox.draw();
            }
        }
    }

    fn draw_controls(&mut self) {
        for control in &self.control {
            control.draw_control();
        }

        for controls in self.control.windows(4).step_by(3) {
            let (anchor, control) = (controls[0], controls[1]);
            let color = Color::from_vec(anchor.color.to_vec().lerp(control.color.to_vec(), 0.5));
            draw_line(anchor.pos.x, anchor.pos.y, control.pos.x, control.pos.y, 1.0, color);

            let (anchor, control) = (controls[3], controls[2]);
            let color = Color::from_vec(anchor.color.to_vec().lerp(control.color.to_vec(), 0.5));
            draw_line(anchor.pos.x, anchor.pos.y, control.pos.x, control.pos.y, 1.0, color);
        }
    }
}

/// Main Function - here we treat the inputs, the curve
///  creation and call the drawing methods each frame
#[macroquad::main("Trabalho Numéricos")]
async fn main() {
    let mut color_it = [ORANGE, BLUE, RED, PURPLE].into_iter().cycle();
    let mut curve = Curve::default();
    curve.modified = true;

    let mut selected: Option<usize> = None;
    let mut draw_bounding = false;
    let mut draw_grid = false;
    let mut use_casteljau = false;
    loop {
        clear_background(BLACK);

        let (mx, my) = mouse_position();

        // Collision - if we're dragging a point, move it. Otherwise, try colliding with every point
        if let Some(id) = selected {
            curve.control[id].pos = vec2(mx, my);
            curve.modified = true;
        } else {
            for (i, p) in curve.control.iter().enumerate() {
                let dist = ((mx - p.pos.x).powi(2) + (my - p.pos.y).powi(2)).sqrt();
                if dist <= CONTROLPOINT_RADIUS {
                    selected = Some(i);
                }
            }
        }

        // Delete point on right click
        if let Some(id) = selected {
            if is_mouse_button_pressed(MouseButton::Right) {
                curve.control.remove(id);
                curve.modified = true;
            }
        }

        // Add point on left click
        if selected.is_none() {
            if is_mouse_button_pressed(MouseButton::Left) {
                let new_point = Point::new(vec2(mx, my), color_it.next().unwrap());
                curve.control.push(new_point);
                curve.modified = true;
            }
        }

        // Un-selected the previously draggable point
        if !is_mouse_button_down(MouseButton::Left) {
            selected = None;
        }

        if is_key_pressed(KeyCode::B) {
            draw_bounding = !draw_bounding;
        }

        if is_key_pressed(KeyCode::G) {
            draw_grid = !draw_grid;
        }

        if is_key_pressed(KeyCode::M) {
            use_casteljau = !use_casteljau;
            info!("Mode toggled! Casteljau: {}", use_casteljau);
        }

        // Everything is rendered here - the order matters!
        if draw_grid { draw_grid2d() };
        curve.draw_controls();
        curve.draw(draw_bounding, use_casteljau);
        next_frame().await;
    }
}

/// Draw a grid centered at (0, 0, 0)
pub fn draw_grid2d() {
    let wmid = screen_width()/2.0;
    let hmid = screen_height()/2.0;

    let x_step = (screen_width()/16.0) as usize;
    let y_step = (screen_height()/9.0) as usize;

    for i in (0..=wmid as i32).rev().step_by(x_step)  {
        draw_line(
        i as f32, 0.,
            i as f32, screen_height(),
            1.0,
            GREEN,
        );
    }

    for i in (wmid as i32..screen_width() as i32).step_by(x_step)  {
        draw_line(
        i as f32, 0.,
            i as f32, screen_height(),
            1.0,
            GREEN,
        );
    }

    for i in (0..=hmid as i32).rev().step_by(y_step)  {
        draw_line(
            0., i as f32,
            screen_width(), i as f32,
                1.0,
                GREEN,
            );
    }

    for i in (hmid as i32..screen_width() as i32).step_by(y_step)  {
        draw_line(
            0., i as f32,
            screen_width(), i as f32,
                1.0,
                GREEN,
        );
    }

    draw_circle(wmid, hmid, 5.0, YELLOW);
    draw_line(0.0, hmid, screen_width(), hmid, 1.0, YELLOW);
    draw_line(wmid, 0.0, wmid, screen_height(), 1.0, YELLOW);
}