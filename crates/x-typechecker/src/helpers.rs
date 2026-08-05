pub(crate) use x_utils::sanitize_symbol as sanitize_symbol_component;

#[macro_export]
macro_rules! function_signature {
    ($params:expr => $return_type:expr) => {
        $crate::function_signature!($params => $return_type, throws: false, is_static: false)
    };
    ($params:expr => $return_type:expr, throws: $is_throws:expr, is_static: $is_static:expr $(,)?) => {
        $crate::FunctionSignature {
            parameter_types: $params,
            return_type: $return_type,
            is_throws: $is_throws,
            is_pure: false,
            is_static: $is_static,
        }
    };
}
