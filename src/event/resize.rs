use crate::App;
use crate::Model;
use nannou::prelude::*;

pub fn resize(app: &App, model: &mut Model, _dim: Vec2) {
    let Model {
        ref window_width,
        ref window_height,
        ref mut solver,
        ..
    } = *model;

    // get old variable
    let mut old_width = solver.get_value(window_width.to_owned());
    let mut old_height = solver.get_value(window_height.to_owned());

    // get new area
    let window_rect = app.window_rect();
    let width = window_rect.x.len();
    let height = window_rect.y.len();

    // add constraints to solver
    solver.suggest_value(*window_width, width as f64).unwrap();
    solver.suggest_value(*window_height, height as f64).unwrap();

    // sugest all block's width
    // how to keep the width percentage?
    // how to add by suggestion?
    // for h in 0..position.len(){
    //     solver.suggest_value(position[h][0].left, solver.get_value(position[h][0].left) * (width as f64 / old_width).ceil()).unwrap();
    //     solver.suggest_value(position[h][0].right, solver.get_value(position[h][0].right) * (width as f64 / old_width).ceil()).unwrap();
    // }

    println!("windows: ({}, {})", width, height);
}

