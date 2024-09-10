use crate::rect::MyRect;
use cassowary::strength::{REQUIRED, WEAK};
use cassowary::{strength::STRONG, Constraint};
use cassowary::{Expression, Solver, Term};
use nannou::prelude::*;
use nannou::Draw;
use nannou_egui::egui::math::Numeric;
use nannou_egui::egui::plot::Points;
use nannou_egui::egui::Key;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Node {
    // pub selected: bool,
    pub rect: MyRect,
}

impl Node {
    pub fn new(selected: bool, minbox: f32) -> Self {
        let rect = MyRect::new(minbox);

        Node {
            // selected: selected,
            rect: rect,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InsertionWay {
    VERT,
    HORI,
}

#[derive(Clone, Debug)]
pub enum LayoutVecType {
    Layout(MyLayout),
    Node(Node),
}

#[derive(Clone, Debug)]
pub struct MyLayout {
    pub nodes: Vec<Rc<RefCell<LayoutVecType>>>,
    pub direc: InsertionWay,
    pub constr: Vec<Constraint>,
    pub vert_constraint: Option<Constraint>,
    pub horz_constraint: Option<Constraint>,
    pub selected: i32,
}

impl MyLayout {
    pub fn new(insertway: InsertionWay) -> Self {
        MyLayout {
            nodes: Vec::new(),
            direc: insertway,
            constr: Vec::new(),
            vert_constraint: None,
            horz_constraint: None,
            selected: -1,
        }
    }

    pub fn insert_with_constraint(
        &mut self,
        node: &Node,
        insertway: InsertionWay,
        padding: f32,
        solver: &mut Solver,
    ) -> Option<Vec<Constraint>> {
        if self.nodes.len() == 0 {
            self.nodes
                .push(Rc::new(RefCell::new(LayoutVecType::Node(node.clone()))));
            self.direc = insertway;
            return None;
        }

        // can not call inside loop
        // because the self.node is borrowed mutable and
        // these two functions's self == this self
        let mut first_height = self.first_node_height(padding);
        let mut first_width = self.first_node_width(padding);

        // change direction if node len = 1
        // and the insertionway changes
        // if self.nodes.len() == 1 {
        //     self.nodes
        //         .push(Rc::new(RefCell::new(LayoutVecType::Node(node.clone()))));
        //     self.direc = insertway;
        //
        //     // fixed side means for:
        //     // insertway = vert ==> all node's height should be same
        //     // insertway = hori ==> all node's width should be same
        //     let mut fixed_side_constraints = match self.direc {
        //         InsertionWay::HORI => Constraint::new(
        //             first_width - node.rect.width(),
        //             cassowary::RelationalOperator::Equal,
        //             REQUIRED,
        //         ),
        //         InsertionWay::VERT => Constraint::new(
        //             first_height - node.rect.height(),
        //             cassowary::RelationalOperator::Equal,
        //             REQUIRED,
        //         ),
        //     };
        //     // push and remove as index
        //     //
        //     //  |N|N|N|L|N|L|N|
        //     //   ^ 0 1   2   3
        //     //   reference
        //     self.constr.insert(1, fixed_side_constraints.clone());
        //     // solver.add_constraint(fixed_side_constraints);
        //     // return;
        //     return Some(fixed_side_constraints);
        // }

        let mut target = 0;
        let mut found_target = false;

        if self.nodes.len() == 1 {
            self.direc = insertway;
        }

        for i in 0..self.nodes.len() {
            match *self.nodes.to_owned()[i].as_ref().borrow_mut() {
                LayoutVecType::Node(ref n) => {
                    // if selected and direction same
                    // insert directly
                    if i as i32 == self.selected && self.direc == insertway {
                        self.nodes.insert(
                            i + 1,
                            Rc::new(RefCell::new(LayoutVecType::Node(node.clone()))),
                        );
                        println!("insert finish, len = {}", self.nodes.len());
                        let mut fixed_side_constraint = match self.direc {
                            InsertionWay::HORI => Constraint::new(
                                first_width - node.rect.width(),
                                cassowary::RelationalOperator::Equal,
                                REQUIRED,
                            ),
                            InsertionWay::VERT => Constraint::new(
                                first_height - node.rect.height(),
                                cassowary::RelationalOperator::Equal,
                                REQUIRED,
                            ),
                        };
                        if i == self.nodes.len() - 2 {
                            self.constr.push(fixed_side_constraint.clone());
                        } else {
                            self.constr.insert(i + 1, fixed_side_constraint.clone());
                        }
                        // solver.add_constraint(fixed_side_constraint);
                        return Some(Vec::from([fixed_side_constraint]));
                    }

                    // if selected and direction not same
                    // do the following
                    // 1. mark the index
                    // 2. create new layout
                    // 3. insert origin as reference
                    // 4. insert new node
                    if i as i32 == self.selected && self.direc != insertway {
                        target = i;
                        println!("change dir finish, len = {}", self.nodes.len());
                        break;
                    }
                }
                LayoutVecType::Layout(ref mut l) => {
                    l.insert_with_constraint(node, insertway, padding, solver);
                }
            }
        }

        let mut new_layout = MyLayout::new(insertway);
        let mut new_layout_height = Expression {
            terms: Vec::new(),
            constant: 0.0,
        };
        let mut new_layout_width = None;
        // let mut new_layout_width = Expression {
        //     terms: Vec::new(),
        //     constant: 0.0,
        // };
        // replace with layout:
        // |N|N|N|N| -> press H at position 2 for example
        //
        // ==> |N|L|N|N|
        //
        // ==> |N|-|N|N|
        //     | |N| | |
        //     | |N| | |
        //     | |-| | |
        //
        // ==> horizental layout will return
        //     1. same width
        //     2. layout height
        //
        // use 2. to match vertical layout's width constraint
        //
        self.nodes.get(target).unwrap().replace_with(|self_| {
            new_layout
                .nodes
                .push(Rc::new(RefCell::new(self_.to_owned())));
            new_layout.select_first();
            solver.remove_constraint(&self.constr[target]).unwrap();
            self.constr.remove(target);
            match insertway {
                InsertionWay::VERT => {
                    // new_layout_height =
                    //     new_layout.insert_with_constraint(node, insertway, padding, solver);
                    // new_layout_width = new_layout.first_layout_width(padding);
                }
                InsertionWay::HORI => {
                    new_layout_width =
                        new_layout.insert_with_constraint(node, insertway, padding, solver);
                    new_layout_height = new_layout.first_layout_height(padding);
                }
            }

            LayoutVecType::Layout(new_layout)
        });
        if target == 0 {
            // all nodes should be reconstraint
        } else {
            // origin constraint in constr should be replace
            let mut new_layout_constraints = Vec::new();
            let mut new_layout_height_constraint = Constraint::new(
                first_height - new_layout_height.clone(),
                cassowary::RelationalOperator::Equal,
                REQUIRED,
            );

            self.constr
                .insert(target, new_layout_height_constraint.clone());
            new_layout_constraints.push(new_layout_height_constraint.clone());

            match new_layout_width {
                Some(ref mut c) => {
                    new_layout_constraints.append(c);
                }
                None => {}
            };

            return Some(new_layout_constraints);
        }
        None
    }
    pub fn remove(&mut self, minbox: f32, solver: &mut Solver) -> bool {
        let mut target = 0;
        let mut find_target = false;
        let mut target_node: Option<Node> = None;
        let mut inside_layout_delete_change_dir = false;

        for i in 0..self.nodes.len() {
            match *self.nodes.to_owned()[i].as_ref().borrow_mut() {
                LayoutVecType::Node(ref n) => {
                    if i as i32 == self.selected {
                        target = i;
                        find_target = true;
                        target_node = Some(n.clone());

                        break;
                    }
                }
                LayoutVecType::Layout(ref mut l) => {
                    if l.remove(minbox, solver) {
                        inside_layout_delete_change_dir = true;
                        target = i;
                        break;
                    }
                }
            }
        }

        if inside_layout_delete_change_dir {
            // remove old constraint
            // add new node
            self.nodes.get(target).unwrap().replace_with(|self_| {
                let mut new_node = Node::new(true, minbox);
                LayoutVecType::Node(new_node)
            });
            return false;
        }

        if find_target {
            // remove constraint in layout
            match target_node {
                Some(n) => {
                    solver.remove_edit_variable(n.rect.top).unwrap();
                    solver.remove_edit_variable(n.rect.bottom).unwrap();
                    solver.remove_edit_variable(n.rect.left).unwrap();
                    solver.remove_edit_variable(n.rect.right).unwrap();
                }
                None => {}
            }

            // remove node in vec
            self.nodes.remove(target);

            // select next or prev
        }

        if self.nodes.len() == 0 {
            return true;
        }

        return false;
    }

    pub fn first_node_width(&self, padding: f32) -> Expression {
        match *self.nodes[0].as_ref().borrow() {
            LayoutVecType::Node(ref n) => {
                return n.rect.width();
            }
            LayoutVecType::Layout(ref l) => {
                return l.first_layout_width(padding);
            }
        }
    }

    pub fn first_layout_width(&self, padding: f32) -> Expression {
        let mut expression = Expression::new(Vec::new(), 0.0);
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    expression = expression + n.rect.width();
                }
                LayoutVecType::Layout(ref l) => {
                    expression = expression + l.any_layout_width(padding);
                }
            }
        }
        expression = expression + (self.nodes.len() - 1) as f32 * padding;
        expression
    }

