use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::{Color, Computed, Point};

#[derive(Clone)]
pub enum Property {
    String(Computed<String>),
    Color(Computed<Color>),
    Point(Computed<Point>),
    Float(Computed<f32>),
    Int(Computed<i32>),
    Any(Rc<dyn std::any::Any>),
}

impl Property {
    pub fn as_string(&self) -> Computed<String> {
        if let Self::String(out) = self {
            return out.clone();
        }
        panic!("This property doesn't contain a string.");
    }

    pub fn as_color(&self) -> Computed<Color> {
        if let Self::Color(out) = self {
            return out.clone();
        }
        panic!("This property doesn't contain a color.");
    }

    pub fn as_point(&self) -> Computed<Point> {
        if let Self::Point(out) = self {
            return out.clone();
        }
        panic!("This property doesn't contain a point.");
    }

    pub fn as_float(&self) -> Computed<f32> {
        if let Self::Float(out) = self {
            return out.clone();
        }
        panic!("This property doesn't contain a float.");
    }

    pub fn as_int(&self) -> Computed<i32> {
        if let Self::Int(out) = self {
            return out.clone();
        }
        panic!("This property doesn't contain an int.");
    }

    pub fn as_any<T: 'static>(&self) -> &T {
        if let Self::Any(out) = self {
            return out.downcast_ref().expect("The Any didn't contain the expected type.");
        }
        panic!("This property doesn't contain an Any.");
    }
}

#[derive(Clone, Eq, Hash, Copy)]
pub struct PropertyId {
    pub id: usize,
    #[cfg(debug_assertions)]
    name: &'static str,
}

impl PartialEq for PropertyId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PropertyId {
    pub const fn new(id: usize, #[allow(unused)] name: &'static str) -> PropertyId {
        Self {
            id,
            #[cfg(debug_assertions)]
            name,
        }
    }
}

#[derive(Default)]
pub struct PropertyStore {
    map: FxHashMap<PropertyId, Property>,
    fetched: Vec<PropertyId>,
}

impl PropertyStore {
    pub fn set(&mut self, key: PropertyId, value: Property) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: PropertyId) -> Option<&Property> {
        self.map.get(&key)
    }

    pub fn set_dirty(&mut self, id: PropertyId, dirty: bool) {
        if dirty {
            if let Some(idx) = self.fetched.iter().position(|i| id == *i) {
                self.fetched.remove(idx);
            }
        } else {
            self.fetched.push(id);
        }
    }

    pub fn is_dirty(&self, id: PropertyId) -> bool {
        !self.fetched.contains(&id)
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn contains(&self, key: PropertyId) -> bool {
        self.map.contains_key(&key)
    }

    pub fn remove(&mut self, key: PropertyId) -> Option<Property> {
        self.map.remove(&key)
    }
}
