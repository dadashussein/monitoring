//! Shared utility functions
//!
//! This module provides common utility functions used across multiple modules.
//! All utility functions are stateless and reusable.
//!
//! # Functions
//!
//! - [`bytes_to_gb`]: Convert bytes to gigabytes for human-readable display
//! - [`refresh_system`]: Refresh all system information for monitoring
//!
//! # Example
//!
//! ```
//! use ubuntu_resource_api::utils::bytes_to_gb;
//!
//! let bytes = 1_073_741_824; // 1 GB in bytes
//! let gb = bytes_to_gb(bytes);
//! assert_eq!(gb, 1.0);
//! ```

use sysinfo::System;

/// Convert bytes to gigabytes
///
/// # Arguments
///
/// * `bytes` - Number of bytes to convert
///
/// # Returns
///
/// The equivalent value in gigabytes (GB)
///
/// # Example
///
/// ```
/// use ubuntu_resource_api::utils::bytes_to_gb;
///
/// let bytes = 2_147_483_648; // 2 GB
/// let gb = bytes_to_gb(bytes);
/// assert_eq!(gb, 2.0);
/// ```
pub fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// Refresh all system information
///
/// This function updates all system metrics including CPU, memory, disk, network, and processes.
///
/// # Arguments
///
/// * `system` - Mutable reference to the System instance to refresh
///
/// # Example
///
/// ```no_run
/// use sysinfo::System;
/// use ubuntu_resource_api::utils::refresh_system;
///
/// let mut system = System::new_all();
/// refresh_system(&mut system);
/// ```
pub fn refresh_system(system: &mut System) {
    system.refresh_all();
}
