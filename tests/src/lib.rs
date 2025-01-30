#[cfg(test)]
mod tests {
    use error_proc_macros::EnumError;

    #[test]
    fn top_level_format() {
        #[derive(EnumError)]
        #[format = "an error occured: {}"]
        enum MyError {
            Foo(i8),
        }

        assert_eq!(String::from("an error occured: 10"), MyError::Foo(10).to_string());
    }

    #[test]
    fn discriminant_error() {
        #[derive(EnumError)]
        #[format = "returned with error: {}"]
        enum MyError {
            NotFound = 404,
        }

        assert_eq!(String::from("returned with error: 404"), MyError::NotFound.to_string());
    }
    #[test]
    fn unit_error() {
        #[derive(EnumError)]
        enum MyError {
            #[format = "unexpected null pointer"]
            NullError,
        }

        assert_eq!(String::from("unexpected null pointer"), MyError::NullError.to_string());
    }
}
