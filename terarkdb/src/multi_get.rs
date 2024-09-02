use std::{
    ffi::c_char,
    iter::{FusedIterator, IntoIterator, Zip},
    vec,
};

use crate::{
    error::Error,
    util::{Malloced, MallocedBytes},
};

#[derive(Debug)]
pub struct MultiGet {
    pub(crate) errors: Vec<Option<Error>>,
    pub(crate) values: Vec<Option<Malloced<c_char>>>,
    pub(crate) lens: Vec<usize>,
}

impl MultiGet {
    pub fn new(num_values: usize) -> MultiGet {
        let mut errors = Vec::new();
        errors.resize_with(num_values, || None);

        let mut values = Vec::new();
        values.resize_with(num_values, || None);

        MultiGet {
            errors,
            values,
            lens: vec![0; num_values],
        }
    }
}

impl IntoIterator for MultiGet {
    type IntoIter = MultiGetIntoIter;
    type Item = Result<Option<MallocedBytes>, Error>;

    fn into_iter(self) -> MultiGetIntoIter {
        MultiGetIntoIter {
            raw: self
                .errors
                .into_iter()
                .zip(self.values.into_iter().zip(self.lens.into_iter())),
        }
    }
}

pub struct MultiGetIntoIter {
    raw: Zip<
        vec::IntoIter<Option<Error>>,
        Zip<vec::IntoIter<Option<Malloced<c_char>>>, vec::IntoIter<usize>>,
    >,
}

impl Iterator for MultiGetIntoIter {
    type Item = Result<Option<MallocedBytes>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw
            .next()
            .map(|(error, (maybe_value, len))| match (error, maybe_value) {
                (Some(error), _) => Err(error),
                (_, Some(value)) => Ok(Some(unsafe { MallocedBytes::from_parts(value, len) })),
                (_, None) => Ok(None),
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}

impl ExactSizeIterator for MultiGetIntoIter {
    fn len(&self) -> usize {
        self.raw.len()
    }
}

impl FusedIterator for MultiGetIntoIter {}
