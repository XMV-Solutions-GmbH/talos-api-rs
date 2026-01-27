// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed wrappers for System Information APIs.
//!
//! Provides access to system metrics like CPU, memory, disk, and network stats.

use crate::api::generated::machine::{
    CpUsInfo as ProtoCpUsInfo, CpuInfo as ProtoCpuInfo, CpuInfoResponse as ProtoCpuInfoResponse,
    DiskStat as ProtoDiskStat, DiskStats as ProtoDiskStats,
    DiskStatsResponse as ProtoDiskStatsResponse, LoadAvg as ProtoLoadAvg,
    LoadAvgResponse as ProtoLoadAvgResponse, Memory as ProtoMemory,
    MemoryResponse as ProtoMemoryResponse, MountStat as ProtoMountStat,
    MountsResponse as ProtoMountsResponse, NetDev as ProtoNetDev,
    NetworkDeviceStats as ProtoNetworkDeviceStats,
    NetworkDeviceStatsResponse as ProtoNetworkDeviceStatsResponse, Process as ProtoProcess,
    ProcessInfo as ProtoProcessInfo, ProcessesResponse as ProtoProcessesResponse,
};

// =============================================================================
// LoadAvg
// =============================================================================

/// System load averages.
#[derive(Debug, Clone)]
pub struct LoadAvgResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// 1-minute load average.
    pub load1: f64,
    /// 5-minute load average.
    pub load5: f64,
    /// 15-minute load average.
    pub load15: f64,
}

impl From<ProtoLoadAvg> for LoadAvgResult {
    fn from(proto: ProtoLoadAvg) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            load1: proto.load1,
            load5: proto.load5,
            load15: proto.load15,
        }
    }
}

/// Response from load average request.
#[derive(Debug, Clone)]
pub struct LoadAvgResponse {
    /// Results from each node.
    pub results: Vec<LoadAvgResult>,
}

impl From<ProtoLoadAvgResponse> for LoadAvgResponse {
    fn from(proto: ProtoLoadAvgResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(LoadAvgResult::from)
                .collect(),
        }
    }
}

impl LoadAvgResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&LoadAvgResult> {
        self.results.first()
    }
}

// =============================================================================
// Memory
// =============================================================================

/// Memory information for a node.
#[derive(Debug, Clone)]
pub struct MemoryResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Total memory in bytes.
    pub mem_total: u64,
    /// Free memory in bytes.
    pub mem_free: u64,
    /// Available memory in bytes.
    pub mem_available: u64,
    /// Buffer memory in bytes.
    pub buffers: u64,
    /// Cached memory in bytes.
    pub cached: u64,
    /// Swap total in bytes.
    pub swap_total: u64,
    /// Swap free in bytes.
    pub swap_free: u64,
}

impl From<ProtoMemory> for MemoryResult {
    fn from(proto: ProtoMemory) -> Self {
        let meminfo = proto.meminfo.unwrap_or_default();
        Self {
            node: proto.metadata.map(|m| m.hostname),
            mem_total: meminfo.memtotal,
            mem_free: meminfo.memfree,
            mem_available: meminfo.memavailable,
            buffers: meminfo.buffers,
            cached: meminfo.cached,
            swap_total: meminfo.swaptotal,
            swap_free: meminfo.swapfree,
        }
    }
}

impl MemoryResult {
    /// Get total memory in bytes.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.mem_total
    }

    /// Get free memory in bytes.
    #[must_use]
    pub fn free(&self) -> u64 {
        self.mem_free
    }

    /// Get available memory in bytes.
    #[must_use]
    pub fn available(&self) -> u64 {
        self.mem_available
    }

    /// Get used memory in bytes.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.mem_total.saturating_sub(self.mem_available)
    }

    /// Get memory usage percentage.
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        if self.mem_total == 0 {
            0.0
        } else {
            (self.used() as f64 / self.mem_total as f64) * 100.0
        }
    }
}

/// Response from memory request.
#[derive(Debug, Clone)]
pub struct MemoryResponse {
    /// Results from each node.
    pub results: Vec<MemoryResult>,
}

impl From<ProtoMemoryResponse> for MemoryResponse {
    fn from(proto: ProtoMemoryResponse) -> Self {
        Self {
            results: proto.messages.into_iter().map(MemoryResult::from).collect(),
        }
    }
}

