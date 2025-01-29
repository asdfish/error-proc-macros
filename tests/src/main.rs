use error_proc_macros::{
    *,
};

#[derive(Debug, Error, StructError)]
#[format = "{foo}"]
struct MyError {
    foo: &'static str
}

fn main() {
    let my_error = MyError {
        foo: "hello world"
    };
    println!("{}", my_error);
}