    pub fn any_layout_width(&self, padding: f32) -> Expression {
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    return n.rect.width();
                }
                LayoutVecType::Layout(ref l) => {}
            }
        }

        let expression = match (*self.nodes[0].as_ref().borrow()) {
            LayoutVecType::Layout(ref l) => l.first_layout_width(padding),
            _ => Expression {
                terms: Vec::new(),
                constant: 0.0,
            },
        };

        return expression;
    }

    pub fn first_node_height(&self, padding: f32) -> Expression {
        match *self.nodes[0].as_ref().borrow() {
            LayoutVecType::Node(ref n) => {
                return n.rect.height();
            }
            LayoutVecType::Layout(ref l) => {
                return l.first_layout_height(padding);
            }
        }
    }

    pub fn first_layout_height(&self, padding: f32) -> Expression {
        let mut expression = Expression::new(Vec::new(), 0.0);
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    expression = expression + n.rect.height();
                }
                LayoutVecType::Layout(ref l) => {
                    // we can make sure this is definately vert
                    // so we use any one of node inside layout
                    //
                    //  |L|L|L|L|...|L|
                    //   ^
                    //  worst case, eg. HORI layout with multiple HORI layout:
                    //      choose the first and get first layout height
                    //  ---------------
                    //  |L|L|L|L|...|N|
                    //               ^
                    //  second worst, but at least we can directly return;
                    expression = expression + l.any_layout_height(padding);
                }
            }
        }
        expression = expression + (self.nodes.len() - 1) as f32 * padding;
        expression
    }

    pub fn any_layout_height(&self, padding: f32) -> Expression {
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    return n.rect.height();
                }
                LayoutVecType::Layout(ref l) => {}
            }
        }

        let expression = match (*self.nodes[0].as_ref().borrow()) {
            LayoutVecType::Layout(ref l) => l.first_layout_height(padding),
            _ => Expression {
                terms: Vec::new(),
                constant: 0.0,
            },
        };

        return expression;
    }

    pub fn select_first(&mut self) {
        match *self.nodes[0].as_ref().borrow_mut() {
            LayoutVecType::Node(ref n) => {
                self.selected = 0;
            }
            LayoutVecType::Layout(ref mut l) => {
                self.selected = 0;
                l.select_first();
            }
        }
    }

    pub fn select(&mut self, key: Key, has_parent: bool) -> bool {
        if self.selected == -1 {
            return false;
        }

        match *self.nodes[self.selected as usize].as_ref().borrow_mut() {
            LayoutVecType::Node(ref n) => match (self.direc, key) {
                (InsertionWay::VERT, Key::ArrowUp) => {
                    if has_parent {
                        self.selected = -1;
                        return true;
                    }
                    return false;
                }
                (InsertionWay::VERT, Key::ArrowDown) => {
                    if has_parent {
                        self.selected = -1;
                        return true;
                    }
                    return false;
                }
                (InsertionWay::VERT, Key::ArrowLeft) => {
                    if has_parent && self.selected == 0 {
                        self.selected = -1;
                        return true;
                    }
                    if has_parent && self.selected != 0 {
                        self.selected -= 1;
                        return false;
                    }
                    if !has_parent && self.selected == 0 {
                        return false;
                    }
                    if !has_parent && self.selected != 0 {
                        self.selected -= 1;
                        return false;
                    }
                }
                (InsertionWay::VERT, Key::ArrowRight) => {
                    if has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                        self.selected = -1;
                        return true;
                    }
                    if has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                        self.selected += 1;
                        return false;
                    }
                    if !has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                        return false;
                    }
                    if !has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                        self.selected += 1;
                        return false;
                    }
                }
                (InsertionWay::HORI, Key::ArrowUp) => {
                    if has_parent && self.selected == 0 {
                        self.selected = -1;
                        return true;
                    }
                    if has_parent && self.selected != 0 {
                        self.selected -= 1;
                        return false;
                    }
                    if !has_parent && self.selected == 0 {
                        return false;
                    }
                    if !has_parent && self.selected != 0 {
                        self.selected -= 1;
                        return false;
                    }
                }
                (InsertionWay::HORI, Key::ArrowDown) => {
                    if has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                        self.selected = -1;
                        return true;
                    }
                    if has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                        self.selected += 1;
                        return false;
                    }
                    if !has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                        return false;
                    }
                    if !has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                        self.selected += 1;
                        return false;
                    }
                }
                (InsertionWay::HORI, Key::ArrowLeft) => {
                    if has_parent {
                        self.selected = -1;
                        return true;
                    }
                    return false;
                }
                (InsertionWay::HORI, Key::ArrowRight) => {
                    if has_parent {
                        self.selected = -1;
                        return true;
                    }
                    return false;
                }
                _ => {}
            },
            LayoutVecType::Layout(ref mut l) => {
                let dir_change_parent_level = l.select(key, true);
                if dir_change_parent_level {
                    match (self.direc, key) {
                        (InsertionWay::VERT, Key::ArrowUp) => {
                            if has_parent {
                                self.selected = -1;
                                return true;
                            }
                            l.select_first();
                            return false;
                        }
                        (InsertionWay::VERT, Key::ArrowDown) => {
                            if has_parent {
                                self.selected = -1;
                                return true;
                            }
                            l.select_first();
                            return false;
                        }
                        (InsertionWay::VERT, Key::ArrowLeft) => {
                            if has_parent && self.selected == 0 {
                                self.selected = -1;
                                return true;
                            }
                            if has_parent && self.selected != 0 {
                                self.selected -= 1;
                                return false;
                            }
                            if !has_parent && self.selected == 0 {
                                return false;
                            }
                            if !has_parent && self.selected != 0 {
                                self.selected -= 1;
                                return false;
                            }
                        }
                        (InsertionWay::VERT, Key::ArrowRight) => {
                            if has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                                self.selected = -1;
                                return true;
                            }
                            if has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                                self.selected -= 1;
                                return false;
                            }
                            if !has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                                return false;
                            }
                            if !has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                                self.selected -= 1;
                                return false;
                            }
                        }
                        (InsertionWay::HORI, Key::ArrowUp) => {
                            if has_parent && self.selected == 0 {
                                self.selected = -1;
                                return true;
                            }
                            if has_parent && self.selected != 0 {
                                self.selected -= 1;
                                return false;
                            }
                            if !has_parent && self.selected == 0 {
                                return false;
                            }
                            if !has_parent && self.selected != 0 {
                                self.selected -= 1;
                                return false;
                            }
                        }
                        (InsertionWay::HORI, Key::ArrowDown) => {
                            if has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                                self.selected = -1;
                                return true;
                            }
                            if has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                                self.selected -= 1;
                                return false;
                            }
                            if !has_parent && self.selected == (self.nodes.len() - 1) as i32 {
                                return false;
                            }
                            if !has_parent && self.selected != (self.nodes.len() - 1) as i32 {
                                self.selected -= 1;
                                return false;
                            }
                        }
                        (InsertionWay::HORI, Key::ArrowLeft) => {
                            if has_parent {
                                self.selected = -1;
                                return true;
                            }
                            l.select_first();
                            return false;
                        }
                        (InsertionWay::HORI, Key::ArrowRight) => {
                            if has_parent {
                                self.selected = -1;
                                return true;
                            }
                            l.select_first();
                            return false;
                        }
                        _ => {}
                    }
                }
            }
        }

        false
    }

    // pub fn select(&mut self, key: Key, select_first: bool) {
    //     match self.direc {
    //         InsertionWay::HORI => match key {
    //             Key::ArrowDown => {
    //                 let mut next_set_true = false;
    //                 for mut i in 0..self.nodes.len() {
    //                     match *self.nodes[i].as_ref().borrow_mut() {
    //                         LayoutVecType::Node(ref mut n) => {
    //                             if i == 0 && select_first {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if n.selected {
    //                                 n.selected = false;
    //                                 next_set_true = true;
    //                                 if i == self.nodes.len() - 1 {
    //                                     n.selected = true;
    //                                 }
    //                             }
    //                         }
    //                         LayoutVecType::Layout(ref mut l) => {
    //                             if i == 0 && select_first {
    //                                 l.select(key, select_first);
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 l.select(key, true);
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             Key::ArrowUp => {
    //                 let mut next_set_true = false;
    //                 for mut i in (0..self.nodes.len()).rev() {
    //                     match *self.nodes[i].as_ref().borrow_mut() {
    //                         LayoutVecType::Node(ref mut n) => {
    //                             if i == self.nodes.len() - 1 && select_first {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if n.selected {
    //                                 n.selected = false;
    //                                 next_set_true = true;
    //                                 if i == 0 {
    //                                     n.selected = true;
    //                                 }
    //                             }
    //                         }
    //                         LayoutVecType::Layout(ref mut l) => {
    //                             if i == self.nodes.len() - 1 && select_first {
    //                                 l.select(key, select_first);
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 l.select(key, true);
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => {}
    //         },
    //         InsertionWay::VERT => match key {
    //             Key::ArrowRight => {
    //                 let mut next_set_true = false;
    //                 for mut i in 0..self.nodes.len() {
    //                     match *self.nodes[i].as_ref().borrow_mut() {
    //                         LayoutVecType::Node(ref mut n) => {
    //                             if i == 0 && select_first {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if n.selected {
    //                                 n.selected = false;
    //                                 next_set_true = true;
    //                                 if i == self.nodes.len() - 1 {
    //                                     n.selected = true;
    //                                 }
    //                             }
    //                         }
    //                         LayoutVecType::Layout(ref mut l) => {
    //                             if i == 0 && select_first {
    //                                 l.select(key, select_first);
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 l.select(key, true);
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             Key::ArrowLeft => {
    //                 let mut next_set_true = false;
    //                 for mut i in (0..self.nodes.len()).rev() {
    //                     match *self.nodes[i].as_ref().borrow_mut() {
    //                         LayoutVecType::Node(ref mut n) => {
    //                             if i == self.nodes.len() - 1 && select_first {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 n.selected = true;
    //                                 return;
    //                             }
    //                             if n.selected {
    //                                 n.selected = false;
    //                                 next_set_true = true;
    //                                 if i == 0 {
    //                                     n.selected = true;
    //                                 }
    //                             }
    //                         }
    //                         LayoutVecType::Layout(ref mut l) => {
    //                             if i == self.nodes.len() - 1 && select_first {
    //                                 l.select(key, select_first);
    //                                 return;
    //                             }
    //                             if next_set_true {
    //                                 l.select(key, true);
    //                                 return;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => {}
    //         },
    //     }
    // }

    pub fn print(&self, solver: &Solver) {
        println!("insertway: {:?}", self.direc);
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    let top = solver.get_value(n.rect.top);
                    let left = solver.get_value(n.rect.left);
                    let right = solver.get_value(n.rect.right);
                    let bottom = solver.get_value(n.rect.bottom);
                    println!("{}: {} {} {} {}", i, top, left, right, bottom);
                }
                LayoutVecType::Layout(ref l) => l.print(solver),
            }
        }
        println!("end");
    }

    pub fn draw(
        &self,
        draw: &Draw,
        solver: &Solver,
        mut reference: Rect,
        mut window: &Rect,
        padding: f32,
    ) -> Rect {
        let mut wh = Vec2::new(0.0, 0.0);
        let mut xy = Vec2::new(0.0, 0.0);
        let old_reference = reference.clone();
        for i in 0..self.nodes.len() {
            match *self.nodes[i].as_ref().borrow() {
                LayoutVecType::Node(ref n) => {
                    let top = solver.get_value(n.rect.top);
                    let left = solver.get_value(n.rect.left);
                    let right = solver.get_value(n.rect.right);
                    let bottom = solver.get_value(n.rect.bottom);

                    let mut rect = Rect::from_w_h((right - left) as f32, (bottom - top) as f32);

                    if reference == *window {
                        rect = rect.top_left_of(reference);
                        xy = rect.xy();
                    } else {
                        match self.direc {
                            InsertionWay::VERT => {
                                // rect = rect.top_left_of(reference);
                                rect = rect.right_of(reference).shift_x(padding);
                                wh.x += rect.w();
                                wh.y = rect.h();
                                xy.x += rect.w() / 2.;
                            }
                            InsertionWay::HORI => {
                                // rect = rect.top_left_of(reference);
                                rect = rect.below(reference).shift_y(-padding);
                                wh.x = rect.w();
                                wh.y += rect.h();
                                xy.y += rect.h() / 2.;
                            }
                        }
                    }

                    reference = rect.clone();

                    // reference = match self.direc {
                    //     InsertionWay::VERT => {
                    //         Rect::from_corners(rect.top_right(), window.bottom_right())
                    //             .pad_left(padding)
                    //     }
                    //     InsertionWay::HORI => {
                    //         Rect::from_corners(rect.bottom_left(), window.bottom_right())
                    //             .pad_top(padding)
                    //     }
                    // };

                    if i as i32 == self.selected {
                        draw.rect().wh(rect.wh()).xy(rect.xy()).color(RED);
                    } else {
                        draw.rect().wh(rect.wh()).xy(rect.xy());
                    }
                }
                LayoutVecType::Layout(ref l) => match self.direc {
                    InsertionWay::HORI => {
                        let mut new_window = match (reference == *window) {
                            true => *window,
                            false => {
                                Rect::from_corners(reference.bottom_left(), window.bottom_right())
                                    .shift_y(padding)
                            }
                        };
                        reference = l.draw(draw, solver, new_window, &new_window, padding);
                        // println!("{:?}", reference);
                    }
                    InsertionWay::VERT => {
                        let mut new_window = match (reference == *window) {
                            true => *window,
                            false => {
                                Rect::from_corners(reference.top_right(), window.bottom_right())
                                    .shift_x(padding)
                            }
                        };
                        reference = l.draw(draw, solver, new_window, &new_window, padding);
                        // println!("{:?}", reference);
                    }
                },
            }
        }
        let mut full_layout_rect = Rect::from_xy_wh(wh, xy);
        full_layout_rect
    }
}
