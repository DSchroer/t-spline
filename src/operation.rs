use crate::models::TMesh;

/// Edit operation to perform on a spline mesh
pub trait SplineOp {
    /// Error type of the operation
    type Error;

    /// Perform the operation, returning any error as needed
    fn perform(&mut self, mesh: &mut TMesh) -> Result<(), Self::Error>;
}

/// Result mapper to handle multiple clojure return types
pub trait IntoOpResult {
    /// Error type of the operation
    type Error;

    /// Map value into a [SplineOp] result
    fn into_result(self) -> Result<(), Self::Error>;
}

impl IntoOpResult for () {
    type Error = ();
    fn into_result(self) -> Result<(), ()> { Ok(()) }
}

impl<R> IntoOpResult for Result<(), R> {
    type Error = R;
    fn into_result(self) -> Result<(), R> { self }
}

impl<T, Out> SplineOp for T
    where T: FnMut(&mut TMesh) -> Out,
    Out: IntoOpResult
{
    type Error = Out::Error;

    fn perform(&mut self, mesh: &mut TMesh) -> Result<(), Self::Error> {
        self(mesh).into_result()
    }
}

