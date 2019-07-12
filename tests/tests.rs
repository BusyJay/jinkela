#[cfg(feature = "repr-prost")]
extern crate prost;

#[cfg(feature = "repr-prost")]
mod prost_tests {
    use jinkela::GenericEnum;

    #[derive(::jinkela::Classicalize, Default, Debug)]
    struct A {
        #[prost(message)]
        b1: Option<B>,
        #[prost(message)]
        b2: ::std::option::Option<B>,
    }

    #[derive(::jinkela::Classicalize, Default, Debug, PartialEq)]
    struct B {
        #[prost(uint64)]
        b: u64,
    }

    #[derive(::jinkela::Classicalize, Debug, PartialEq)]
    enum E {
        T = 0,
        C = 1,
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
    }

    #[test]
    fn test_enum() {
        assert_eq!(E::values(), &[E::T, E::C]);
    }
}