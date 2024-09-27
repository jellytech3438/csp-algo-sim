use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::Constraint;
use cassowary::Solver;
use cassowary::Variable;
use cassowary::{Expression, RelationalOperator};
use nannou::prelude::*;
use nannou_egui::egui::math::Numeric;
use nannou_egui::egui::Key;
use nannou_egui::{self, egui, Egui};
use std::ops::Index;
use std::{cell::RefCell, rc::Rc};

mod event;
mod layout;
mod rect;

use event::{key_pressed, resize, view};
use layout::*;
use rect::MyRect;

fn main() {
    nannou::app(model).update(update).run();
}

pub struct Model {
    window_width: Variable,
    window_height: Variable,
    solver: Solver,
    layout: MyLayout,
    padding: f32,
    minbox: f32,
    step: f32,
}

impl Model {
    fn insertion(&mut self, selection: bool, insertway: InsertionWay, init: bool) {
        let mut node = Node::new(selection, self.minbox);
        self.add_rect_edit_variable(&mut node.rect);
        self.add_rect_contraints(&mut node.rect);

        // insert into layout
        // if let Some(c) =
        //     self.layout
        //         .insert_with_constraint(&node, insertway, self.padding, &mut self.solver)
        // {
        //     println!("{:?}", c);
        //     self.solver.add_constraint(c).unwrap();
        // }

        self.layout
            .insert_with_constraint(&node, insertway, self.padding, &mut self.solver);
        // match self
        //     .layout
        //     .insert_with_constraint(&node, insertway, self.padding, &mut self.solver)
        // {
        //     Some(ref c) => {
        //         self.solver.add_constraints(c).unwrap();
        //     }
        //     None => {}
        // }

        // layout constraints
        self.rm_layout_constraint();
        self.add_layout_constr();
        self.print_variable();
    }

    pub fn get_rect_variable(&self, rect: &MyRect) -> (f64, f64, f64, f64) {
        let top = self.solver.get_value(rect.top);
        let bottom = self.solver.get_value(rect.bottom);
        let left = self.solver.get_value(rect.left);
        let right = self.solver.get_value(rect.right);
        (top, bottom, left, right)
    }

    pub fn add_rect_edit_variable(&mut self, rect: &mut MyRect) {
        self.solver.add_edit_variable(rect.top, WEAK).unwrap();
        self.solver.add_edit_variable(rect.bottom, WEAK).unwrap();
        self.solver.add_edit_variable(rect.left, WEAK).unwrap();
        self.solver.add_edit_variable(rect.right, WEAK).unwrap();
    }

    pub fn rm_edit_variable(&mut self, rect: &mut MyRect) {
        self.solver.remove_edit_variable(rect.top).unwrap();
        self.solver.remove_edit_variable(rect.bottom).unwrap();
        self.solver.remove_edit_variable(rect.left).unwrap();
        self.solver.remove_edit_variable(rect.right).unwrap();
    }

    // basic constraints:
    //     any border length >= 0
    //     width & height >= MINBOX
    pub fn add_rect_contraints(&mut self, rect: &mut MyRect) {
        let mut constraints = Vec::new();

        // top |GE(REQUIRED)| 0.0
        let mut rTop = Constraint::new(
            Expression::from(rect.top),
            RelationalOperator::GreaterOrEqual,
            REQUIRED,
        );
        // height |GE(REQUIRED)| 0.0
        let mut rHeight = Constraint::new(
            Expression::from(rect.height() - self.minbox),
            RelationalOperator::GreaterOrEqual,
            REQUIRED,
        );
        // width |GE(REQUIRED)| 0.0
        let mut rWidth = Constraint::new(
            Expression::from(rect.width() - self.minbox),
            RelationalOperator::GreaterOrEqual,
            REQUIRED,
        );
        // left |GE(REQUIRED)| 0.0
        let mut rLeft = Constraint::new(
            Expression::from(rect.left),
            RelationalOperator::GreaterOrEqual,
            REQUIRED,
        );

        constraints.push(rTop);
        constraints.push(rHeight);
        constraints.push(rWidth);
        constraints.push(rLeft);

        self.solver.add_constraints(&constraints).unwrap();

        rect.init_constr(&constraints);
    }

    pub fn rm_rect_constraints(&mut self, rect: &mut MyRect) {
        match rect.border_constr {
            Some(ref c) => {
                for i in 0..4 {
                    self.solver.remove_constraint(&c[i]);
                }
            }
            _ => {}
        }
    }

