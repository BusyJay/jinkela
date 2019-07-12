#[cfg(feature = "repr-prost")]
extern crate prost;

#[cfg(feature = "repr-prost")]
mod prost_tests {
    use jinkela::GenericEnum;

    #[derive(::jinkela::Classicalize, Default, Debug)]
    struct A {
        #[prost(message, optional)]
        b1: Option<B>,
        #[prost(message, optional)]
        b2: ::std::option::Option<B>,
        #[prost(message, repeated)]
        b3: Vec<B>,
        #[prost(enumeration = "E")]
        e: i32,
    }

    #[derive(::jinkela::Classicalize, Default, Debug, PartialEq)]
    struct B {
        #[prost(uint64)]
        b: u64,
    }

    #[derive(::jinkela::Classicalize, Debug, PartialEq)]
    #[repr(i32)]
    enum E {
        T = 0,
        C = 1,
    }

    impl Default for E {
        fn default() -> E {
            E::T
        }
    }

    impl E {
        pub fn from_i32(i: i32) -> Option<E> {
            match i {
                0 => Some(E::T),
                1 => Some(E::C),
                _ => None
            }
        }
    }

    #[test]
    fn test_methods() {
        let mut a = A::default();
        assert_eq!(a.get_b1().b, 0);
        assert_eq!(a.get_b2().b, 0);
        a.mut_b1().b = 2;
        a.mut_b2().b = 1;
        assert_eq!(a.get_b1().b, 2);
        assert_eq!(a.get_b2().b, 1);
        let b = B::default();
        assert_eq!(*B::default_instance(), b);
        a.set_b2(b);
        assert_eq!(a.get_b2().b, 0);
        assert_eq!(a.get_e(), E::T);
        a.e = E::C as i32;
        assert_eq!(a.get_e(), E::C);
    }

    #[test]
    fn test_enum() {
        assert_eq!(E::values(), &[E::T, E::C]);
    }
}