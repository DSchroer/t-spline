use crate::models::ids::{EdgeID};

#[derive(Debug)]
#[derive(Clone)]
pub struct Face {
    pub edge: EdgeID,
}
