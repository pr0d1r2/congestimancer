# SPEC

## §G GOAL

`src/gpu` — GPU utilisation & free memory gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/gpu|GPU utilisation & free memory

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`. NVML loaded at runtime ∴ ⊥ link-time dep.

## §I INTERFACES

- switches: `--gpu X` · `--gpu-mem X` · `--gpu-device I`.
- sources: Linux NVML (NVIDIA), sysfs `gpu_busy_percent` & `mem_info_vram_total|used` (AMD) · macOS IOKit `PerformanceStatistics`.

## §V INVARIANTS

V1: `--gpu X` ⇔ utilisation % < X.
V2: `--gpu-mem X` ⇔ free GPU memory > X.
V3: default = ∀ GPUs, worst wins; `--gpu-device I` selects index; unknown I → usage 64.
V4: no GPU source → `Unsupported`.

## §T TASKS

id|status|task|cites
T1|.|AMD sysfs parser|V1,V2
T2|.|NVML reads|V1,V2,V4
T3|.|macOS IOKit GPU stats|V1,V2,V4
T4|.|device selection & worst-wins|V3

## §B BUGS

id|date|cause|fix
