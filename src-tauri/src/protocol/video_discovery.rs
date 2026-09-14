use iroh::protocol::{AcceptError, ProtocolHandler};
use tokio::io::AsyncReadExt;

use crate::access_list::list_manager::AccessListManager;

#[derive(Debug)]
pub struct VideoDiscovery {
    list_manager: AccessListManager,
}

impl ProtocolHandler for VideoDiscovery {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        let (mut send, mut recv) = connection
            .accept_bi()
            .await
            .map_err(AcceptError::from_err)?;

        let len = recv.read_u32().await?;

        let mut request_bytes: Vec<u8> = vec![0u8; len as usize];
        recv.read_exact(&mut request_bytes)
            .await
            .map_err(AcceptError::from_err)?;

        let namespace = String::from_utf8(request_bytes).map_err(AcceptError::from_err)?;

        todo!()
    }
}
