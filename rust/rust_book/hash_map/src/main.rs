use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();

    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    // We use & to avoid moving the ownership of key and value from the hash map
    for (key, value) in &scores {
        println!("{key}: {value}")
    }

    // For types that implement the Copy trait, like i32,
    // the values are copied into the hash map. For owned values like String, the values will
    // be moved and the hash map will be the owner of those values
    // println!("{}", field_name); would fail after insert

    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");

    let mut map = HashMap::new();
    map.insert(field_name, field_value);

    // We use the entry function to check if we can insert a key value
    // or if the key exists do not overwrite the value

    scores.entry(String::from("Blue")).or_insert(100);
    println!("{:?}", scores);

    // or_entry is defined to return a mutable refernce to the value
    // We can use that API to modify hash map value on the fly.

    let text = "hello world wonderful world";

    let mut map = HashMap::new();

    for word in text.split_whitespace() {
        let count = map.entry(word).or_insert(0);
        *count += 1;
    }

    println!("{map:?}");
}
