use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
use cassowary::Constraint;
use cassowary::Expression;
use cassowary::PartialConstraint;
use cassowary::RelationalOperator;
use cassowary::Solver;
use cassowary::Variable;
use cassowary::WeightedRelation;
use nannou::draw::primitive::Rect;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

#[derive(Debug, Hash, Clone)]
pub struct MyRect {
    pub top: Variable, // x
    pub bottom: Variable,
    pub left: Variable, //y
    pub right: Variable,
    pub border_constr: Option<Vec<Constraint>>,
}

impl MyRect {
    pub fn new(minbox: f32) -> Self {
        let mut rect = MyRect {
            top: Variable::new(),
            bottom: Variable::new(),
            left: Variable::new(),
            right: Variable::new(),
            border_constr: None,
        };
        rect
    }

    pub fn width(&self) -> Expression {
        self.right - self.left
    }

    pub fn negwidth(&self) -> Expression {
        -(self.right - self.left)
    }

    pub fn height(&self) -> Expression {
        self.bottom - self.top
    }

    pub fn origin_x(&self) -> Expression {
        (self.left + self.right) / 2.0
    }

    pub fn origin_y(&self) -> Expression {
        (self.bottom + self.top) / 2.0
    }

    pub fn init_constr(&mut self, constraints: &Vec<Constraint>) {
        self.border_constr = Some(constraints.to_owned());
    }
}

impl Eq for MyRect {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MyRect {
    fn eq(&self, other: &Self) -> bool {
        self.top.eq(&other.top)
            && self.bottom.eq(&other.bottom)
            && self.left.eq(&other.left)
            && self.right.eq(&other.right)
    }
    fn ne(&self, other: &Self) -> bool {
        self.top.ne(&other.top)
            || self.bottom.ne(&other.bottom)
            || self.left.ne(&other.left)
            || self.right.ne(&other.right)
    }
}
