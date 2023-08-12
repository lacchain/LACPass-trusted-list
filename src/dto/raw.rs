use rocket::data::{self, Data, FromData, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::FromForm;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::MediaType;
use rocket_okapi::okapi::openapi3::RequestBody;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::okapi::schemars::Map;
use rocket_okapi::request::{OpenApiFromData, OpenApiFromRequest, RequestHeaderInput};

use crate::dto::utils::fn_request_body;

#[derive(FromForm, JsonSchema)]
pub struct RawData<'r>(pub &'r str);

#[rocket::async_trait]
impl<'r> FromData<'r> for RawData<'r> {
    type Error = Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        use rocket::outcome::Outcome::*;
        use Error::*;

        // Ensure the content type is correct before opening the data.
        let person_ct = ContentType::new("text", "plain");
        if req.content_type() != Some(&person_ct) {
            return Forward(data);
        }

        // Use a configured limit with name 'rawData' or fallback to default.
        let limit = req.limits().get("rawData").unwrap_or(5.mebibytes());

        // Read the data into a string.
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Failure((Status::PayloadTooLarge, TooLarge)),
            Err(e) => return Failure((Status::InternalServerError, Io(e))),
        };

        // We store `string` in request-local cache for long-lived borrows.
        let string = request::local_cache!(req, string);

        Success(RawData { 0: string })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawData<'r> {
    type Error = ();

    async fn from_request(_request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        todo!()
    }
}

impl<'r> OpenApiFromRequest<'r> for RawData<'r> {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

impl<'r> OpenApiFromData<'r> for RawData<'r> {
    fn request_body(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<RequestBody> {
        fn_request_body!(gen, RawData, "text/plain") // establishes type for this
    }
}

#[derive(Debug)]
pub enum Error {
    TooLarge,
    NoColon,
    Io(std::io::Error),
}