impl MemoryResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&MemoryResult> {
        self.results.first()
    }
}

// =============================================================================
// CPUInfo
// =============================================================================

/// Information about a single CPU.
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// Processor number.
    pub processor: u32,
    /// Vendor ID.
    pub vendor_id: String,
    /// Model name.
    pub model_name: String,
    /// CPU MHz.
    pub cpu_mhz: f64,
    /// Number of cores.
    pub cpu_cores: u32,
    /// CPU flags.
    pub flags: Vec<String>,
}

impl From<ProtoCpuInfo> for CpuInfo {
    fn from(proto: ProtoCpuInfo) -> Self {
        Self {
            processor: proto.processor,
            vendor_id: proto.vendor_id,
            model_name: proto.model_name,
            cpu_mhz: proto.cpu_mhz,
            cpu_cores: proto.cpu_cores,
            flags: proto.flags,
        }
    }
}

/// CPU information result for a node.
#[derive(Debug, Clone)]
pub struct CpuInfoResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// CPU information.
    pub cpus: Vec<CpuInfo>,
}

impl From<ProtoCpUsInfo> for CpuInfoResult {
    fn from(proto: ProtoCpUsInfo) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            cpus: proto.cpu_info.into_iter().map(CpuInfo::from).collect(),
        }
    }
}

/// Response from CPU info request.
#[derive(Debug, Clone)]
pub struct CpuInfoResponse {
    /// Results from each node.
    pub results: Vec<CpuInfoResult>,
}

impl From<ProtoCpuInfoResponse> for CpuInfoResponse {
    fn from(proto: ProtoCpuInfoResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(CpuInfoResult::from)
                .collect(),
        }
    }
}

impl CpuInfoResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&CpuInfoResult> {
        self.results.first()
    }

    /// Get total number of CPUs across all results.
    #[must_use]
    pub fn total_cpus(&self) -> usize {
        self.results.iter().map(|r| r.cpus.len()).sum()
    }
}

// =============================================================================
// DiskStats
// =============================================================================

/// Statistics for a single disk.
#[derive(Debug, Clone)]
pub struct DiskStat {
    /// Device name.
    pub name: String,
    /// Reads completed.
    pub read_completed: u64,
    /// Sectors read.
    pub read_sectors: u64,
    /// Read time in ms.
    pub read_time_ms: u64,
    /// Writes completed.
    pub write_completed: u64,
    /// Sectors written.
    pub write_sectors: u64,
    /// Write time in ms.
    pub write_time_ms: u64,
    /// I/O operations in progress.
    pub io_in_progress: u64,
    /// I/O time in ms.
    pub io_time_ms: u64,
}

impl From<ProtoDiskStat> for DiskStat {
    fn from(proto: ProtoDiskStat) -> Self {
        Self {
            name: proto.name,
            read_completed: proto.read_completed,
            read_sectors: proto.read_sectors,
            read_time_ms: proto.read_time_ms,
            write_completed: proto.write_completed,
            write_sectors: proto.write_sectors,
            write_time_ms: proto.write_time_ms,
            io_in_progress: proto.io_in_progress,
            io_time_ms: proto.io_time_ms,
        }
    }
}

/// Disk statistics result for a node.
#[derive(Debug, Clone)]
pub struct DiskStatsResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Total stats across all devices.
    pub total: Option<DiskStat>,
    /// Per-device stats.
    pub devices: Vec<DiskStat>,
}

impl From<ProtoDiskStats> for DiskStatsResult {
    fn from(proto: ProtoDiskStats) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            total: proto.total.map(DiskStat::from),
            devices: proto.devices.into_iter().map(DiskStat::from).collect(),
        }
    }
}

/// Response from disk stats request.
#[derive(Debug, Clone)]
pub struct DiskStatsResponse {
    /// Results from each node.
    pub results: Vec<DiskStatsResult>,
}

impl From<ProtoDiskStatsResponse> for DiskStatsResponse {
    fn from(proto: ProtoDiskStatsResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(DiskStatsResult::from)
                .collect(),
        }
    }
}

impl DiskStatsResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&DiskStatsResult> {
        self.results.first()
    }
}

// =============================================================================
// NetworkDeviceStats
// =============================================================================

