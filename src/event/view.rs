use crate::layout::{LayoutVecType, MyLayout};
use crate::rect::MyRect;
use crate::App;
use crate::Model;
use nannou::prelude::*;
use std::ops::Deref;
use std::rc::Rc;

/**
 * in layout, when
 * resize:              change the percentage
 * add constraints:     change the percentage and recoordinate
 * remove constraint:   change the percentage and recoordinate
 *
 * percentage:      suggest value (?) / boundary (?)
 * recoordinate:    suggest value (?) / boundary (?)
 */
pub fn view(app: &App, model: &Model, frame: Frame) {
    let solver = &model.solver;
    let padding = model.padding;
    let ww = &model.window_width;
    let wh = &model.window_height;

    let window_width = solver.get_value(*ww);
    let window_height = solver.get_value(*wh);

    let draw = app.draw();
    draw.background().color(GRAY);
    let win = app.window_rect();
    let win_p = win.pad(padding);

    let mut queue: Vec<Rc<MyLayout>> = Vec::from([Rc::new(model.layout.to_owned())]);

    let mut vert = 0;
    let mut hori = 0;

    // draw to screen

    // println!("start:");
    model.layout.draw(&draw, solver, win_p, &win_p, padding);
    // println!(":end");

    draw.to_frame(app, &frame).unwrap();
}
