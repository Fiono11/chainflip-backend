use futures::Stream;
use slog::o;
use tokio_stream::StreamExt;

use crate::{
    logging::COMPONENT_KEY,
    mq::{IMQClient, Subject},
    p2p::P2PMessage,
};

use super::{P2PMessageCommand, P2PNetworkClient};
use crate::p2p::ValidatorId;

/// Intermediates P2P events between MQ and P2P interface
pub fn start<S, P2P, MQ>(
    mut p2p: P2P,
    mq: MQ,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    logger: &slog::Logger,
) -> impl futures::Future
where
    MQ: IMQClient + Send,
    S: Stream<Item = P2PMessage> + Unpin,
    P2P: P2PNetworkClient<ValidatorId, S> + Send,
{
    let logger = logger.new(o!(COMPONENT_KEY => "P2PConductor"));

    async move {
        slog::info!(logger, "Starting");

        let mut p2p_command_stream = mq
            .subscribe::<P2PMessageCommand>(Subject::P2POutgoing)
            .await
            .expect("Should be able to subscribe to Subject::P2POutgoing");

        let mut p2p_stream = p2p.take_stream().await.expect("Should have p2p stream");

        loop {
            tokio::select! {
                Some(outgoing) = p2p_command_stream.next() => {
                    if let Ok(P2PMessageCommand { destination, data }) = outgoing {
                        p2p.send(&destination, &data).await.expect("Could not send outgoing P2PMessageCommand");
                    }
                }
                Some(incoming) = p2p_stream.next() => {
                    mq.publish::<P2PMessage>(Subject::P2PIncoming, &incoming)
                        .await
                        .expect("Could not publish incoming message to Subject::P2PIncoming");
                }
                Ok(()) = &mut shutdown_rx =>{
                    slog::info!(logger, "Shutting down");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::{
        logging,
        mq::mq_mock::MQMock,
        p2p::{mock::NetworkMock, P2PMessageCommand, ValidatorId},
    };

    use super::*;

    use tokio::time::timeout;

    #[tokio::test]
    async fn conductor_reads_from_mq() {
        use crate::mq::Subject;

        let network = NetworkMock::new();

        let logger = logging::test_utils::create_test_logger();

        // NOTE: for some reason connecting to the mock nat's server
        // is slow (0.5-1 seconds), which will add up when we have a
        // lot of tests. Will need to fix this.

        // Validator 1 setup
        let id_1: ValidatorId = ValidatorId::new(1);

        let mq = MQMock::new();

        let mc1 = mq.get_client();
        let mc1_copy = mq.get_client();
        let p2p_client_1 = network.new_client(id_1);

        // Validator 2 setup
        let id_2: ValidatorId = ValidatorId::new(2);

        let mq = MQMock::new();
        let mc2 = mq.get_client();
        let mc2_copy = mq.get_client();
        let p2p_client_2 = network.new_client(id_2.clone());

        let (_, shutdown_conductor1_rx) = tokio::sync::oneshot::channel::<()>();
        let (_, shutdown_conductor2_rx) = tokio::sync::oneshot::channel::<()>();

        let conductor_fut_1 = timeout(
            Duration::from_millis(100),
            start(p2p_client_1, mc1, shutdown_conductor1_rx, &logger),
        );
        let conductor_fut_2 = timeout(
            Duration::from_millis(100),
            start(p2p_client_2, mc2, shutdown_conductor2_rx, &logger),
        );

        let msg = String::from("hello");

        let message = P2PMessageCommand {
            destination: id_2,
            data: Vec::from(msg.as_bytes()),
        };

        let write_fut = async move {
            mc1_copy
                .publish(Subject::P2POutgoing, &message)
                .await
                .expect("Could not publish incoming P2PMessageCommand to Subject::P2POutgoing");
        };

        let read_fut = async move {
            let mut p2p_messages = mc2_copy
                .subscribe::<P2PMessage>(Subject::P2PIncoming)
                .await
                .expect("Could not subscribe to Subject::P2PIncoming");

            // Second client should be able to receive the message
            let maybe_msg = timeout(Duration::from_millis(100), p2p_messages.next()).await;

            assert!(maybe_msg.is_ok(), "recv timeout");

            assert_eq!(maybe_msg.unwrap().unwrap().unwrap().data, msg.as_bytes());
        };

        let _ = futures::join!(conductor_fut_1, conductor_fut_2, write_fut, read_fut);
    }
}
