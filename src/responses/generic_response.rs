use core::default::Default;
use rocket::response::Responder;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses as R;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::okapi::schemars::{self, Map};
use rocket_okapi::response::OpenApiResponderInner;

#[derive(Responder, JsonSchema)]
pub enum Responses<T, E> {
    #[response(status = 201, content_type = "json")]
    Created(T),
    #[response(status = 200, content_type = "json")]
    Sucess(T),
    #[response(status = 400, content_type = "json")]
    BadRequest(E),
}

impl<T, E> OpenApiResponderInner for Responses<T, E> {
    fn responses(
        _gen: &mut OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};
        let mut responses = Map::new();
        responses.insert(
            "400".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
                # [400 Bad Request](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400)\n\
                The request given is wrongly formatted or data asked could not be fulfilled. \
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                This response is given when you request a page that does not exists.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
                # [422 Unprocessable Entity](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/422)\n\
                This response is given when you request body is not correctly formatted. \
                ".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
                # [500 Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)\n\
                This response is given when something wend wrong on the server. \
                ".to_string(),
                ..Default::default()
            }),
        );
        Ok(R {
            responses,
            ..Default::default()
        })
    }
}
