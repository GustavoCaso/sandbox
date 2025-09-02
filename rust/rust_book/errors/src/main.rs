use std::fs::File;
use std::io::{self, ErrorKind, Read};

fn main() {
    // let file = match File::open("hello.txt") {
    //     Ok(file) => file,
    //     Err(error) => match error.kind() {
    //         ErrorKind::NotFound => match File::create("hello.txt") {
    //             Ok(fc) => fc,
    //             Err(e) => panic!("Problem creating the file: {:?}", e),
    //         },
    //         other_error => {
    //             panic!("Problem opening the file: {:?}", other_error);
    //         }
    //     },
    // };

    // By using unwrap_or_else we can remove the nested match expressions
    File::open("hello.txt").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|error| {
                panic!("Problem creating the file: {:?}", error);
            })
        } else {
            panic!("Problem opening the file: {:?}", error);
        }
    });

    println!(
        "{}",
        read_username_from_file().unwrap_or_else(|_| String::from("no username"))
    );
}

fn read_username_from_file() -> Result<String, io::Error> {
    // let username_file_result = File::open("username.txt");

    // let mut username_file = match username_file_result {
    //     Ok(file) => file,
    //     Err(e) => return Err(e),
    // };

    // let mut username = String::new();

    // match username_file.read_to_string(&mut username) {
    //     Ok(_) => Ok(username),
    //     Err(e) => Err(e),
    // }

    // Here the ? operator allow us to reduce a lot of bolier plate.
    // If the call to File::open  rerturns Ok it would assign it to username_file variable
    // otherwise it would return from the function call and return the error
    // The ? operator uses the Form Trait.
    // When the ? operator calls the from function, the error type received
    // is converted into the error type defined in the return type of the current function.
    // This is useful when a function returns one error type to represent all
    // the ways a function might fail, even if parts might fail for many different reasons.
    let mut username_file = File::open("username.txt")?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}
