use std::str::FromStr;

use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol, ALPN as BLOBS_ALPN};
use iroh_docs::{protocol::Docs, DocTicket, ALPN as DOCS_ALPN};
use iroh_gossip::{Gossip, ALPN as GOSSIP_ALPN};
use sea_orm::{ActiveHasMany, ActiveValue::Set, DatabaseConnection};

use crate::{
    access_list::list_manager::AccessListManager,
    entities::{address, topic},
    iroh::iroh_mem_instance::IrohMemInstance,
    protocol::{access_control::AccessControl, video_discovery::VideoDiscovery},
    store::storage_manager::StorageManager,
    ALPN, DISCOVERY_ALPN,
};

pub struct IrohRuntime {
    _router: Router,
    access_control: AccessControl,
    db: DatabaseConnection,
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
        })
    }

    pub async fn import_ticket(&self, ticket: String) -> anyhow::Result<()> {
        let doc_ticket = DocTicket::from_str(&ticket)?;
        self.access_control.import(doc_ticket).await?;

        Ok(())
    }

    pub async fn add_remote_store(&self, endpoint: String, topic: String) -> anyhow::Result<()> {
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

    pub async fn get_videos(&self, topic: String) {
        if let Ok(topic) = topic::Entity::find_by_topic(topic)
            .find_with_related(address::Entity)
            .all(&self.db)
            .await
        {}

        todo!()
    }
}
