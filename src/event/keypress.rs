use std::borrow::BorrowMut;

use crate::App;
use crate::InsertionWay;
use crate::Model;
use nannou::prelude::*;

pub fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::V => {
            model
                .borrow_mut()
                .insertion(false, InsertionWay::VERT, false);

            println!("vertical split");
        }
        Key::H => {
            model
                .borrow_mut()
                .insertion(false, InsertionWay::HORI, false);

            println!("horizental split");
        }
        Key::D => {
            if model.layout.nodes.len() == 0 || model.layout.nodes.len() == 1 {
                return;
            }

            model.layout.remove(model.minbox, &mut model.solver);
        }
        Key::Up => {
            model.borrow_mut().select(nannou_egui::egui::Key::ArrowUp);
            println!("select right");
        }
        Key::Down => {
            model.borrow_mut().select(nannou_egui::egui::Key::ArrowDown);
            println!("select right");
        }
        Key::Left => {
            model.borrow_mut().select(nannou_egui::egui::Key::ArrowLeft);
            println!("select right");
        }
        Key::Right => {
            model
                .borrow_mut()
                .select(nannou_egui::egui::Key::ArrowRight);
            println!("select right");
        }
        _ => {}
    }
}
