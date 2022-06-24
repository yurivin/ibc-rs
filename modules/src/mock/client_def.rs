use ibc_proto::ibc::core::commitment::v1::MerkleProof;

use crate::core::ics02_client::client_consensus::ConsensusState;
use crate::core::ics02_client::client_def::ClientDef;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::context::LightClientReader;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::header::Header;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::context::ChannelMetaReader;
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics23_commitment::merkle::apply_prefix;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::ics24_host::Path;
use crate::mock::client_state::{MockClientState, MockConsensusState};
use crate::mock::header::MockHeader;
use crate::prelude::*;
use crate::Height;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockClient;

impl ClientDef for MockClient {
    type ClientState = MockClientState;
    type ConsensusState = MockConsensusState;

    fn check_header_and_update_state(
        &self,
        _ctx: &dyn LightClientReader,
        _client_id: ClientId,
        client_state: Self::ClientState,
        header: &dyn Header,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Error> {
        if client_state.latest_height() >= header.height() {
            return Err(Error::low_header_height(
                header.height(),
                client_state.latest_height(),
            ));
        }

        let header = *header
            .as_any()
            .downcast_ref::<MockHeader>()
            .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;
        Ok((
            MockClientState::new(header),
            MockConsensusState::new(header),
        ))
    }

    fn verify_client_consensus_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        _expected_consensus_state: &dyn ConsensusState,
    ) -> Result<(), Error> {
        let client_prefixed_path = Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: consensus_height.revision_number,
            height: consensus_height.revision_height,
        })
        .to_string();

        let _path = apply_prefix(prefix, vec![client_prefixed_path]);

        Ok(())
    }

    fn verify_connection_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _connection_id: &ConnectionId,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_channel_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_client_full_state<U>(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _expected_client_state: &dyn ClientState<UpgradeOptions = U>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_data(
        &self,
        _ctx: &dyn ChannelMetaReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _commitment: PacketCommitment,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_acknowledgement(
        &self,
        _ctx: &dyn ChannelMetaReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _ack: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_next_sequence_recv(
        &self,
        _ctx: &dyn ChannelMetaReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_receipt_absence(
        &self,
        _ctx: &dyn ChannelMetaReader,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_upgrade_and_update_state(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        _proof_upgrade_client: MerkleProof,
        _proof_upgrade_consensus_state: MerkleProof,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Error> {
        Ok((*client_state, consensus_state.clone()))
    }
}
