use std::ops::Add;

#[derive(Debug)]
struct Value(i8);

impl Add<Option<i8>> for Value {
    type Output = Value;

    fn add(self, other: Option<i8>) -> <Self as Add<Option<i8>>>::Output {
        match other {
            Some(x) => Value(self.0 + x),
            None => self,
        }
    }
}

fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

fn main() {
    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);

    dbg!("{}, {}", six, none);

    let value = Value(8);
    let some_value = Some(9);

    let sum = value + some_value;

    println!("{:?}", sum);

    println!("{:?}", sum + None);
}
