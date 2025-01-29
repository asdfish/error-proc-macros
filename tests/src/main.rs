use error_proc_macros::EnumError;

#[derive(EnumError)]
enum MyError {
    Foo(u8),
    Bar(i8),
    Baz,
}

fn main() {
    println!("Hello, world!");
}
