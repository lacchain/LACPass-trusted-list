use rocket::fairing::AdHoc;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};

use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, HideShowConfig, RapiDocConfig};
use rocket_okapi::settings::{OpenApiSettings, UrlObject};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::{mount_endpoints_and_merged_docs, openapi_get_routes_spec};
use sea_orm_rocket::Database;

use crate::config::env_config::Config;
use crate::databases::pool::Db;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |_rocket_instance| async {
        let figment = Config::figment();
        let mut building_rocket = rocket::custom(figment)
            .attach(Db::init())
            .mount(
                "/swagger-ui/",
                make_swagger_ui(&SwaggerUIConfig {
                    url: "../api/v1/openapi.json".to_owned(),
                    ..Default::default()
                }),
            )
            .mount(
                "/rapidoc/",
                make_rapidoc(&RapiDocConfig {
                    title: Some("LACPass-Verifier RapiDoc documentation | RapiDoc".to_owned()),
                    general: GeneralConfig {
                        spec_urls: vec![UrlObject::new("General", "../api/v1/openapi.json")],
                        ..Default::default()
                    },
                    hide_show: HideShowConfig {
                        allow_spec_url_load: false,
                        allow_spec_file_load: false,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            )
            .attach(cors());
        let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
        // let custom_route_spec = (vec![], custom_openapi_spec());
        mount_endpoints_and_merged_docs! {
        building_rocket, "/api/v1".to_owned(), openapi_settings,
            // "" => custom_route_spec,
            "/certificates" => get_routes_and_docs(&openapi_settings)
        };
        building_rocket
    })
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: crate::controllers::certificate_controller::verify_certificate
    ]
}

fn cors() -> Cors {
    let allowed_origins = AllowedOrigins::All;

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();
    cors
}
