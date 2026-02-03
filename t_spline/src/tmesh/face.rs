use crate::tmesh::ids::EdgeID;

#[derive(Debug, Clone)]
pub struct Face {
    pub edge: EdgeID,
}
