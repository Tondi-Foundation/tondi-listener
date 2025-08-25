use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use tondi_rpc_core::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "content")]
pub enum Event {
    BlockAdded(BlockAddedNotification),
    VirtualChainChanged(VirtualChainChangedNotification),
    FinalityConflict(FinalityConflictNotification),
    FinalityConflictResolved(FinalityConflictResolvedNotification),
    UtxosChanged(UtxosChangedNotification),
    SinkBlueScoreChanged(SinkBlueScoreChangedNotification),
    VirtualDaaScoreChanged(VirtualDaaScoreChangedNotification),
    PruningPointUtxoSetOverride(PruningPointUtxoSetOverrideNotification),
    NewBlockTemplate(NewBlockTemplateNotification),
}

impl TryFrom<Event> for Message {
    type Error = SerdeJsonError;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        let text = serde_json::to_string(&event)?;
        Ok(Message::Text(text.into()))
    }
}

impl From<Notification> for Event {
    fn from(notification: Notification) -> Self {
        use Notification::*;
        match notification {
            BlockAdded(m) => Event::BlockAdded(m),
            VirtualChainChanged(m) => Event::VirtualChainChanged(m),
            FinalityConflict(m) => Event::FinalityConflict(m),
            FinalityConflictResolved(m) => Event::FinalityConflictResolved(m),
            UtxosChanged(m) => Event::UtxosChanged(m),
            SinkBlueScoreChanged(m) => Event::SinkBlueScoreChanged(m),
            VirtualDaaScoreChanged(m) => Event::VirtualDaaScoreChanged(m),
            PruningPointUtxoSetOverride(m) => Event::PruningPointUtxoSetOverride(m),
            NewBlockTemplate(m) => Event::NewBlockTemplate(m),
        }
    }
}
