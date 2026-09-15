use std::str::FromStr;

use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol, ALPN as BLOBS_ALPN};
use iroh_docs::{protocol::Docs, DocTicket, ALPN as DOCS_ALPN};
use iroh_gossip::{Gossip, ALPN as GOSSIP_ALPN};

use crate::{
    access_list::list_manager::AccessListManager,
    iroh::iroh_mem_instance::IrohMemInstance,
    protocol::{access_control::AccessControl, video_discovery::VideoDiscovery},
    store::storage_manager::StorageManager,
    ALPN, DISCOVERY_ALPN,
};

pub struct IrohRuntime {
    router: Router,
    access_control: AccessControl,
}

impl IrohRuntime {
    pub async fn new() -> anyhow::Result<Self> {
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

        let router = Router::builder(endpoint)
            .accept(DOCS_ALPN, docs)
            .accept(GOSSIP_ALPN, gossip)
            .accept(BLOBS_ALPN, BlobsProtocol::new(&access_list_blobs, None))
            .accept(ALPN, access_control.clone())
            .accept(DISCOVERY_ALPN, video_discovery)
            .spawn();

        Ok(Self {
            router,
            access_control,
        })
    }

    pub async fn import_ticket(&self, ticket: String) -> anyhow::Result<()> {
        let doc_ticket = DocTicket::from_str(&ticket)?;
        self.access_control.import(doc_ticket).await?;

        Ok(())
    }
}
