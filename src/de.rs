use serde;
use serde_json;
use hyper::{Response, Body};

pub trait FromJsonResponse<'de> {
    fn from_json<T: serde::Deserialize<'de>>(self) -> serde_json::Result<Response<T>>;
}

// impl FromJsonResponse<'static> for Response<Vec<u8>> {
//     fn from_json<T:serde::Deserialize<'static>>(self) -> serde_json::Result<Response<T>> {
//         let (parts, body) = self.into_parts();
//         let body = serde_json::from_slice(&body)?;
//         Ok(Response::from_parts(parts, body))
//     }
// }
