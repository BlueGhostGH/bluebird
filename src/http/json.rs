pub(in crate::http) mod extractor
{
    use crate::http::api_error;

    use axum::{extract::rejection, http, response};

    use axum_macros::FromRequest;
    use serde_json::json;

    #[derive(FromRequest)]
    #[from_request(via(axum::Json), rejection(Error))]
    pub(in crate::http) struct Json<T>(pub(in crate::http) T);

    #[derive(Debug)]
    pub(in crate::http) struct Error
    {
        api_code: api_error::Code,
        http_code: http::StatusCode,
        message: String,
    }

    impl From<rejection::JsonRejection> for Error
    {
        fn from(rejection: rejection::JsonRejection) -> Self
        {
            let (api_code, http_code) = match rejection {
                rejection::JsonRejection::JsonSyntaxError(_) => (
                    api_error::Code::JSON_SYNTAX_ERROR,
                    http::StatusCode::BAD_REQUEST,
                ),
                rejection::JsonRejection::JsonDataError(_) => (
                    api_error::Code::JSON_DATA_ERROR,
                    http::StatusCode::UNPROCESSABLE_ENTITY,
                ),
                rejection::JsonRejection::MissingJsonContentType(_) => (
                    api_error::Code::JSON_MISSING_CONTENT_TYPE,
                    http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                ),
                _ => (
                    api_error::Code::JSON_UNKNOWN_ERROR,
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ),
            };

            let message = rejection.to_string();

            Error {
                api_code,
                http_code,
                message,
            }
        }
    }

    impl response::IntoResponse for Error
    {
        fn into_response(self) -> response::Response
        {
            let payload = json!({
                "message": self.message,
                "code": self.api_code
            });

            (self.http_code, axum::Json(payload)).into_response()
        }
    }
}
