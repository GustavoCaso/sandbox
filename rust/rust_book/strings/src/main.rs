fn main() {
    let mut s = String::from("initial comments");
    let hello = String::from("你好");
    // 你
    // 好
    for c in hello.chars() {
        println!("{c}");
    }

    // 228
    // 189
    // 160
    // 229
    // 165
    // 189
    for c in hello.bytes() {
        println!("{c}");
    }

    println!("{}, {}", s, hello);

    s.push_str(" after println");
    println!("{}", s);

    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2; // note s1 has been moved here and can no longer be used
    let s4 = format!("{s3} Foo-Bar");

    // println!("{}", s1); Invalid
    println!("{}", s2);
    println!("{}", s3);
    println!("{}", s4);
}
