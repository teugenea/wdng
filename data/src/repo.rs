use std::{future::{Future, self}, pin::Pin};

use crate::model::*;

pub trait LangUnitRepo {
    fn get_next(&self);
}