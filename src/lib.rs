mod utils;

use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Reflect;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::option::Option;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct QuadtreePoint {
    data: [f64; 2],
}

enum QuadtreeNode {
    Set(Rc<RefCell<[QuadtreeNode; 4]>>),
    List(Rc<RefCell<LinkedList<QuadtreePoint>>>),
    Nil,
}

impl Clone for QuadtreeNode {
    fn clone(&self) -> QuadtreeNode {
        match self {
            QuadtreeNode::Set(set) => QuadtreeNode::Set(Rc::clone(set)),
            QuadtreeNode::List(point) => QuadtreeNode::List(Rc::clone(point)),
            QuadtreeNode::Nil => QuadtreeNode::Nil,
        }
    }
}

impl QuadtreeNode {
    fn new_set() -> [QuadtreeNode; 4] {
        [
            QuadtreeNode::Nil,
            QuadtreeNode::Nil,
            QuadtreeNode::Nil,
            QuadtreeNode::Nil,
        ]
    }

    fn new_set_rc() -> Rc<RefCell<[QuadtreeNode; 4]>> {
        Rc::new(RefCell::new(QuadtreeNode::new_set()))
    }

    fn new_set_node() -> QuadtreeNode {
        QuadtreeNode::Set(QuadtreeNode::new_set_rc())
    }
}

fn default_x(d: [f64; 2]) -> f64 {
    d[0]
}

fn default_y(d: [f64; 2]) -> f64 {
    d[1]
}

#[wasm_bindgen]
pub struct Quadtree {
    x0: Option<f64>,
    y0: Option<f64>,
    x1: Option<f64>,
    y1: Option<f64>,
    root: QuadtreeNode,
}

#[wasm_bindgen]
impl Quadtree {
    pub fn new() -> Quadtree {
        Quadtree {
            x0: None,
            y0: None,
            x1: None,
            y1: None,
            root: QuadtreeNode::Nil,
        }
    }

    pub fn visit(&self, callback: Function) {
        let null = JsValue::null();
        let args = Array::new_with_length(5);
        callback.apply(&null, &args);
        let mut queue: Vec<(&QuadtreeNode, f64, f64, f64, f64)> = Vec::new();
        let node = &self.root;
        if let QuadtreeNode::Nil = node {
            return;
        }

        queue.push((
            node,
            self.x0.unwrap(),
            self.y0.unwrap(),
            self.x1.unwrap(),
            self.y1.unwrap(),
        ));

        loop {
            let q = queue.pop();

            match q {
                Some((current, x0, y0, x1, y1)) => {
                    let js_node = Object::new();
                    Reflect::set(&js_node, &"x".into(), &x0.into());
                    let _ = JsValue::from_f64(1.0) + JsValue::from_f64(2.0);
                }
                None => {
                    return;
                }
            }
        }
    }

    pub fn add_all(&mut self, x_data: Array, y_data: Array) {
        let mut data = Vec::new();
        for i in 0..x_data.length() {
            data.push([
                x_data.get(i).as_f64().unwrap(),
                y_data.get(i).as_f64().unwrap(),
            ])
        }

        let mut min_x = default_x(data[0]);
        let mut max_x = min_x;

        let mut min_y = default_y(data[0]);
        let mut max_y = min_y;

        for i in 1..data.len() {
            let x = default_x(data[i]);
            let y = default_y(data[i]);

            if x < min_x {
                min_x = x;
            }

            if x > max_x {
                max_x = x;
            }

            if y < min_y {
                min_y = y;
            }

            if y > max_y {
                max_y = y;
            }
        }

        if min_x > max_x || min_y > max_y {
            return;
        }

        self.cover(min_x, min_y);
        self.cover(max_x, max_y);

        for d in data {
            self.add(d);
        }
    }

    fn cover(&mut self, x: f64, y: f64) {
        match self.x0 {
            Some(mut x0) => {
                let mut node = self.root.clone();

                let mut y0 = self.y0.unwrap();
                let mut x1 = self.x1.unwrap();
                let mut y1 = self.y1.unwrap();
                let mut i;
                let mut z;
                let mut parent;
                if x1 > x0 {
                    z = x1 - x0;
                } else {
                    z = 1.0;
                }

                while x0 > x || x >= x1 || y0 > y || y >= y1 {
                    i = ((y < y0) as usize) << 1 | ((x < x0) as usize);
                    parent = QuadtreeNode::new_set();
                    parent[i] = node;
                    node = QuadtreeNode::Set(Rc::new(RefCell::new(parent)));
                    z = z * 2.0;
                    match i {
                        0 => {
                            x1 = x0 + z;
                            y1 = y0 + z;
                        }
                        1 => {
                            x0 = x1 - z;
                            y1 = y0 + z;
                        }
                        2 => {
                            x1 = x0 + z;
                            y0 = y1 - z;
                        }
                        3 => {
                            x0 = x1 - z;
                            y0 = y1 - z;
                        }
                        _ => {}
                    }
                }

                if let QuadtreeNode::Set(_) = self.root {
                    self.root = node;
                }

                self.x0 = Some(x0);
                self.y0 = Some(y0);
                self.x1 = Some(x1);
                self.y1 = Some(y1);
            }
            None => {
                let x0 = x.floor();
                let y0 = y.floor();
                self.x0 = Some(x0);
                self.y0 = Some(y0);
                self.x1 = Some(x0 + 1.0);
                self.y1 = Some(y0 + 1.0);
            }
        }
    }

