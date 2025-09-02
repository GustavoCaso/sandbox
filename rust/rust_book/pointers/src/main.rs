/// The most straightforward smart pointer is a box, whose type is written Box<T>.
/// Boxes allow you to store data on the heap rather than the stack.
/// What remains on the stack is the pointer to the heap data.
///
/// Boxes don’t have performance overhead, other than storing their data on the heap instead of on the stack. But they don’t have many extra capabilities either. You’ll use them most often in these situations:
/// - When you have a type whose size can’t be known at compile time and you want to use a value of that type in a context that requires an exact size
/// - When you have a large amount of data and you want to transfer ownership but ensure the data won’t be copied when you do so
/// - When you want to own a value and you care only that it’s a type that implements a particular trait rather than being of a specific type
///
/// The Rc<T> type keeps track of the number of references to a value to determine whether or not the value is still in use.
/// If there are zero references to a value, the value can be cleaned up without any references becoming invalid.
use pointers::List::{Cons, Nil};
use pointers::MyBox;
use pointers::RCList;
use std::rc::Rc;

fn main() {
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    let x = 5;
    let y = MyBox::new(x);

    assert_eq!(5, x);
    assert_eq!(5, *y);

    let rc_list = Rc::new(RCList::Cons(
        5,
        Rc::new(RCList::Cons(10, Rc::new(RCList::Nil))),
    ));
    println!(
        "count after creating rc_list = {}",
        Rc::strong_count(&rc_list)
    );
    let rc_list_b = RCList::Cons(3, Rc::clone(&rc_list));
    println!(
        "count after creating rc_list_b = {}",
        Rc::strong_count(&rc_list)
    );
    {
        let rc_list_c = RCList::Cons(4, Rc::clone(&rc_list));
        println!(
            "count after creating rc_list_c = {}",
            Rc::strong_count(&rc_list)
        );
    }
    println!(
        "count after rc_list_c goes out of scope = {}",
        Rc::strong_count(&rc_list)
    );
}
