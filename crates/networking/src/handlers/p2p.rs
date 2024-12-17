#![allow(unused_results)]

use crate::gossip::{MyBehaviourRequest, MyBehaviourResponse, NetworkService};
use crate::key_types::CryptoKeyCurve;
use gadget_crypto::hashing::blake3_256;
use gadget_crypto::KeyType;
use gadget_std::string::ToString;
use libp2p::gossipsub::IdentTopic;
use libp2p::{request_response, PeerId};
use std::sync::atomic::Ordering;

impl NetworkService<'_> {
    #[tracing::instrument(skip(self, event))]
    pub(crate) async fn handle_p2p(
        &mut self,
        event: request_response::Event<MyBehaviourRequest, MyBehaviourResponse>,
    ) {
        use request_response::Event::{InboundFailure, Message, OutboundFailure, ResponseSent};
        match event {
            Message { peer, message } => {
                gadget_logging::trace!("Received P2P message from: {peer}");
                self.handle_p2p_message(peer, message).await;
            }
            OutboundFailure {
                peer,
                request_id,
                error,
            } => {
                gadget_logging::error!("Failed to send message to peer: {peer} with request_id: {request_id} and error: {error}");
            }
            InboundFailure {
                peer,
                request_id,
                error,
            } => {
                gadget_logging::error!("Failed to receive message from peer: {peer} with request_id: {request_id} and error: {error}");
            }
            ResponseSent { peer, request_id } => {
                gadget_logging::debug!(
                    "Sent response to peer: {peer} with request_id: {request_id}"
                );
            }
        }
    }

    #[tracing::instrument(skip(self, message))]
    async fn handle_p2p_message(
        &mut self,
        peer: PeerId,
        message: request_response::Message<MyBehaviourRequest, MyBehaviourResponse>,
    ) {
        use request_response::Message::{Request, Response};
        match message {
            Request {
                request,
                channel,
                request_id,
            } => {
                gadget_logging::trace!(
                    "Received request with request_id: {request_id} from peer: {peer}"
                );
                self.handle_p2p_request(peer, request_id, request, channel)
                    .await;
            }
            Response {
                response,
                request_id,
            } => {
                gadget_logging::trace!(
                    "Received response from peer: {peer} with request_id: {request_id}"
                );
                self.handle_p2p_response(peer, request_id, response).await;
            }
        }
    }

    #[tracing::instrument(skip(self, req, channel))]
    async fn handle_p2p_request(
        &mut self,
        peer: PeerId,
        request_id: request_response::InboundRequestId,
        req: MyBehaviourRequest,
        channel: request_response::ResponseChannel<MyBehaviourResponse>,
    ) {
        use crate::gossip::MyBehaviourRequest::{Handshake, Message};
        let result = match req {
            Handshake {
                crypto_public_key,
                signature,
            } => {
                gadget_logging::trace!("Received handshake from peer: {peer}");
                // Verify the signature
                let msg = peer.to_bytes();
                let hash = blake3_256(&msg);
                let valid =
                    <CryptoKeyCurve as KeyType>::verify(&crypto_public_key, &hash, &signature);
                if !valid {
                    gadget_logging::warn!("Invalid signature from peer: {peer}");
                    let _ = self.swarm.disconnect_peer_id(peer);
                    return;
                }
                if self
                    .crypto_peer_id_to_libp2p_id
                    .write()
                    .await
                    .insert(crypto_public_key, peer)
                    .is_none()
                {
                    let _ = self.connected_peers.fetch_add(1, Ordering::Relaxed);
                }
                // Send response with our public key
                let my_peer_id = self.swarm.local_peer_id();
                let msg = my_peer_id.to_bytes();
                let hash = blake3_256(&msg);
                match <CryptoKeyCurve as KeyType>::sign_with_secret_pre_hashed(
                    &mut self.crypto_secret_key.clone(),
                    &hash,
                ) {
                    Ok(signature) => self.swarm.behaviour_mut().p2p.send_response(
                        channel,
                        MyBehaviourResponse::Handshaked {
                            crypto_public_key: self.crypto_secret_key.public(),
                            crypto_signature: signature,
                        },
                    ),
                    Err(e) => {
                        gadget_logging::error!("Failed to sign message: {e}");
                        return;
                    }
                }
            }
            Message { topic, raw_payload } => {
                // Reject messages from self
                if peer == self.my_id {
                    return;
                }

                let topic = IdentTopic::new(topic);
                if let Some((_, tx, _)) = self
                    .inbound_mapping
                    .iter()
                    .find(|r| r.0.to_string() == topic.to_string())
                {
                    if let Err(e) = tx.send(raw_payload) {
                        gadget_logging::error!("Failed to send message to worker: {e}");
                    }
                } else {
                    gadget_logging::error!("No registered worker for topic: {topic}!");
                }
                self.swarm
                    .behaviour_mut()
                    .p2p
                    .send_response(channel, MyBehaviourResponse::MessageHandled)
            }
        };
        if result.is_err() {
            gadget_logging::error!("Failed to send response for {request_id}");
        }
    }

    #[tracing::instrument(skip(self, message))]
    async fn handle_p2p_response(
        &mut self,
        peer: PeerId,
        request_id: request_response::OutboundRequestId,
        message: MyBehaviourResponse,
    ) {
        use crate::gossip::MyBehaviourResponse::{Handshaked, MessageHandled};
        match message {
            Handshaked {
                crypto_public_key,
                crypto_signature,
            } => {
                gadget_logging::trace!("Received handshake-ack message from peer: {peer}");
                let msg = peer.to_bytes();
                let hash = blake3_256(&msg);
                let valid = <CryptoKeyCurve as KeyType>::verify(
                    &crypto_public_key,
                    &hash,
                    &crypto_signature,
                );
                if !valid {
                    gadget_logging::warn!("Invalid signature from peer: {peer}");
                    // TODO: report this peer.
                    self.crypto_peer_id_to_libp2p_id
                        .write()
                        .await
                        .remove(&crypto_public_key);
                    let _ = self.swarm.disconnect_peer_id(peer);
                    return;
                }
                if self
                    .crypto_peer_id_to_libp2p_id
                    .write()
                    .await
                    .insert(crypto_public_key, peer)
                    .is_none()
                {
                    let _ = self.connected_peers.fetch_add(1, Ordering::Relaxed);
                }
            }
            MessageHandled => {}
        }
    }
}
