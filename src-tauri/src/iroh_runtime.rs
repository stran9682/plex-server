use std::collections::HashMap;
use std::sync::Arc;
use std::{path::PathBuf, str::FromStr};

use ffmpeg_sidecar::command::{ffmpeg_is_installed, FfmpegCommand};
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol, ALPN as BLOBS_ALPN};
use iroh_docs::{protocol::Docs, DocTicket, ALPN as DOCS_ALPN};
use iroh_gossip::{Gossip, ALPN as GOSSIP_ALPN};
use sea_orm::EntityTrait;
use sea_orm::{ActiveHasMany, ActiveValue::Set, DatabaseConnection};
use serde::Deserialize;
use tempfile::TempDir;
use tokio::fs::File;

use crate::Error::IrohErr;
use crate::VideoInfo;
use crate::{
    access_list::list_manager::AccessListManager,
    discovery::discovery_service::DiscoveryService,
    entities::{address, topic},
    iroh::iroh_mem_instance::IrohMemInstance,
    protocol::access_control::{AccessControl, Request},
    store::storage_manager::StorageManager,
    Error, ALPN, DISCOVERY_ALPN,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio_util::io::ReaderStream;

pub struct IrohRuntime {
    _router: Router,
    access_control: AccessControl,
    db: DatabaseConnection,
    discovery: DiscoveryService,
}

impl IrohRuntime {
    pub async fn new(db: DatabaseConnection) -> anyhow::Result<Self> {
        let endpoint = Endpoint::bind(presets::N0).await?;
        let access_list_blobs = MemStore::new();
        let gossip = Gossip::builder().spawn(endpoint.clone());

        let docs = Docs::memory()
            .spawn(
                endpoint.clone(),
                (*access_list_blobs).clone(),
                gossip.clone(),
            )
            .await?;

        let acl_iroh_instance =
            IrohMemInstance::new(access_list_blobs.clone(), docs.clone(), endpoint.clone());

        let list_manager = AccessListManager::new(acl_iroh_instance);

        let storage_blobs = MemStore::new();
        let storage_iroh_instance =
            IrohMemInstance::new(storage_blobs, docs.clone(), endpoint.clone());
        let storage_manager = StorageManager::new(storage_iroh_instance);

        let access_control =
            AccessControl::new(list_manager.clone(), storage_manager, endpoint.id());

        println!("Endpoint: {}", endpoint.id());

        let discovery_service = DiscoveryService::new(endpoint.clone(), gossip.clone());

        let _router = Router::builder(endpoint)
            .accept(DOCS_ALPN, docs)
            .accept(GOSSIP_ALPN, gossip)
            .accept(BLOBS_ALPN, BlobsProtocol::new(&access_list_blobs, None))
            .accept(ALPN, access_control.clone())
            .accept(DISCOVERY_ALPN, access_control.clone())
            .spawn();

        Ok(Self {
            _router,
            access_control,
            db,
            discovery: discovery_service,
        })
    }

    pub async fn import_ticket(&self, ticket: String) -> Result<(), Error> {
        let doc_ticket =
            DocTicket::from_str(&ticket).map_err(|e| Error::InputErr(e.to_string()))?;
        self.access_control
            .import(doc_ticket)
            .await
            .map_err(|e| Error::IrohErr(e.to_string()))?;

        Ok(())
    }

    pub async fn add_dir(
        &self,
        file_path: PathBuf,
        namespace: Option<String>,
    ) -> Result<(), Error> {
        if !ffmpeg_is_installed() {
            return Err(Error::InputErr("ffmpeg was not installed".to_string()));
        }

        let Some(filename) = file_path.file_name() else {
            return Err(Error::InputErr("Filename was invalid".to_owned()));
        };

        let temp_dir = TempDir::new()
            .map_err(|e| Error::InputErr(format!("Failed to create temp dir {e}")))?;

        let args = format!(
            "-codec: copy -start_number 0 -hls_time 10 -hls_list_size 0 -f hls {}/output.m3u8",
            temp_dir.path().to_string_lossy()
        );
        let mut command = FfmpegCommand::new()
            .input("")
            .args(args.split(' '))
            .spawn()
            .unwrap();

        command.iter().unwrap();

        self.access_control
            .upload_new(
                &temp_dir.path().to_string_lossy(),
                &filename.to_string_lossy(),
                namespace.as_deref(),
            )
            .await
            .map_err(|e| IrohErr(e.to_string()))?;

        Ok(())
    }

    pub async fn add_remote_store(&self, endpoint: String, topic: String) -> Result<(), Error> {
        topic::ActiveModelEx {
            topic: Set(topic),
            addresses: ActiveHasMany::Append(vec![address::ActiveModelEx {
                endpoint: Set(endpoint),
                ..Default::default()
            }]),
            ..Default::default()
        }
        .save(&self.db)
        .await?;

        Ok(())
    }

    pub async fn download_file(
        &self,
        namespace: &str,
        resource: &str,
        filename: &str,
    ) -> anyhow::Result<Option<File>> {
        let request = Request::new(
            String::from(namespace),
            String::from(resource),
            String::from(filename),
        );

        let bootstraps = self.get_peer(namespace).await?;

        for endpoint_id in bootstraps {
            if let Ok(Some(file)) = self
                .access_control
                .make_request(Some(endpoint_id), &request)
                .await
            {
                return Ok(Some(file));
            }
        }

        Ok(None)
    }

    pub async fn start_adding_topic_peers(&self, topic: String) -> Result<(), Error> {
        self.discovery.cancel_topic(&topic);
        Ok(self.discovery.emit_topic(&topic, &self.db, false).await?)
    }

    pub fn stop_adding_topic_peers(&self, topic: String) -> bool {
        self.discovery.cancel_topic(&topic)
    }

    pub async fn request_authorized_videos(
        &self,
    ) -> Result<HashMap<String, Vec<VideoInfo>>, Error> {
        let topic: Vec<(topic::Model, Vec<address::Model>)> = topic::Entity::find()
            .find_with_related(address::Entity)
            .all(&self.db)
            .await?;

        let mut namespace_videos: HashMap<String, Vec<VideoInfo>> = HashMap::new();
        for (namespace, addresses) in topic {
            for address in addresses {
                let endpoint = EndpointId::from_str(&address.endpoint)
                    .map_err(|e| Error::InputErr(e.to_string()))?;

                if let Ok(Some(videos)) = self
                    .access_control
                    .request_authorized_videos(&namespace.topic, &endpoint)
                    .await
                {
                    namespace_videos.insert(namespace.topic, videos);
                    break;
                } else {
                    continue;
                }
            }
        }

        Ok(namespace_videos)
    }

    async fn get_peer(&self, topic: &str) -> Result<Vec<EndpointId>, Error> {
        let topic: Vec<(topic::Model, Vec<address::Model>)> = topic::Entity::find_by_topic(topic)
            .find_with_related(address::Entity)
            .all(&self.db)
            .await?;

        let (_, addresses) = &topic[0];

        let bootstrap: Vec<EndpointId> = addresses
            .iter()
            .filter_map(|i| EndpointId::from_str(&i.endpoint).ok())
            .collect();

        Ok(bootstrap)
    }
}

#[derive(Deserialize)]
pub struct RequestArgs {
    namespace: String,
    resource: String,
    filename: String,
}

pub async fn download_handler(
    Query(request_args): Query<RequestArgs>,
    State(access_control_service): State<Arc<IrohRuntime>>,
) -> impl IntoResponse {
    let file = match access_control_service
        .download_file(
            &request_args.namespace,
            &request_args.resource,
            &request_args.filename,
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
