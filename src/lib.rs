#![feature(generators, generator_trait)]
#![feature(or_patterns)]
#![feature(never_type)]
#![feature(trait_alias)]

#[macro_use]
extern crate bitflags;

pub mod bus;
pub mod cpu;
pub mod devs;
pub mod tools;
