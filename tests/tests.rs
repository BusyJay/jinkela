extern crate prost;

#[derive(::jinkela::Classicalize, Default)]
struct A {
    #[prost(message)]
    f: Option<B>,
}

#[derive(::jinkela::Classicalize, Default)]
struct B {
}