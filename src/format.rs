//! Size formatting utilities for human-readable output

/// Unit system for size formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    /// Binary units (1024-based): KiB, MiB, GiB, TiB
    Binary,
    /// SI units (1000-based): KB, MB, GB, TB
    Si,
}

impl Default for UnitSystem {
    fn default() -> Self {
        UnitSystem::Binary
    }
}

/// Format a size in bytes to a human-readable string
///
/// # Arguments
/// * `bytes` - Size in bytes
/// * `unit_system` - Whether to use binary (1024) or SI (1000) units
/// * `precision` - Number of decimal places (default: 1)
///
/// # Returns
/// * Formatted string like "4.5 GB" or "1.2 GiB"
pub fn format_size(bytes: u64, unit_system: UnitSystem, precision: usize) -> String {
    let (base, units) = match unit_system {
        UnitSystem::Binary => (1024.0, &["B", "K", "M", "G", "T", "P"][..]),
        UnitSystem::Si => (1000.0, &["B", "KB", "MB", "GB", "TB", "PB"][..]),
    };

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let mut size = bytes_f;
    let mut unit_index = 0;

    while size >= base && unit_index < units.len() - 1 {
        size /= base;
        unit_index += 1;
    }

    // Use precision only for non-byte values
    if unit_index == 0 {
        format!("{} {}", bytes, units[unit_index])
    } else {
        format!("{:.prec$} {}", size, units[unit_index], prec = precision)
    }
}

/// Format a size in bytes with automatic precision adjustment
/// Uses 1 decimal place for clarity, matching the PRD requirements
pub fn format_size_auto(bytes: u64) -> String {
    format_size(bytes, UnitSystem::Binary, 1)
}

/// Calculate percentage and format as string
pub fn format_percentage(part: u64, total: u64) -> String {
    if total == 0 {
        return "0%".to_string();
    }

    let percentage = (part as f64 / total as f64) * 100.0;
    format!("{:>3.0}%", percentage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_binary() {
        assert_eq!(format_size(0, UnitSystem::Binary, 1), "0 B");
        assert_eq!(format_size(500, UnitSystem::Binary, 1), "500 B");
        assert_eq!(format_size(1024, UnitSystem::Binary, 1), "1.0 K");
        assert_eq!(format_size(1536, UnitSystem::Binary, 1), "1.5 K");
        assert_eq!(format_size(1048576, UnitSystem::Binary, 1), "1.0 M");
        assert_eq!(format_size(1073741824, UnitSystem::Binary, 1), "1.0 G");
    }

    #[test]
    fn test_format_size_si() {
        assert_eq!(format_size(1000, UnitSystem::Si, 1), "1.0 KB");
        assert_eq!(format_size(1500, UnitSystem::Si, 1), "1.5 KB");
        assert_eq!(format_size(1000000, UnitSystem::Si, 1), "1.0 MB");
        assert_eq!(format_size(1000000000, UnitSystem::Si, 1), "1.0 GB");
    }

    #[test]
    fn test_format_size_auto() {
        assert_eq!(format_size_auto(0), "0 B");
        assert_eq!(format_size_auto(1024), "1.0 K");
        assert_eq!(format_size_auto(1572864), "1.5 M");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(50, 100), " 50%");
        assert_eq!(format_percentage(1, 3), " 33%");
        assert_eq!(format_percentage(0, 100), "  0%");
        assert_eq!(format_percentage(100, 100), "100%");
    }
}
