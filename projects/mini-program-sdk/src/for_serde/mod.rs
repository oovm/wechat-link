use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::{ApiError, ApiResult};

impl<T, E: ApiError> Serialize for ApiResult<T, E>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = Serializer::serialize_struct(serializer, "ApiResult", 8)?;
        match self {
            ApiResult::Success(value) => {
                ser.serialize_field("code", &0)?;
                ser.serialize_field("data", value)?;
            }
            ApiResult::Failure(error) => {
                ser.serialize_field("code", &error.error_code())?;
                ser.serialize_field("message", &error.error_message())?;
            }
        }
        ser.end()
    }
}


