pub use break_stack_macros::Component;

pub trait ComponentAsRef {
    type Ref;
    fn as_ref(self) -> Self::Ref;
}

pub trait Component: axum::response::IntoResponse {}
