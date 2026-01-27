// SPDX-License-Identifier: MIT OR Apache-2.0

//! Strongly typed domain wrappers for Talos resources.
//!
//! This module provides ergonomic, type-safe wrappers around the raw
//! protobuf types generated from the Talos API.

mod advanced;
mod bootstrap;
mod configuration;
mod dmesg;
mod etcd;
mod files;
mod images;
mod kubeconfig;
mod logs;
mod reset;
mod services;
mod system;
mod upgrade;

pub use bootstrap::{
    BootstrapRequest, BootstrapRequestBuilder, BootstrapResponse, BootstrapResult,
};
pub use configuration::{
    ApplyConfigurationRequest, ApplyConfigurationRequestBuilder, ApplyConfigurationResponse,
    ApplyConfigurationResult, ApplyMode,
};
pub use dmesg::{DmesgRequest, DmesgRequestBuilder, DmesgResponse};
pub use etcd::{
    EtcdAlarmDisarmResponse, EtcdAlarmDisarmResult, EtcdAlarmListResponse, EtcdAlarmResult,
    EtcdAlarmType, EtcdDefragmentResponse, EtcdDefragmentResult, EtcdForfeitLeadershipRequest,
    EtcdForfeitLeadershipResponse, EtcdForfeitLeadershipResult, EtcdLeaveClusterRequest,
    EtcdLeaveClusterResponse, EtcdLeaveClusterResult, EtcdMember, EtcdMemberAlarm,
    EtcdMemberListRequest, EtcdMemberListResponse, EtcdMemberStatus, EtcdMembersResult,
    EtcdRemoveMemberByIdRequest, EtcdRemoveMemberByIdResponse, EtcdRemoveMemberByIdResult,
    EtcdStatusResponse, EtcdStatusResult,
};
pub use kubeconfig::KubeconfigResponse;
pub use logs::{ContainerDriver, LogsRequest, LogsRequestBuilder, LogsResponse};
pub use reset::{
    ResetPartitionSpec, ResetRequest, ResetRequestBuilder, ResetResponse, ResetResult, WipeMode,
};
pub use services::{
    ServiceRestartRequest, ServiceRestartResponse, ServiceRestartResult, ServiceStartRequest,
    ServiceStartResponse, ServiceStartResult, ServiceStopRequest, ServiceStopResponse,
    ServiceStopResult,
};
pub use upgrade::{
    UpgradeRebootMode, UpgradeRequest, UpgradeRequestBuilder, UpgradeResponse, UpgradeResult,
};

pub use system::{
    CpuInfo, CpuInfoResponse, CpuInfoResult, DiskStat, DiskStatsResponse, DiskStatsResult,
    LoadAvgResponse, LoadAvgResult, MemoryResponse, MemoryResult, MountStat, MountsResponse,
    MountsResult, NetDevStat, NetworkDeviceStatsResponse, NetworkDeviceStatsResult, ProcessInfo,
    ProcessesResponse, ProcessesResult,
};

pub use files::{
    CopyRequest, CopyResponse, DiskUsageInfo, DiskUsageRequest, DiskUsageRequestBuilder,
    DiskUsageResponse, FileInfo, FileType, ListRequest, ListRequestBuilder, ListResponse,
    ReadRequest, ReadResponse,
};

pub use advanced::{
    ConnectionRecord, ConnectionState, GenerateClientConfigurationRequest,
    GenerateClientConfigurationRequestBuilder, GenerateClientConfigurationResponse,
    GenerateClientConfigurationResult, L4ProtoFilter, NetstatFilter, NetstatRequest,
    NetstatRequestBuilder, NetstatResponse, NetstatResult, PacketCaptureRequest,
    PacketCaptureRequestBuilder, PacketCaptureResponse, RollbackResponse, RollbackResult,
};

pub use images::{
    ContainerdNamespace, ImageInfo, ImageListRequest, ImagePullRequest, ImagePullResponse,
    ImagePullResult,
};
