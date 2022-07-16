use crate::model::*;

pub trait UnitRepo {
    fn get_next(&self) -> Unit;
}