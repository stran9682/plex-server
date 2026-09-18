use std::{str::FromStr, sync::Arc};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use iroh::EndpointId;
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::protocol::access_control::{AccessControl, Request};

#[derive(Deserialize)]
pub struct RequestArgs {
    namespace: String,
    resource: String,
    filename: String,
    endpoint_id: Option<String>,
}

pub struct AccessControlService {
    access_control: AccessControl,
}

impl AccessControlService {
    pub fn new(access_control: AccessControl) -> Self {
        Self { access_control }
    }

    pub async fn download_file(
        &self,
        namespace: &str,
        resource: &str,
        filename: &str,
        endpoint_id: Option<EndpointId>,
    ) -> anyhow::Result<Option<File>> {
        let request = Request::new(
            String::from(namespace),
            String::from(resource),
            String::from(filename),
        );

        let file = self
            .access_control
            .make_request(endpoint_id, &request)
            .await?;

        Ok(file)
    }
}

pub async fn download_handler(
    Query(request_args): Query<RequestArgs>,
    State(access_control_service): State<Arc<AccessControlService>>,
) -> impl IntoResponse {
    let endpoint_id = if let Some(endpoint_id) = request_args.endpoint_id {
        iroh::EndpointId::from_str(&endpoint_id).ok()
    } else {
        None
    };

    let file = match access_control_service
        .download_file(
            &request_args.namespace,
            &request_args.resource,
            &request_args.filename,
            endpoint_id,
        )
        .await
    {
        Ok(Some(file)) => file,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(
                    "Permission error or file hash didn't match resource",
                ))
                .unwrap();
        }
        Err(e) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Network error occured, {e}")))
                .unwrap();
        }
    };

    let content_type = mime_guess::from_path(&request_args.filename).first_or_octet_stream();

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type.as_ref())
        .body(body)
        .unwrap()
}
