#[cfg(test)]
mod tests {
    use error_proc_macros::EnumError;

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
    fn enum_discriminant_error() {
        #[derive(EnumError)]
        #[format = "returned with error: {}"]
        enum TestError {
            NotFound = 404,
        }

        assert_eq!(String::from("returned with error: 404"), TestError::NotFound.to_string());
    }
    #[test]
    fn enum_struct_error() {
        use std::path::Path;

        #[derive(EnumError)]
        enum TestError {
            #[format = "decoding error at {file} {offset}"]
            Decoding { file: &'static str, offset: usize },
        }

        assert_eq!(String::from("decoding error at foo.mp3 10"), TestError::Decoding { file: "foo.mp3", offset: 10 }.to_string());
    }
    #[test]
    fn enum_unit_error() {
        #[derive(EnumError)]
        enum TestError {
            #[format = "unexpected null pointer"]
            NullError,
        }

        assert_eq!(String::from("unexpected null pointer"), TestError::NullError.to_string());
    }
}
