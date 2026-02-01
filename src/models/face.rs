use crate::models::ids::{EdgeID, FaceID};

#[derive(Debug)]
#[derive(Clone)]
pub struct Face {
    pub edge: EdgeID,
}
