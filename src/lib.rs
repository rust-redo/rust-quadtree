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
use serde_wasm_bindgen::to_value;
use serde_wasm_bindgen::from_value;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct QuadtreePoint {
    data: [f64; 2],
}

enum QuadtreeNode {
    Node(Rc<RefCell<[QuadtreeNode; 4]>>),
    Leaf(Rc<RefCell<QuadtreePoint>>),
    Nil,
}

impl Clone for QuadtreeNode {
  fn clone(&self) -> Self {
      match self {
          QuadtreeNode::Node(node) => QuadtreeNode::Node(Rc::clone(node)),
          QuadtreeNode::Leaf(leaf) => QuadtreeNode::Leaf(Rc::clone(leaf)),
          QuadtreeNode::Nil => QuadtreeNode::Nil
      }
  }
}


impl QuadtreeNode {
  pub fn init_node() -> Rc<RefCell<[QuadtreeNode;4]>> {
    Rc::new(RefCell::new([
      QuadtreeNode::Nil,
      QuadtreeNode::Nil,
      QuadtreeNode::Nil,
      QuadtreeNode::Nil,
    ]))
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

  // https://github.com/d3/d3-quadtree/blob/main/src/cover.js
  pub fn cover(&mut self, x: f64, y: f64) {
    if self.x0.is_none() {
      self.x0 = Some(x.floor());
      self.y0 = Some(y.floor());
      self.x1 = Some(x.floor() + 1.0);
      self.y1 = Some(y.floor() + 1.0);
      return;
    }

    let mut x0 = self.x0.unwrap();
    let mut x1 = self.x1.unwrap();
    let mut y0 = self.y0.unwrap();
    let mut y1 = self.y1.unwrap();
    let mut z = x1 - x0;
    let mut node = self.root.clone();
    let mut i:usize;
    let mut parent: Rc<RefCell<[QuadtreeNode;4]>>;

    while x0 > x || x >= x1 || y0 > y || y >= y1 {
      i = (if y < y0 {1} else {0}) << 1 | (if x < x0 {1} else {0});
      parent = QuadtreeNode::init_node();
      parent.borrow_mut()[i] = node.clone();
      node = QuadtreeNode::Node(parent);
      z *= 2.0;

      match i {
        0 => {
          x1 = x0 + z;
          y1 = y0 + z;
        },
        1 => {
          x0 = x1 - z;
          y1 = y0 + z;
        },
        2 => {
          x1 = x0 + z;
          y0 = y1 - z;
        },
        3 => {
          x0 = x1 - z;
          y0 = y1 - z;
        },
        _ => {}
      }
    }

    match self.root {
      QuadtreeNode::Node(_) => {
        self.root = node;
      },
      _ => {}
    }

    self.x0 = Some(x0);
    self.y0 = Some(y0);
    self.x1 = Some(x1);
    self.y1 = Some(y1);
  }

  pub fn extent(&mut self, origin_points: JsValue) {
    let mut points;
    match from_value::<[[f64;2];2]>(origin_points) {
      Ok(p) => points = p,
      Err(_) => {
        return
      }
    }
  }
}