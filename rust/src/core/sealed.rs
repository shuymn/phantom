/// Sealed trait pattern for API stability
/// This module provides sealed traits to prevent downstream implementations
/// while maintaining API flexibility for the library

/// Private module to hide the sealed trait
mod private {
    pub trait Sealed {}
}

/// Re-export the Sealed trait (but not its implementations)
pub(crate) use private::Sealed;

/// Macro to easily implement sealed traits
#[macro_export]
macro_rules! sealed_trait {
    (
        $(#[$meta:meta])*
        pub trait $trait_name:ident: $($bound:tt)+ {
            $($trait_body:tt)*
        }
    ) => {
        $(#[$meta])*
        pub trait $trait_name: $crate::core::sealed::Sealed + $($bound)+ {
            $($trait_body)*
        }
    };
    
    (
        $(#[$meta:meta])*
        pub trait $trait_name:ident {
            $($trait_body:tt)*
        }
    ) => {
        $(#[$meta])*
        pub trait $trait_name: $crate::core::sealed::Sealed {
            $($trait_body)*
        }
    };
}

/// Example of how to use sealed traits
#[cfg(test)]
mod example {
    use super::private::Sealed;
    
    // Define a sealed trait
    #[allow(dead_code)]
    pub trait SealedExample: Sealed {
        fn method(&self) -> &str;
    }
    
    // Internal type that implements the sealed trait
    #[allow(dead_code)]
    pub struct InternalType;
    
    // Implement the private Sealed trait
    impl Sealed for InternalType {}
    
    // Implement the public trait
    impl SealedExample for InternalType {
        fn method(&self) -> &str {
            "internal"
        }
    }
    
    // External code cannot implement SealedExample because they
    // cannot implement the private Sealed trait
}