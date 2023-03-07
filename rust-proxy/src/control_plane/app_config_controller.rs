use super::responder::ApiError;
use crate::configuration_service::app_config_servive::GLOBAL_APP_CONFIG;
use crate::vojo::app_config::ApiService;
use crate::vojo::app_config::AppConfig;
use crate::vojo::app_config::ServerType;
use crate::vojo::vojo::BaseResponse;
use rocket::route::Route;
use rocket::serde::json::Json;
#[get("/appConfig", format = "json")]
async fn get_app_config() -> Result<Json<BaseResponse<AppConfig>>, ApiError> {
    let app_config = GLOBAL_APP_CONFIG.read().await;
    Ok(Json(BaseResponse {
        response_code: 0,
        response_object: app_config.clone(),
    }))
}

#[post("/appConfig", format = "json", data = "<api_services_json>")]
async fn set_app_config(
    api_services_json: Json<Vec<ApiService>>,
) -> Result<Json<BaseResponse<u32>>, ApiError> {
    let api_services = api_services_json.into_inner().clone();

    let validata_result = api_services
        .iter()
        .filter(|s| s.service_config.server_type == ServerType::HTTPS)
        .map(|s| {
            return validate_tls_config(
                s.service_config.cert_str.clone(),
                s.service_config.key_str.clone(),
            );
        })
        .collect::<Result<Vec<()>, anyhow::Error>>();
    if validata_result.is_err() {
        return Err(ApiError::NotFound(String::from(
            "Parse the key string or the certificate string error!",
        )));
    }

    let mut rw_global_lock = GLOBAL_APP_CONFIG.write().await;
    (*rw_global_lock).api_service_config = api_services.clone();

    Ok(Json(BaseResponse {
        response_code: 0,
        response_object: 0,
    }))
}
fn validate_tls_config(
    cert_pem_option: Option<String>,
    key_pem_option: Option<String>,
) -> Result<(), anyhow::Error> {
    if cert_pem_option.is_none() || key_pem_option.is_none() {
        return Err(anyhow!("cert or key is none"));
    }
    let cert_pem = cert_pem_option.unwrap();
    let mut cer_reader = std::io::BufReader::new(cert_pem.as_bytes());
    let result_certs = rustls_pemfile::certs(&mut cer_reader);
    if result_certs.is_err() || result_certs.unwrap().len() == 0 {
        return Err(anyhow!("can not parse the certs pem."));
    }
    let key_pem = key_pem_option.unwrap();
    let key_pem_result = pkcs8::PrivateKeyDocument::from_pem(key_pem.as_str());
    if key_pem_result.is_err() {
        return Err(anyhow!("can not parse the key pem."));
    }
    Ok(())
}

pub fn get_app_config_controllers() -> Vec<Route> {
    routes![get_app_config, set_app_config]
}
