use poem_openapi::payload::Json;
use crate::npu::ApiResponse;

#[macro_export]
macro_rules! unwrap_err {
    ($result:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e.to_string());
                let mut response = ApiResponse::default();
                response.with_code(500).with_msg(e.to_string());
                return Ok(Json(response));
            }
        }
    };
}

