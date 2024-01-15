use serde_json::value::RawValue;

use crate::json_rpc::{error::ErrorPayload, id::Id};

/// A JSONRPC-2.0 response object containing a [`ResponsePayload`].
///
/// This object is used to represent a JSONRPC-2.0 response. It may contain
/// either a successful result or an error. The `id` field is used to match
/// the response to the request that it is responding to, and should be
/// mirrored from the response.
#[derive(Debug, Clone)]
pub struct Response<Payload = Box<RawValue>, ErrData = Box<RawValue>> {
    /// The ID of the request that this response is responding to.
    pub id: Id,
    /// The response payload.
    pub payload: ResponsePayload<Payload, ErrData>,
}

#[derive(Debug, Clone)]
pub enum ResponsePayload<Payload = Box<RawValue>, ErrData = Box<RawValue>> {
    /// A successful response payload
    Success(Payload),
    /// An error response payload
    Failure(ErrorPayload<ErrData>),
}

impl<Payload, ErrData> ResponsePayload<Payload, ErrData> {
    /// Returns `true` if the response payload is a success.
    pub const fn is_success(&self) -> bool {
        matches!(self, ResponsePayload::Success(_))
    }

    /// Returns `true` if the response payload is an error.
    pub const fn is_error(&self) -> bool {
        matches!(self, ResponsePayload::Failure(_))
    }
}
