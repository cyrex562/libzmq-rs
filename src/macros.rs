#![allow(unused_variables)]

// Internal marker traits
pub trait ZmqObject {}

// Unused object handling (though rarely needed in Rust due to _ pattern)
#[inline(always)]
pub fn unused<T>(object: T) {
    // Equivalent to LIBZMQ_UNUSED
}

// Trait for non-copyable and non-movable types
pub trait NonCopyableNorMovable {}

// Implementation helper macro for non-copyable types
#[macro_export]
macro_rules! impl_non_copyable_nor_movable {
    ($type:ty) => {
        impl NonCopyableNorMovable for $type {}
    };
}

// Constants for feature detection (though not typically needed in Rust)
pub const ZMQ_HAVE_NOEXCEPT: bool = true;

// Note: Most of the C++ macros like ZMQ_NOEXCEPT, ZMQ_OVERRIDE, ZMQ_FINAL
// are not needed in Rust as they're part of the language:
// - noexcept -> Rust is already exception-free
// - override -> impl automatically handles this
// - final -> Rust has 'sealed' traits or 'sealed' keyword (unstable)
// - default -> #[derive(Default)] or impl Default
