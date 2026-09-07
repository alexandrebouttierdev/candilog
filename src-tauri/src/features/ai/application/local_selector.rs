//! Sélection conservatrice du modèle et du backend à partir du matériel détecté.

use crate::features::ai::domain::{
    LocalAiBackend, LocalAiHardware, LocalAiRecommendation, LocalModelDefinition,
    LocalModelEvaluation, LocalModelProfile, ModelCompatibility, ModelRegistry,
};

const CPU_MEMORY_PERCENT: u64 = 68;
const GPU_MEMORY_PERCENT: u64 = 80;

pub struct LocalModelSelector;

impl LocalModelSelector {
    #[must_use]
    pub fn recommend(hardware: LocalAiHardware) -> LocalAiRecommendation {
        let backend = preferred_backend(&hardware);
        let mut models = ModelRegistry::all();
        models.reverse();

        let mut selected = None;
        let mut evaluations = Vec::with_capacity(models.len());
        for model in models {
            let (comfortable, can_load, reason) = compatibility_for(&hardware, backend, &model);
            let compatibility = if comfortable && selected.is_none() {
                selected = Some(model.clone());
                ModelCompatibility::Optimal
            } else if comfortable {
                ModelCompatibility::Supported
            } else if can_load {
                ModelCompatibility::NotRecommended
            } else {
                ModelCompatibility::Unsupported
            };
            evaluations.push(LocalModelEvaluation {
                model,
                compatibility,
                reason,
            });
        }

        let reason = selected.as_ref().map_or_else(
            || "Aucun profil ne dispose d'une marge mémoire et CPU suffisante pour garantir une utilisation fluide.".into(),
            |model| format!("Le profil {} conserve une marge confortable pour Candilog et le système.", profile_label(model.profile)),
        );
        LocalAiRecommendation {
            hardware,
            selected_model: selected,
            backend,
            evaluations,
            reason,
        }
    }
}

fn preferred_backend(hardware: &LocalAiHardware) -> LocalAiBackend {
    if hardware.apple_silicon && hardware.metal {
        return LocalAiBackend::Metal;
    }
    let best = hardware
        .gpus
        .iter()
        .filter_map(|gpu| gpu.backend.zip(gpu.total_vram_mb))
        .max_by_key(|(_, memory)| *memory)
        .map(|(backend, _)| backend);
    best.unwrap_or(LocalAiBackend::Cpu)
}

fn compatibility_for(
    hardware: &LocalAiHardware,
    backend: LocalAiBackend,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    if hardware.apple_silicon {
        return unified_memory_compatibility(hardware, model);
    }
    if backend != LocalAiBackend::Cpu {
        return gpu_compatibility(hardware, backend, model);
    }
    cpu_compatibility(hardware, model)
}

fn unified_memory_compatibility(
    hardware: &LocalAiHardware,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let total = hardware.unified_memory_mb.unwrap_or(hardware.total_ram_mb);
    let budget = total.saturating_mul(CPU_MEMORY_PERCENT) / 100;
    let available = hardware.available_ram_mb;
    let comfortable = total >= model.recommended_ram_mb
        && budget >= model.estimated_ram_mb
        && available >= model.estimated_ram_mb;
    let can_load = budget >= model.estimated_ram_mb;
    (
        comfortable,
        can_load,
        format!(
            "{} de mémoire unifiée, {} disponibles ; {} recommandés.",
            format_go(total),
            format_go(available),
            format_go(model.recommended_ram_mb)
        ),
    )
}

fn gpu_compatibility(
    hardware: &LocalAiHardware,
    backend: LocalAiBackend,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let gpu = hardware
        .gpus
        .iter()
        .filter(|gpu| gpu.backend == Some(backend))
        .max_by_key(|gpu| gpu.total_vram_mb.unwrap_or_default());
    let total = gpu.and_then(|gpu| gpu.total_vram_mb).unwrap_or_default();
    let available = gpu.and_then(|gpu| gpu.available_vram_mb).unwrap_or(total);
    let budget = total.saturating_mul(GPU_MEMORY_PERCENT) / 100;
    let load_estimate = model.download_size_bytes / 1_048_576 + 768;
    let recommended = model
        .recommended_vram_mb
        .unwrap_or(model.recommended_ram_mb);
    let comfortable = total >= recommended && budget >= load_estimate && available >= load_estimate;
    let can_load = budget >= load_estimate;
    (
        comfortable,
        can_load,
        format!(
            "{} de mémoire graphique, {} disponibles ; {} recommandés.",
            format_go(total),
            format_go(available),
            format_go(recommended)
        ),
    )
}

fn cpu_compatibility(
    hardware: &LocalAiHardware,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let total = hardware.total_ram_mb;
    let available = hardware.available_ram_mb;
    let budget = total.saturating_mul(CPU_MEMORY_PERCENT) / 100;
    let physical = hardware.physical_cores.unwrap_or(hardware.logical_cores);
    let minimum_cores = model.recommended_cores;
    let comfortable = total >= model.recommended_ram_mb
        && budget >= model.estimated_ram_mb
        && available >= model.estimated_ram_mb
        && physical >= minimum_cores;
    let can_load = budget >= model.estimated_ram_mb;
    (
        comfortable,
        can_load,
        format!(
            "{} de RAM, {} disponibles et {} cœurs physiques ; {} et {} cœurs recommandés.",
            format_go(total),
            format_go(available),
            physical,
            format_go(model.recommended_ram_mb),
            minimum_cores
        ),
    )
}

