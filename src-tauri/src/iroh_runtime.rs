use std::str::FromStr;

use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol, ALPN as BLOBS_ALPN};
use iroh_docs::{protocol::Docs, DocTicket, ALPN as DOCS_ALPN};
use iroh_gossip::{Gossip, ALPN as GOSSIP_ALPN};
use sea_orm::{ActiveHasMany, ActiveValue::Set, DatabaseConnection};

use crate::{
    access_list::list_manager::AccessListManager,
    discovery::discovery_service::DiscoveryService,
    entities::{address, topic},
    iroh::iroh_mem_instance::IrohMemInstance,
    protocol::{access_control::AccessControl, video_discovery::VideoDiscovery},
    store::storage_manager::StorageManager,
    Error, ALPN, DISCOVERY_ALPN,
};

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
        let video_discovery = VideoDiscovery::new(list_manager.clone());

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
            .accept(DISCOVERY_ALPN, video_discovery)
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

    pub async fn start_adding_topic_peers(&self, topic: String) -> Result<(), Error> {
        self.discovery.cancel_topic(&topic);
        Ok(self.discovery.emit_topic(&topic, &self.db, false).await?)
    }

    pub fn stop_adding_topic_peers(&self, topic: String) -> bool {
        self.discovery.cancel_topic(&topic)
    }

    pub async fn get_authorized_videos(&self, topic: String) -> Result<Option<Vec<String>>, Error> {
        let topic: Vec<(topic::Model, Vec<address::Model>)> = topic::Entity::find_by_topic(topic)
            .find_with_related(address::Entity)
            .all(&self.db)
            .await?;

        let (namespace, addresses) = &topic[0];

        for address in addresses {
            let endpoint = EndpointId::from_str(&address.endpoint)
                .map_err(|e| Error::InputErr(e.to_string()))?;

            if let Some(videos) = self
                .access_control
                .get_authorized_videos(&namespace.topic, &endpoint)
                .await
                .map_err(|e| Error::IrohErr(e.to_string()))?
            {
                return Ok(Some(videos));
            }
        }

        Ok(None)
    }
}
