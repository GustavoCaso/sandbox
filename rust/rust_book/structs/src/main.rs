#[derive(Debug)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

#[derive(Debug)]
struct Color(i32, i32, i32);
#[derive(Debug)]
struct Point(i32, i32, i32);

fn main() {
    let user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };

    println!("{:?}", user1);

    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };

    println!("{:?}", user2);

    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    println!("{:?}, {:?}", black, origin);
}
