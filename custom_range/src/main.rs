#![feature(inclusive_range_syntax)]
#![feature(step_trait)]

use std::iter::Step;
use std::mem;
use std::ops::{Add, Sub};
use std::option::Option::{Some, None};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NewType(u32);

impl NewType {
    fn checked_add(self, other: NewType) -> Option<NewType> {
        match self.0.checked_add(other.0) {
            Some(i) => Some(NewType(i)),
            None => None
        }
    }
}

impl Add for NewType {
    type Output = NewType;
    fn add(self, other: NewType) -> NewType {
        NewType(self.0 + other.0)
    }
}

impl<'a> Add for &'a NewType {
    type Output = NewType;
    fn add(self, other: &'a NewType) -> NewType {
        NewType(self.0 + other.0)
    }
}

impl Sub for NewType {
    type Output = NewType;
    fn sub(self, other: NewType) -> NewType {
        NewType(self.0 - other.0)
    }
}

impl Step for NewType {
    fn steps_between(start: &NewType, end: &NewType) -> Option<usize> {
        if start.0 < end.0 {
            Some((end.0 - start.0) as usize)
        } else {
            Some(0)
        }
    }

    fn add_usize(&self, n: usize) -> Option<Self> {
        self.checked_add(NewType(n as u32))
    }

    fn replace_one(&mut self) -> Self {
        mem::replace(self, NewType(1))
    }

    fn replace_zero(&mut self) -> Self {
        mem::replace(self, NewType(0))
    }

    fn add_one(&self) -> Self {
        Add::add(*self, NewType(1))
    }

    fn sub_one(&self) -> Self {
        Sub::sub(*self, NewType(1))
    }
}

fn main() {       
    let a = NewType(10);
    let b = NewType(20);

    for i in a...b {
        println!("{:?}", i);
    }
}
