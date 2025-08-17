use serde::{Deserialize, Serialize};
use tondi_grpc_core::protowire::tondid_response::Payload;
use tondi_rpc_core::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GrpcReturn {
    Ping(PingResponse),
    GetSyncStatus(GetSyncStatusResponse),
    GetServerInfo(GetServerInfoResponse),
    GetMetrics(GetMetricsResponse),
    GetConnections(GetConnectionsResponse),
    GetSystemInfo(GetSystemInfoResponse),
    SubmitBlock(SubmitBlockResponse),
    GetBlockTemplate(GetBlockTemplateResponse),
    GetBlock(GetBlockResponse),
    GetBlockStatus(GetBlockStatusResponse),
    GetTransaction(GetTransactionResponse),
    GetInfo(GetInfoResponse),
    GetCurrentNetwork(GetCurrentNetworkResponse),
    GetPeerAddresses(GetPeerAddressesResponse),
    GetSink(GetSinkResponse),
    GetMempoolEntry(GetMempoolEntryResponse),
    GetMempoolEntries(GetMempoolEntriesResponse),
    GetConnectedPeerInfo(GetConnectedPeerInfoResponse),
    AddPeer(AddPeerResponse),
    SubmitTransaction(SubmitTransactionResponse),
    SubmitTransactionReplacement(SubmitTransactionReplacementResponse),
    GetSubnetwork(GetSubnetworkResponse),
    GetVirtualChainFromBlock(GetVirtualChainFromBlockResponse),
    GetBlocks(GetBlocksResponse),
    GetBlockCount(GetBlockCountResponse),
    GetBlockDagInfo(GetBlockDagInfoResponse),
    ResolveFinalityConflict(ResolveFinalityConflictResponse),
    Shutdown(ShutdownResponse),
    GetHeader(GetHeaderResponse),
    GetHeaders(GetHeadersResponse),
    GetUtxosByAddresses(GetUtxosByAddressesResponse),
    GetBalanceByAddress(GetBalanceByAddressResponse),
    GetBalancesByAddresses(GetBalancesByAddressesResponse),
    GetSinkBlueScore(GetSinkBlueScoreResponse),
    Ban(BanResponse),
    Unban(UnbanResponse),
    EstimateNetworkHashesPerSecond(EstimateNetworkHashesPerSecondResponse),
    GetMempoolEntriesByAddresses(GetMempoolEntriesByAddressesResponse),
    GetCoinSupply(GetCoinSupplyResponse),
    GetDaaScoreTimestampEstimate(GetDaaScoreTimestampEstimateResponse),
    GetFeeEstimate(GetFeeEstimateResponse),
    GetFeeEstimateExperimental(GetFeeEstimateExperimentalResponse),
    GetCurrentBlockColor(GetCurrentBlockColorResponse),
    GetUtxoReturnAddress(GetUtxoReturnAddressResponse),
}

impl TryFrom<Payload> for GrpcReturn {
    type Error = RpcError;

