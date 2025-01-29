use error_proc_macros::*;

#[derive(EnumError)]
enum MyError {
    #[message = "foo"]
    Foo,
    Bar(u8),
    Baz(u8),
}

fn main() {
}
