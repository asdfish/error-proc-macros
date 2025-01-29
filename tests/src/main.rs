use error_proc_macros::*;

#[derive(EnumError)]
enum MyError {
    #[format = "asdf {}"]
    Foo = 10,
}

fn main() {
    let my_err = MyError::Foo;
    println!("{}", my_err);
}