    fn try_from(payload: Payload) -> Result<Self, Self::Error> {
        use GrpcReturn::*;
        let ret = match payload {
            Payload::PingResponse(m) => Ping(TryFrom::try_from(&m)?),
            Payload::GetSyncStatusResponse(m) => GetSyncStatus(TryFrom::try_from(&m)?),
            Payload::GetServerInfoResponse(m) => GetServerInfo(TryFrom::try_from(&m)?),
            Payload::GetMetricsResponse(m) => GetMetrics(TryFrom::try_from(&m)?),
            Payload::GetConnectionsResponse(m) => GetConnections(TryFrom::try_from(&m)?),
            Payload::GetSystemInfoResponse(m) => GetSystemInfo(TryFrom::try_from(&m)?),
            Payload::SubmitBlockResponse(m) => SubmitBlock(TryFrom::try_from(&m)?),
            Payload::GetBlockTemplateResponse(m) => GetBlockTemplate(TryFrom::try_from(&m)?),
            Payload::GetBlockResponse(m) => GetBlock(TryFrom::try_from(&m)?),
            Payload::GetBlockStatusResponse(m) => GetBlockStatus(TryFrom::try_from(&m)?),
            Payload::GetTransactionResponse(m) => GetTransaction(TryFrom::try_from(&m)?),
            Payload::GetInfoResponse(m) => GetInfo(TryFrom::try_from(&m)?),
            Payload::GetCurrentNetworkResponse(m) => GetCurrentNetwork(TryFrom::try_from(&m)?),
            Payload::GetPeerAddressesResponse(m) => GetPeerAddresses(TryFrom::try_from(&m)?),
            Payload::GetSinkResponse(m) => GetSink(TryFrom::try_from(&m)?),
            Payload::GetMempoolEntryResponse(m) => GetMempoolEntry(TryFrom::try_from(&m)?),
            Payload::GetMempoolEntriesResponse(m) => GetMempoolEntries(TryFrom::try_from(&m)?),
            Payload::GetConnectedPeerInfoResponse(m) => {
                GetConnectedPeerInfo(TryFrom::try_from(&m)?)
            },
            Payload::AddPeerResponse(m) => AddPeer(TryFrom::try_from(&m)?),
            Payload::SubmitTransactionResponse(m) => SubmitTransaction(TryFrom::try_from(&m)?),
            Payload::SubmitTransactionReplacementResponse(m) => {
                SubmitTransactionReplacement(TryFrom::try_from(&m)?)
            },
            Payload::GetSubnetworkResponse(m) => GetSubnetwork(TryFrom::try_from(&m)?),
            Payload::GetVirtualChainFromBlockResponse(m) => {
                GetVirtualChainFromBlock(TryFrom::try_from(&m)?)
            },
            Payload::GetBlocksResponse(m) => GetBlocks(TryFrom::try_from(&m)?),
            Payload::GetBlockCountResponse(m) => GetBlockCount(TryFrom::try_from(&m)?),
            Payload::GetBlockDagInfoResponse(m) => GetBlockDagInfo(TryFrom::try_from(&m)?),
            Payload::ResolveFinalityConflictResponse(m) => {
                ResolveFinalityConflict(TryFrom::try_from(&m)?)
            },
            Payload::ShutdownResponse(m) => Shutdown(TryFrom::try_from(&m)?),
            Payload::GetHeaderResponse(m) => GetHeader(TryFrom::try_from(&m)?),
            Payload::GetHeadersResponse(m) => GetHeaders(TryFrom::try_from(&m)?),
            Payload::GetUtxosByAddressesResponse(m) => GetUtxosByAddresses(TryFrom::try_from(&m)?),
            Payload::GetBalanceByAddressResponse(m) => GetBalanceByAddress(TryFrom::try_from(&m)?),
            Payload::GetBalancesByAddressesResponse(m) => {
                GetBalancesByAddresses(TryFrom::try_from(&m)?)
            },
            Payload::GetSinkBlueScoreResponse(m) => GetSinkBlueScore(TryFrom::try_from(&m)?),
            Payload::BanResponse(m) => Ban(TryFrom::try_from(&m)?),
            Payload::UnbanResponse(m) => Unban(TryFrom::try_from(&m)?),
            Payload::EstimateNetworkHashesPerSecondResponse(m) => {
                EstimateNetworkHashesPerSecond(TryFrom::try_from(&m)?)
            },
            Payload::GetMempoolEntriesByAddressesResponse(m) => {
                GetMempoolEntriesByAddresses(TryFrom::try_from(&m)?)
            },
            Payload::GetCoinSupplyResponse(m) => GetCoinSupply(TryFrom::try_from(&m)?),
            Payload::GetDaaScoreTimestampEstimateResponse(m) => {
                GetDaaScoreTimestampEstimate(TryFrom::try_from(&m)?)
            },
            Payload::GetFeeEstimateResponse(m) => GetFeeEstimate(TryFrom::try_from(&m)?),
            Payload::GetFeeEstimateExperimentalResponse(m) => {
                GetFeeEstimateExperimental(TryFrom::try_from(&m)?)
            },
            Payload::GetCurrentBlockColorResponse(m) => {
                GetCurrentBlockColor(TryFrom::try_from(&m)?)
            },
            Payload::GetUtxoReturnAddressResponse(m) => {
                GetUtxoReturnAddress(TryFrom::try_from(&m)?)
            },
            _ => unreachable!("Reserved Interface"),
        };
        Ok(ret)
    }
}