/// Statistics for a network device.
#[derive(Debug, Clone)]
pub struct NetDevStat {
    /// Device name.
    pub name: String,
    /// Bytes received.
    pub rx_bytes: u64,
    /// Packets received.
    pub rx_packets: u64,
    /// Receive errors.
    pub rx_errors: u64,
    /// Bytes transmitted.
    pub tx_bytes: u64,
    /// Packets transmitted.
    pub tx_packets: u64,
    /// Transmit errors.
    pub tx_errors: u64,
}

impl From<ProtoNetDev> for NetDevStat {
    fn from(proto: ProtoNetDev) -> Self {
        Self {
            name: proto.name,
            rx_bytes: proto.rx_bytes,
            rx_packets: proto.rx_packets,
            rx_errors: proto.rx_errors,
            tx_bytes: proto.tx_bytes,
            tx_packets: proto.tx_packets,
            tx_errors: proto.tx_errors,
        }
    }
}

/// Network stats result for a node.
#[derive(Debug, Clone)]
pub struct NetworkDeviceStatsResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Total stats across all devices.
    pub total: Option<NetDevStat>,
    /// Per-device stats.
    pub devices: Vec<NetDevStat>,
}

impl From<ProtoNetworkDeviceStats> for NetworkDeviceStatsResult {
    fn from(proto: ProtoNetworkDeviceStats) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            total: proto.total.map(NetDevStat::from),
            devices: proto.devices.into_iter().map(NetDevStat::from).collect(),
        }
    }
}

/// Response from network device stats request.
#[derive(Debug, Clone)]
pub struct NetworkDeviceStatsResponse {
    /// Results from each node.
    pub results: Vec<NetworkDeviceStatsResult>,
}

impl From<ProtoNetworkDeviceStatsResponse> for NetworkDeviceStatsResponse {
    fn from(proto: ProtoNetworkDeviceStatsResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(NetworkDeviceStatsResult::from)
                .collect(),
        }
    }
}

impl NetworkDeviceStatsResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&NetworkDeviceStatsResult> {
        self.results.first()
    }
}

// =============================================================================
// Mounts
// =============================================================================

/// Mount point information.
#[derive(Debug, Clone)]
pub struct MountStat {
    /// Filesystem type.
    pub filesystem: String,
    /// Total size in bytes.
    pub size: u64,
    /// Available space in bytes.
    pub available: u64,
    /// Mount point path.
    pub mounted_on: String,
}

impl From<ProtoMountStat> for MountStat {
    fn from(proto: ProtoMountStat) -> Self {
        Self {
            filesystem: proto.filesystem,
            size: proto.size,
            available: proto.available,
            mounted_on: proto.mounted_on,
        }
    }
}

impl MountStat {
    /// Get used space in bytes.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.size.saturating_sub(self.available)
    }

    /// Get usage percentage.
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            (self.used() as f64 / self.size as f64) * 100.0
        }
    }
}

/// Mounts result for a node.
#[derive(Debug, Clone)]
pub struct MountsResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// Mount points.
    pub stats: Vec<MountStat>,
}

/// Response from mounts request.
#[derive(Debug, Clone)]
pub struct MountsResponse {
    /// Results from each node.
    pub results: Vec<MountsResult>,
}

impl From<ProtoMountsResponse> for MountsResponse {
    fn from(proto: ProtoMountsResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(|m| MountsResult {
                    node: m.metadata.map(|meta| meta.hostname),
                    stats: m.stats.into_iter().map(MountStat::from).collect(),
                })
                .collect(),
        }
    }
}

impl MountsResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&MountsResult> {
        self.results.first()
    }
}

// =============================================================================
// Processes
// =============================================================================

/// Information about a process.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: i32,
    /// Parent process ID.
    pub ppid: i32,
    /// Process state.
    pub state: String,
    /// Number of threads.
    pub threads: i32,
    /// CPU time.
    pub cpu_time: f64,
    /// Virtual memory size.
    pub virtual_memory: u64,
    /// Resident memory size.
    pub resident_memory: u64,
    /// Command name.
    pub command: String,
    /// Executable path.
    pub executable: String,
    /// Command line arguments.
    pub args: String,
}

impl From<ProtoProcessInfo> for ProcessInfo {
    fn from(proto: ProtoProcessInfo) -> Self {
        Self {
            pid: proto.pid,
            ppid: proto.ppid,
            state: proto.state,
            threads: proto.threads,
            cpu_time: proto.cpu_time,
            virtual_memory: proto.virtual_memory,
            resident_memory: proto.resident_memory,
            command: proto.command,
            executable: proto.executable,
            args: proto.args,
        }
    }
}

