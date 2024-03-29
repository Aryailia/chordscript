//run: cargo test -- --nocapture
// run: cargo run
// Here we use std::mem::transmutate;

// No dependencies here

// This macro is for ergonomics, capacity and str can be specified on one line
// This then calculates total capacity, allocates, then pushes
// A way specify length of what is pushed and do the pushing side-by-side
#[macro_export]
macro_rules! sidebyside_len_and_push {
    (
        $(! $( $prefix:ident )+ !)? $len:ident, $push_into:ident $(<$U2:ident>)?
            ($self:ident : $ty1:ty, $extra:ident : $ty2:ty, $buffer:ident: $U:ident)
        {
            $( $init:stmt; )*
        } {
            $( $stmts:tt )*
        }
    ) => {
        $( $( $prefix )* )? fn $len($self: $ty1, $extra: $ty2) -> usize {
            $( $init )*
            sidebyside_len_and_push!(@size $($stmts)*)
        }
        fn $push_into $(<$U2: $crate::templates::Consumer>)? ($self: $ty1, $extra: $ty2, $buffer: &mut $U) {
            //#[cfg(debug_assertions)]
            //{
            //    $( $init )*
            //    let size = sidebyside_len_and_push!(@size $($stmts)*);
            //    let mut temp = String::with_capacity(size);
            //    let ptr = &mut temp;
            //    sidebyside_len_and_push!(ptr @push $($stmts)*);
            //}
            $( $init )*
            sidebyside_len_and_push!($buffer @push $($stmts)*);
        }

    };

    // We support two styles of specifying a line either
    //    {} => {};
    //    {};

    // The rest of this is using the TT-mucher pattern
    (@size $size:expr => $push:expr; $($rest:tt)*) => {
        $size + sidebyside_len_and_push!(@size $($rest)*)
    };
    // Additionally we support
    (@size $str:expr; $($rest:tt)*) => {
        $str.len() + sidebyside_len_and_push!(@size $($rest)*)
    };
    (@size) => { 0 };

    ($buffer:ident @push $size:expr => $push:expr; $($rest:tt)*) => {
        $push;
        sidebyside_len_and_push!($buffer @push $($rest)*);
    };
    ($buffer:ident @push $str:literal; $($rest:tt)*) => {
        $buffer.consume($str);
        sidebyside_len_and_push!($buffer @push $($rest)*);
    };
    ($buffer:ident @push) => { 0 };
}

// @TODO: Constants can probably use this
#[macro_export]
macro_rules! pick {
    (1 => $me:expr $(=> $__:expr)*                          ) => { $me };
    (2 => $_1:expr => $me:expr $( => $__:expr)*             ) => { $me };
    (3 => $_1:expr => $_2:expr => $me:expr $( => $__:expr )*) => { $me };
    (4 => $_1:expr => $_2:expr => $_3:expr => $me:expr      ) => { $me };
}

#[macro_export]
macro_rules! array_index_by_enum {
    ($ROW_COUNT:ident : usize
    pub enum $Enum:ident {
        $( $Variant:ident $( => $val:expr )* , )*
    } $( $rest:tt )*) => {
        #[derive(Debug)]
        #[repr(usize)]
        pub enum $Enum {
            $( $Variant, )*
        }
        impl $Enum {
            #[allow(dead_code)]
            pub const fn id(&self) -> usize {
                unsafe { *(self as *const Self as *const usize) }
            }
        }

        const $ROW_COUNT: usize = 0 $( + { let _ = $Enum::$Variant; 1 } )*;
        array_index_by_enum!($( $( => $val)*, )* = $ROW_COUNT $($rest)*);
    };

    ($( $(=> $val:expr)*, )* = $len:ident => $n:tt pub const $VEC:ident : [$ty:ty]
        $( $rest:tt )*
    ) => {
        pub const $VEC: [$ty; $len] = [$( $crate::pick!($n $(=> $val )*), )*];
        array_index_by_enum!($( $(=> $val)*, )* = $len $( $rest )*);
    };

    ($( $_:tt)*) => {}; // End tt-muncher
}
