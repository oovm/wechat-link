use std::convert::Infallible;
use std::ops::FromResidual;
use crate::ApiResult;

impl<T, E1, E2> FromResidual<std::result::Result<Infallible, E1>> for ApiResult<T, E2>
where
    E2: From<E1>,
{
    fn from_residual(residual: Result<Infallible, E1>) -> Self {
        match residual {
            Ok(_) => { unreachable!() }
            Err(e) => {
                ApiResult::Failure(e.into())
            }
        }
    }
}

