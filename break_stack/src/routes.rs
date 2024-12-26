#[macro_export]
macro_rules! build_router {
    ( $app_state:ident, $( ($path_id:ident, $path_fmt:expr, ( $( path -> $( $path_arg:ident : $path_arg_t:ty => $fmt_arg:expr ),* )? ), $handler:expr) , )* ) => {
        #[allow(dead_code)]
        pub mod route_paths {
            $(
                pub fn $path_id($($($path_arg : $path_arg_t),*)*) -> String {
                    format!($path_fmt, $($($path_arg),*)*)
                }
            )*
        }
        pub fn router() -> ::axum::routing::Router<$app_state> {
            let mut router = ::axum::routing::Router::new();
            $(
                router = {
                    let path = format!($path_fmt, $($($fmt_arg),*)*);
                    router.route(&path, $handler)
                };
            )*
            router
        }
    };
}
pub use build_router;