    pub fn add_layout_constr(&mut self) {
        if self.layout.nodes.len() == 0 {
            self.layout.insert_with_constraint(
                &Node::new(true, self.minbox),
                InsertionWay::VERT,
                self.padding,
                &mut self.solver,
            );
        }

        let mut constraints = Vec::new();

        let mut vert_expression = match self.layout.direc {
            InsertionWay::VERT => self.layout.first_layout_width(self.padding),
            InsertionWay::HORI => self.layout.any_layout_width(self.padding),
        };
        let mut hori_expression = match self.layout.direc {
            InsertionWay::VERT => self.layout.any_layout_height(self.padding),
            InsertionWay::HORI => self.layout.first_layout_height(self.padding),
        };

        hori_expression = hori_expression + 2.0 * self.padding.to_f64();
        vert_expression = vert_expression + 2.0 * self.padding.to_f64();

        // match self.layout.direc {
        //     InsertionWay::HORI => {
        //         vert_expression = vert_expression + 2.0 * self.padding.to_f64();
        //     }
        //     InsertionWay::VERT => {
        //         hori_expression = hori_expression + 2.0 * self.padding.to_f64();
        //         vert_expression = vert_expression + 2.0 * self.padding.to_f64();
        //     }
        // }

        let mut vert_constraint = Constraint::new(
            vert_expression - self.window_width,
            RelationalOperator::Equal,
            REQUIRED,
        );
        let mut hori_constraint = Constraint::new(
            hori_expression - self.window_height,
            RelationalOperator::Equal,
            REQUIRED,
        );

        constraints.push(&vert_constraint);
        constraints.push(&hori_constraint);

        self.solver.add_constraints(constraints).unwrap();

        self.layout.vert_constraint = Some(vert_constraint);
        self.layout.horz_constraint = Some(hori_constraint);
    }

    pub fn rm_layout_constraint(&mut self) {
        match self.layout.vert_constraint {
            Some(ref c) => {
                self.solver.remove_constraint(c).unwrap();
            }
            _ => {}
        }

        match self.layout.horz_constraint {
            Some(ref c) => {
                self.solver.remove_constraint(c).unwrap();
            }
            _ => {}
        }
    }

    pub fn select(&mut self, key: Key) {
        self.layout.select(key, false);
    }

    pub fn print_variable(&mut self) {
        self.layout.print(&self.solver);
    }
}

// model init
fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("cassowary window managment")
        .view(view)
        .resized(resize)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    // actual values
    let window_rect_x = window.rect().x.len();
    let window_rect_y = window.rect().y.len();

    // constraints
    let window_width = Variable::new();
    let window_height = Variable::new();

    // solver
    let mut solver = Solver::new();

    // first init node
    // let mut node = Node::new(true);

    let mut model = Model {
        window_width: window_width,
        window_height: window_height,
        solver: solver,
        layout: MyLayout::new(InsertionWay::VERT),
        minbox: 70.0,
        padding: 10.0,
        step: 50.0,
    };

    model.insertion(true, InsertionWay::VERT, true);
    model.layout.select_first();
    model
        .solver
        .add_edit_variable(model.window_width, STRONG)
        .unwrap();
    model
        .solver
        .add_edit_variable(model.window_height, STRONG)
        .unwrap();
    model
        .solver
        .suggest_value(model.window_width, window_rect_x.to_f64())
        .unwrap();
    model
        .solver
        .suggest_value(model.window_height, window_rect_y.to_f64())
        .unwrap();

    model
}

fn update(app: &App, model: &mut Model, update: Update) {
    let Model { ref mut solver, .. } = *model;

    // solver fetching will only print once
    for &(var, value) in solver.fetch_changes() {
        // println!("{:?} {}", var, value);
    }
}

#[cfg(test)]
mod test {
    use std::collections::{hash_map::DefaultHasher, HashMap};

    use crate::*;
    use cassowary::Term;

