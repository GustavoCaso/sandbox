fn main() {
    let mut v: Vec<i32> = Vec::new();

    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);

    println!("{:?}", v);

    println!("let's update the third element in place");

    let third = &mut v[2];
    *third = 6;
    println!("{:?}", v);

    let third: Option<&i32> = v.get(2);
    match third {
        Some(third) => println!("The third element is {third}"),
        None => println!("There is no third element."),
    }

    // let does_not_exist = &v[100]; This format panics
    let _does_not_exist = v.get(100);
    // println!("No element in the 101 position {does_not_exist}");
}
