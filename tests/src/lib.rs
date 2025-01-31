#[cfg(test)]
mod tests {
    use error_proc_macros::{
        Error,
        EnumError,
        StructError,
    };
    #[test]
    fn error() {
        use std::boxed::Box;

        #[derive(Debug, Error, StructError)]
        #[format = "placeholder"]
        pub struct TestError;

        let _: Box<dyn std::error::Error> = Box::new(TestError);
        assert!(true);
    }

    #[test]
    fn enum_lifetimes() {
        #[derive(EnumError)]
        enum TestError<'a> {
            text(&'a str),
        }

        assert!(true);
    }

    #[test]
    fn enum_top_level_format() {
        #[derive(EnumError)]
        #[format = "an error occured: {}"]
        enum TestError {
            Foo(i8),
        }

        assert_eq!(String::from("an error occured: 10"), TestError::Foo(10).to_string());
    }

    #[test]
    fn enum_discriminant() {
        #[derive(EnumError)]
        #[format = "returned with error: {}"]
        enum TestError {
            NotFound = 404,
        }

        assert_eq!(String::from("returned with error: 404"), TestError::NotFound.to_string());
    }
    #[test]
    fn enum_struct() {

        #[derive(EnumError)]
        enum TestError {
            #[format = "decoding error at {file} {offset}"]
            Decoding { file: &'static str, offset: usize },
        }

        assert_eq!(String::from("decoding error at foo.mp3 10"), TestError::Decoding { file: "foo.mp3", offset: 10 }.to_string());
    }
    #[test]
    fn enum_tuple() {
        #[derive(EnumError)]
        enum TestError {
            #[format = "lorem ipsum {arg_0} {arg_1}"]
            Foo(i8, u8)
        }

        assert_eq!(String::from("lorem ipsum 1 2"), format!("{}", TestError::Foo(1, 2)));
    }
    #[test]
    fn enum_unit() {
        #[derive(EnumError)]
        enum TestError {
            #[format = "unexpected null pointer"]
            NullError,
        }

        assert_eq!(String::from("unexpected null pointer"), TestError::NullError.to_string());
    }

    #[test]
    fn struct_lifetimes() {
        #[derive(StructError)]
        #[format = "placeholder"]
        struct FooError<'a> { text: &'a str, }
        assert!(true);
    }
    #[test]
    fn struct_named() {
        #[derive(StructError)]
        #[format = "{x} {y}"]
        struct TestError { x: u32, y: u32 }

        assert_eq!(String::from("69 420"), format!("{}", TestError { x: 69, y: 420 }))
    }
    #[test]
    fn struct_single_tuple() {
        #[derive(StructError)]
        #[format = "{}"]
        struct TestError(&'static str);

        assert_eq!(String::from("foo"), format!("{}", TestError("foo")))
    }
    #[test]
    fn struct_tuple() {
        #[derive(StructError)]
        #[format = "{arg_0} says {arg_1}"]
        struct TestError(&'static str, &'static str);

        assert_eq!(String::from("foo says bar"), format!("{}", TestError("foo", "bar")))
    }
    #[test]
    fn struct_unit() {
        #[derive(StructError)]
        #[format = "an error occurred"]
        struct TestError;

        assert_eq!(String::from("an error occurred"), format!("{}", TestError))
    }
}