const fn profile_label(profile: LocalModelProfile) -> &'static str {
    match profile {
        LocalModelProfile::UltraLight => "Ultra léger",
        LocalModelProfile::Light => "Léger",
        LocalModelProfile::Balanced => "Équilibré",
        LocalModelProfile::Quality => "Qualité",
    }
}

/// Convertit une quantité interne en Mo vers un libellé en Go lisible.
///
/// Les seuils du registre sont en multiples de 1024 Mo ; on garde cette base pour que
/// « 4096 Mo recommandés » s'affiche « 4 Go », pas « 4,1 Go ».
fn format_go(mb: u64) -> String {
    let go = mb as f64 / 1024.0;
    let tenths = (go * 10.0).round() / 10.0;
    if (tenths - tenths.round()).abs() < f64::EPSILON {
        format!("{} Go", tenths.round() as u64)
    } else {
        format!("{tenths:.1} Go").replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ai::domain::LocalGpuInfo;

    fn cpu(total_ram_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            os: "linux".into(),
            architecture: "x86_64".into(),
            cpu: "x86_64".into(),
            cpu_model: "CPU de test".into(),
            logical_cores: 16,
            physical_cores: Some(8),
            total_ram_mb,
            available_ram_mb: total_ram_mb,
            gpus: vec![],
            apple_silicon: false,
            soc_model: None,
            unified_memory_mb: None,
            metal: false,
            cuda: false,
            vulkan: false,
        }
    }

    fn apple(memory_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            apple_silicon: true,
            unified_memory_mb: Some(memory_mb),
            metal: true,
            os: "macos".into(),
            architecture: "aarch64".into(),
            ..cpu(memory_mb)
        }
    }

    fn gpu(vram_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            cuda: true,
            gpus: vec![LocalGpuInfo {
                name: "GPU de test".into(),
                vendor: "NVIDIA".into(),
                total_vram_mb: Some(vram_mb),
                available_vram_mb: Some(vram_mb),
                backend: Some(LocalAiBackend::Cuda),
            }],
            ..cpu(32_768)
        }
    }

    fn selected(hardware: LocalAiHardware) -> Option<LocalModelProfile> {
        LocalModelSelector::recommend(hardware)
            .selected_model
            .map(|model| model.profile)
    }

    #[test]
    fn cpu_thresholds() {
        assert_eq!(selected(cpu(4_096)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(cpu(8_192)), Some(LocalModelProfile::Light));
        assert_eq!(selected(cpu(16_384)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(cpu(24_576)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(cpu(4_095)), None);
        assert_eq!(selected(cpu(8_191)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(cpu(16_383)), Some(LocalModelProfile::Light));
        assert_eq!(selected(cpu(24_575)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(cpu(24_577)), Some(LocalModelProfile::Quality));
    }

    #[test]
    fn apple_silicon_thresholds() {
        assert_eq!(selected(apple(4_096)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(apple(8_192)), Some(LocalModelProfile::Light));
        assert_eq!(selected(apple(16_384)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(apple(24_576)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(apple(4_095)), None);
        assert_eq!(selected(apple(8_191)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(apple(16_383)), Some(LocalModelProfile::Light));
        assert_eq!(selected(apple(24_575)), Some(LocalModelProfile::Balanced));
    }

    #[test]
    fn dedicated_gpu_thresholds() {
        assert_eq!(selected(gpu(2_560)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(gpu(4_096)), Some(LocalModelProfile::Light));
        assert_eq!(selected(gpu(8_192)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(gpu(12_288)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(gpu(2_559)), None);
        assert_eq!(selected(gpu(4_095)), Some(LocalModelProfile::UltraLight));
        assert_eq!(selected(gpu(8_191)), Some(LocalModelProfile::Light));
        assert_eq!(selected(gpu(12_287)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(gpu(12_289)), Some(LocalModelProfile::Quality));
    }

    #[test]
    fn low_end_machine_has_no_automatic_model() {
        assert_eq!(selected(cpu(3_000)), None);
    }

    #[test]
    fn slow_cpu_does_not_select_a_heavy_model_only_because_it_fits() {
        let mut hardware = cpu(24_576);
        hardware.physical_cores = Some(4);
        assert_eq!(selected(hardware), Some(LocalModelProfile::Light));
    }

    /// Les raisons sont lues telles quelles dans l'interface : afficher 15 559 Mo n'aide
    /// pas à comparer avec un seuil de 4 Go. On convertit toujours en Go (base 1024).
    #[test]
    fn format_go_arrondit_et_utilise_la_virgule_francaise() {
        assert_eq!(format_go(4_096), "4 Go");
        assert_eq!(format_go(2_560), "2,5 Go");
        assert_eq!(format_go(15_559), "15,2 Go");
        assert_eq!(format_go(6_954), "6,8 Go");
    }

    #[test]
    fn les_raisons_dexvaluation_affichent_la_memoire_en_go() {
        let mut hardware = cpu(15_559);
        hardware.available_ram_mb = 6_954;
        hardware.physical_cores = Some(4);
        let reason = LocalModelSelector::recommend(hardware)
            .evaluations
            .into_iter()
            .find(|item| item.model.profile == LocalModelProfile::UltraLight)
            .expect("profil ultra léger évalué")
            .reason;
        assert!(
            reason.contains("15,2 Go de RAM") && reason.contains("6,8 Go disponibles"),
            "raison inattendue : {reason}"
        );
        assert!(
            reason.contains("4 Go") && reason.contains("2 cœurs recommandés"),
            "raison inattendue : {reason}"
        );
        assert!(
            !reason.contains(" Mo"),
            "plus aucun Mo dans la raison : {reason}"
        );
    }
}
