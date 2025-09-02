/// Because Rust can’t figure out how much space to allocate for recursively defined types, the compiler gives an error with this helpful suggestion:
/// Boxes provide only the indirection and heap allocation; they don’t have any other special capabilities

pub enum List {
    Cons(i32, Box<List>),
    Nil,
}

use std::ops::Deref;

pub struct MyBox<T>(T);

impl<T> MyBox<T> {
    pub fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::rc::Rc;

/// We could change the definition of Cons to hold references instead,
/// but then we would have to specify lifetime parameters.
/// By specifying lifetime parameters, we would be specifying that every element in the list will live at least as long as the entire list.
/// To vaoid that we use Rc
pub enum RCList {
    Cons(i32, Rc<RCList>),
    Nil,
}