    #[test]
    fn test_solver() {
        let mut solver = Solver::new();
        let mut rect = MyRect::new(2.0);
        let mut minbox = 20.0;

        // for minbox
        let mut min_width = Constraint::new(
            rect.width() - minbox,
            RelationalOperator::GreaterOrEqual,
            WEAK,
        );
        let mut min_height = Constraint::new(
            rect.height() - minbox,
            RelationalOperator::GreaterOrEqual,
            WEAK,
        );

        // for window
        let mut win_width = Variable::new();
        let mut win_height = Variable::new();
        let mut wid_constr = Constraint::new(
            rect.width() - win_width,
            RelationalOperator::Equal,
            REQUIRED,
        );
        let mut hei_constr = Constraint::new(
            rect.height() - win_height,
            RelationalOperator::Equal,
            REQUIRED,
        );

        // constraint
        solver.add_constraint(wid_constr.to_owned()).unwrap();
        solver.add_constraint(hei_constr.to_owned()).unwrap();
        solver.add_constraint(min_height.to_owned()).unwrap();
        solver.add_constraint(min_width.to_owned()).unwrap();
        // variable
        solver.add_edit_variable(win_width, STRONG).unwrap();
        solver.add_edit_variable(win_height, STRONG).unwrap();
        solver.add_edit_variable(rect.top, WEAK).unwrap();
        solver.add_edit_variable(rect.left, WEAK).unwrap();
        solver.add_edit_variable(rect.right, WEAK).unwrap();
        solver.add_edit_variable(rect.bottom, WEAK).unwrap();
        // suggest window value
        solver.suggest_value(win_width, 100.0).unwrap();
        solver.suggest_value(win_height, 100.0).unwrap();

        for &(var, value) in solver.fetch_changes() {
            println!("{:?} {}", var, value);
        }

        let mut rect2 = MyRect::new(2.0);

        // for minbox
        let mut min_width_2 = Constraint::new(
            rect2.width() - minbox,
            RelationalOperator::GreaterOrEqual,
            WEAK,
        );
        let mut min_height_2 = Constraint::new(
            rect2.height() - minbox,
            RelationalOperator::GreaterOrEqual,
            WEAK,
        );
        let mut hei_constr_2 = Constraint::new(
            rect2.height() - win_height,
            RelationalOperator::Equal,
            REQUIRED,
        );

        solver.add_constraint(min_width_2.to_owned()).unwrap();
        solver.add_constraint(min_height_2.to_owned()).unwrap();
        solver.add_constraint(hei_constr_2.to_owned()).unwrap();

        // variable
        solver.add_edit_variable(rect2.top, WEAK).unwrap();
        solver.add_edit_variable(rect2.left, WEAK).unwrap();
        solver.add_edit_variable(rect2.right, WEAK).unwrap();
        solver.add_edit_variable(rect2.bottom, WEAK).unwrap();

        // remove old
        solver.remove_constraint(&wid_constr).unwrap();

        // constraint
        let mut new_wid_const = Constraint::new(
            rect.width() + rect2.width() - win_width,
            RelationalOperator::Equal,
            REQUIRED,
        );

        // add new
        solver.add_constraint(new_wid_const.to_owned()).unwrap();

        // test
        println!("has old = {}", solver.has_constraint(&wid_constr));

        // let temp = solver.get_value(a);
        // solver.suggest_value(a, temp + 1.);

        for &(var, value) in solver.fetch_changes() {
            println!("{:?} {}", var, value);
        }

        println!(
            "{} {} {} {}",
            solver.get_value(rect.top),
            solver.get_value(rect.left),
            solver.get_value(rect.right),
            solver.get_value(rect.bottom)
        );
        println!(
            "{} {} {} {}",
            solver.get_value(rect2.top),
            solver.get_value(rect2.left),
            solver.get_value(rect2.right),
            solver.get_value(rect2.bottom)
        );
    }

    #[test]
    fn test_rect() {
        // let mut win_rect = Rect::from_xy_wh(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        // println!("{:?}", win_rect.l_r_b_t());
        // println!("{:?}", win_rect.relative_to_x(100.0));
        // println!("{:?}", win_rect.shift_x(100.0));
        // println!("{:?}", win_rect.right());
        // println!("{:?}", win_rect.mid_right());
        let mut solver = Solver::new();

        let mut rect = MyRect::new(2.0);
        let mut temp_var = Variable::new();
        let mut fixed_var = Variable::new();

        let mut test_constr = Constraint::new(
            rect.width() + temp_var - fixed_var,
            RelationalOperator::Equal,
            REQUIRED,
        );

        solver.add_edit_variable(temp_var, WEAK);
        solver.add_edit_variable(fixed_var, STRONG);

        solver.add_edit_variable(rect.top, WEAK).unwrap();
        solver.add_edit_variable(rect.left, WEAK).unwrap();
        solver.add_edit_variable(rect.right, WEAK).unwrap();
        solver.add_edit_variable(rect.bottom, WEAK).unwrap();
        solver.add_constraint(test_constr.to_owned()).unwrap();

        solver.suggest_value(fixed_var, 10.0);

        for &(var, value) in solver.fetch_changes() {
            println!("{:?} {}", var, value);
        }

        println!(
            "{} {} {}",
            solver.get_value(temp_var),
            solver.get_value(rect.left),
            solver.get_value(rect.right)
        );

        solver.remove_edit_variable(temp_var).unwrap();
        solver.remove_constraint(&test_constr).unwrap();

        let mut new_constr = test_constr.clone();
        let mut map = HashMap::new();
        map.insert(0, test_constr);
        println!("{:?}", map.get(&0));

        for &(var, value) in solver.fetch_changes() {
            println!("{:?} {}", var, value);
        }

        println!(
            "{} {} {}",
            solver.get_value(temp_var),
            solver.get_value(rect.left),
            solver.get_value(rect.right)
        );
    }
}