/// Processes result for a node.
#[derive(Debug, Clone)]
pub struct ProcessesResult {
    /// Node that returned this result.
    pub node: Option<String>,
    /// List of processes.
    pub processes: Vec<ProcessInfo>,
}

impl From<ProtoProcess> for ProcessesResult {
    fn from(proto: ProtoProcess) -> Self {
        Self {
            node: proto.metadata.map(|m| m.hostname),
            processes: proto.processes.into_iter().map(ProcessInfo::from).collect(),
        }
    }
}

/// Response from processes request.
#[derive(Debug, Clone)]
pub struct ProcessesResponse {
    /// Results from each node.
    pub results: Vec<ProcessesResult>,
}

impl From<ProtoProcessesResponse> for ProcessesResponse {
    fn from(proto: ProtoProcessesResponse) -> Self {
        Self {
            results: proto
                .messages
                .into_iter()
                .map(ProcessesResult::from)
                .collect(),
        }
    }
}

impl ProcessesResponse {
    /// Get the first result.
    #[must_use]
    pub fn first(&self) -> Option<&ProcessesResult> {
        self.results.first()
    }

    /// Get total number of processes across all results.
    #[must_use]
    pub fn total_processes(&self) -> usize {
        self.results.iter().map(|r| r.processes.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_avg_result() {
        let result = LoadAvgResult {
            node: Some("node1".to_string()),
            load1: 0.5,
            load5: 0.7,
            load15: 0.9,
        };
        assert_eq!(result.load1, 0.5);
    }

    #[test]
    fn test_memory_result() {
        let result = MemoryResult {
            node: Some("node1".to_string()),
            mem_total: 16_000_000_000,
            mem_free: 4_000_000_000,
            mem_available: 8_000_000_000,
            buffers: 100_000_000,
            cached: 2_000_000_000,
            swap_total: 1_000_000_000,
            swap_free: 500_000_000,
        };

        assert_eq!(result.total(), 16_000_000_000);
        assert_eq!(result.available(), 8_000_000_000);
        assert_eq!(result.used(), 8_000_000_000);
        assert!((result.usage_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_mount_stat() {
        let stat = MountStat {
            filesystem: "ext4".to_string(),
            size: 100_000_000_000,
            available: 40_000_000_000,
            mounted_on: "/".to_string(),
        };

        assert_eq!(stat.used(), 60_000_000_000);
        assert!((stat.usage_percent() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_cpu_info() {
        let cpu = CpuInfo {
            processor: 0,
            vendor_id: "GenuineIntel".to_string(),
            model_name: "Intel Core i7".to_string(),
            cpu_mhz: 3200.0,
            cpu_cores: 4,
            flags: vec!["avx".to_string(), "sse".to_string()],
        };

        assert_eq!(cpu.processor, 0);
        assert_eq!(cpu.cpu_cores, 4);
    }

    #[test]
    fn test_disk_stat() {
        let stat = DiskStat {
            name: "sda".to_string(),
            read_completed: 1000,
            read_sectors: 50000,
            read_time_ms: 500,
            write_completed: 500,
            write_sectors: 25000,
            write_time_ms: 250,
            io_in_progress: 2,
            io_time_ms: 750,
        };

        assert_eq!(stat.name, "sda");
        assert_eq!(stat.read_completed, 1000);
    }

    #[test]
    fn test_net_dev_stat() {
        let stat = NetDevStat {
            name: "eth0".to_string(),
            rx_bytes: 1_000_000,
            rx_packets: 1000,
            rx_errors: 0,
            tx_bytes: 500_000,
            tx_packets: 500,
            tx_errors: 0,
        };

        assert_eq!(stat.name, "eth0");
        assert_eq!(stat.rx_bytes, 1_000_000);
    }

    #[test]
    fn test_process_info() {
        let proc = ProcessInfo {
            pid: 1,
            ppid: 0,
            state: "S".to_string(),
            threads: 1,
            cpu_time: 10.5,
            virtual_memory: 1_000_000,
            resident_memory: 500_000,
            command: "init".to_string(),
            executable: "/sbin/init".to_string(),
            args: "".to_string(),
        };

        assert_eq!(proc.pid, 1);
        assert_eq!(proc.command, "init");
    }
}