    fn add(&mut self, d: [f64; 2]) {
        let x = default_x(d);
        let y = default_y(d);
        let mut x0 = self.x0.unwrap();
        let mut y0 = self.y0.unwrap();
        let mut x1 = self.x1.unwrap();
        let mut y1 = self.y1.unwrap();
        let mut right;
        let mut bottom;
        let mut i: usize = 0;
        let mut j: usize = 0;

        let mut point = LinkedList::new();
        point.push_back(QuadtreePoint { data: d });
        let point_rc = Rc::new(RefCell::new(point));
        let point_rc_2 = Rc::clone(&point_rc);
        let point_node = QuadtreeNode::List(point_rc);

        if let QuadtreeNode::Nil = self.root {
            self.root = point_node;

            return;
        }

        let mut parent = None;
        let mut next_node = Rc::new(RefCell::new(LinkedList::new()));
        let get_right_bottom = |x0: &mut f64, y0: &mut f64, x1: &mut f64, y1: &mut f64| {
            let right;
            let bottom;
            let xm = (*x0 + *x1) / 2.0;
            let ym = (*y0 + *y1) / 2.0;
            if x >= xm {
                right = 1;
                *x0 = xm;
            } else {
                right = 0;
                *x1 = xm;
            }

            if y >= ym {
                bottom = 1;
                *y0 = ym;
            } else {
                bottom = 0;
                *y1 = ym;
            }

            return (right, bottom, xm, ym);
        };
        let get_i = |right: usize, bottom: usize| bottom << 1 | right;
        let get_j =
            |xm: f64, ym: f64, xp: f64, yp: f64| ((yp >= ym) as usize) << 1 | ((xp >= xm) as usize);

        if let QuadtreeNode::Set(root) = &self.root {
            let mut node = Rc::clone(root);

            loop {
                (right, bottom, _, _) = get_right_bottom(&mut x0, &mut y0, &mut x1, &mut y1);
                i = get_i(right, bottom);

                let child = Rc::clone(&node);
                let mut node_mut = child.borrow_mut();

                if let QuadtreeNode::Nil = node_mut[i] {
                    node_mut[i] = point_node;
                    return;
                }

                match &Rc::clone(&node).borrow()[i] {
                    QuadtreeNode::Set(child) => {
                        parent = Some(node);
                        node = Rc::clone(child);
                    }
                    QuadtreeNode::List(child_point) => {
                        next_node = Rc::clone(child_point);
                        break;
                    }
                    QuadtreeNode::Nil => break,
                }
            }
        }

        let next_node_copy = Rc::clone(&next_node);
        let next_node_mut = next_node_copy.borrow_mut();

        let head = next_node_mut.front();
        let head_value = head.unwrap();
        let xp = default_x(head_value.data);
        let yp = default_y(head_value.data);

        if x == xp && y == yp {
            let mut point_rc_mut = point_rc_2.borrow_mut();
            point_rc_mut.append(&mut next_node.borrow_mut());
            match parent {
                Some(parent_node) => {
                    parent_node.borrow_mut()[i] = point_node;
                }
                None => {
                    self.root = point_node;
                }
            }
            return;
        }

        loop {
            let set = QuadtreeNode::new_set_rc();
            match parent {
                Some(ref parent_node) => {
                    parent_node.borrow_mut()[i] = QuadtreeNode::Set(set.clone());
                }
                None => {
                    self.root = QuadtreeNode::Set(set.clone());
                }
            }
            parent = Some(set);

            let mut xm;
            let mut ym;
            (right, bottom, xm, ym) = get_right_bottom(&mut x0, &mut y0, &mut x1, &mut y1);
            i = get_i(right, bottom);
            j = get_j(xm, ym, xp, yp);

            if i != j {
                break;
            }
        }

        if let Some(set) = parent {
            let mut set_temp = set.borrow_mut();
            set_temp[i] = QuadtreeNode::List(next_node);
            set_temp[j] = point_node;
        }
    }
}
