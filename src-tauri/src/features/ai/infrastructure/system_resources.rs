//! Lecture légère CPU/RAM/VRAM pour le rail — sans initialiser llama.cpp.

use std::sync::Mutex;

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::features::ai::domain::SystemResourceSnapshot;

#[derive(Debug, Clone, Copy)]
struct GpuVramCache {
    total_mb: u64,
    available_mb: u64,
}

static GPU_VRAM_CACHE: Mutex<Option<GpuVramCache>> = Mutex::new(None);
static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

/// Mémorise la VRAM d'un GPU dédié (rempli par la détection hardware locale).
pub fn cache_gpu_vram(total_mb: u64, available_mb: u64) {
    if total_mb == 0 {
        return;
    }
    if let Ok(mut guard) = GPU_VRAM_CACHE.lock() {
        *guard = Some(GpuVramCache {
            total_mb,
            available_mb: available_mb.min(total_mb),
        });
    }
}

/// Vide le cache VRAM (tests).
#[cfg(test)]
pub fn clear_gpu_vram_cache() {
    if let Ok(mut guard) = GPU_VRAM_CACHE.lock() {
        *guard = None;
    }
}

#[must_use]
pub fn take_system_resource_snapshot() -> SystemResourceSnapshot {
    let apple_silicon = is_apple_silicon();
    let (cpu_percent, ram_used_mb, ram_total_mb, ram_used_percent) = read_cpu_and_ram();
    let cached = GPU_VRAM_CACHE.lock().ok().and_then(|guard| *guard);
    build_snapshot(
        apple_silicon,
        cpu_percent,
        ram_used_mb,
        ram_total_mb,
        ram_used_percent,
        cached,
    )
}

fn is_apple_silicon() -> bool {
    let architecture = System::cpu_arch();
    cfg!(target_os = "macos") && matches!(architecture.as_str(), "aarch64" | "arm64")
}

fn read_cpu_and_ram() -> (f32, u64, u64, f32) {
    let mut guard = match SYSTEM.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let system = guard.get_or_insert_with(|| {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(MemoryRefreshKind::nothing().with_ram())
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_memory();
        system.refresh_cpu_usage();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_usage();
        system
    });

    system.refresh_memory();
    system.refresh_cpu_usage();

    let ram_total_mb = system.total_memory() / 1_048_576;
    let ram_available_mb = system.available_memory() / 1_048_576;
    let ram_used_mb = ram_total_mb.saturating_sub(ram_available_mb);
    let ram_used_percent = percent(ram_used_mb, ram_total_mb);
    let cpu_percent = clamp_percent(system.global_cpu_usage());

    (cpu_percent, ram_used_mb, ram_total_mb, ram_used_percent)
}

fn build_snapshot(
    apple_silicon: bool,
    cpu_percent: f32,
    ram_used_mb: u64,
    ram_total_mb: u64,
    ram_used_percent: f32,
    cached: Option<GpuVramCache>,
) -> SystemResourceSnapshot {
    let (vram_available, vram_used_percent, vram_used_mb, vram_total_mb) = if apple_silicon {
        (
            true,
            Some(ram_used_percent),
            Some(ram_used_mb),
            Some(ram_total_mb),
        )
    } else if let Some(cache) = cached {
        let used = cache.total_mb.saturating_sub(cache.available_mb);
        (
            true,
            Some(percent(used, cache.total_mb)),
            Some(used),
            Some(cache.total_mb),
        )
    } else {
        (false, None, None, None)
    };

    SystemResourceSnapshot {
        cpu_percent,
        ram_used_percent,
        ram_used_mb,
        ram_total_mb,
        vram_available,
        vram_used_percent,
        vram_used_mb,
        vram_total_mb,
    }
}

fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    clamp_percent((used as f64 * 100.0 / total as f64) as f32)
}

fn clamp_percent(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_borne_les_pourcentages_entre_0_et_100() {
        let snap = take_system_resource_snapshot();
        assert!((0.0..=100.0).contains(&snap.cpu_percent));
        assert!((0.0..=100.0).contains(&snap.ram_used_percent));
        if snap.vram_available {
            let p = snap.vram_used_percent.expect("vram_used_percent");
            assert!((0.0..=100.0).contains(&p));
            assert!(snap.vram_used_mb.is_some());
            assert!(snap.vram_total_mb.is_some());
        } else {
            assert!(snap.vram_used_percent.is_none());
            assert!(snap.vram_used_mb.is_none());
            assert!(snap.vram_total_mb.is_none());
        }
    }

    #[test]
    fn snapshot_avec_cache_gpu_expose_la_vram() {
        clear_gpu_vram_cache();
        cache_gpu_vram(8192, 4096);
        let snap = build_snapshot(
            false,
            12.0,
            4000,
            16000,
            25.0,
            GPU_VRAM_CACHE.lock().ok().and_then(|g| *g),
        );
        assert!(snap.vram_available);
        assert_eq!(snap.vram_total_mb, Some(8192));
        assert_eq!(snap.vram_used_mb, Some(4096));
        let p = snap.vram_used_percent.expect("percent");
        assert!((49.0..=51.0).contains(&p));
        clear_gpu_vram_cache();
    }

    #[test]
    fn apple_silicon_utilise_la_memoire_unifiee() {
        clear_gpu_vram_cache();
        let snap = build_snapshot(true, 8.0, 8000, 16000, 50.0, None);
        assert!(snap.vram_available);
        assert_eq!(snap.vram_used_mb, Some(8000));
        assert_eq!(snap.vram_total_mb, Some(16000));
        assert_eq!(snap.vram_used_percent, Some(50.0));
    }

    #[test]
    fn sans_cache_gpu_vram_indisponible_hors_apple_silicon() {
        clear_gpu_vram_cache();
        let snap = build_snapshot(false, 5.0, 1000, 8000, 12.5, None);
        assert!(!snap.vram_available);
        assert!(snap.vram_used_percent.is_none());
    }
}
