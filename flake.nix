{
  description = "congestimancer -- gate commands on machine congestion (dev shell)";

  # hk and the fleet tools are pushed to this cache. Without the substituter
  # every entry into the dev shell builds them from source.
  nixConfig = {
    extra-substituters = [ "https://pr0d1r2.cachix.org" ];
    extra-trusted-public-keys = [
      "pr0d1r2.cachix.org-1:NfWjbhgAj41byXhCKiaE+av3Vnphm1fTezHXEGsiQIM="
    ];
  };

  # ONE nixpkgs: `nixpkgs-lock` (nixos-26.05, rustc 1.95.0) and every other
  # input follows it. The guardrail tools are pinned to TAGS, bumped each in
  # its own reviewed commit (`nix:V9`).
  inputs = {
    nixpkgs-lock.url = "github:pr0d1r2/nixpkgs-lock";
    nixpkgs.follows = "nixpkgs-lock/nixpkgs";
    nix-hk = {
      url = "github:pr0d1r2/nix-hk";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    };

    # Spec form and spec operations (`mth`).
    microlith = {
      url = "github:pr0d1r2/microlith/v0.7.4";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.nix-hk.follows = "nix-hk";
    };
    # Context limits (`itok check`).
    itok = {
      url = "github:pr0d1r2/itok/v0.3.1";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.hk.follows = "nix-hk";
    };
    # SPEC-DAG federation, budgets, waves.
    sherd = {
      url = "github:pr0d1r2/sherd/v0.5.4";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.nix-hk.follows = "nix-hk";
    };
    # Allowed characters per file type (`ctrm`).
    characterminator = {
      url = "github:pr0d1r2/characterminator/v0.1.1";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.nix-hk.follows = "nix-hk";
    };
    # Hook guardrails materialised from the repository's files.
    pklith = {
      url = "github:pr0d1r2/pklith/v0.1.0";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.nix-hk.follows = "nix-hk";
    };
    # Rules extracted from agent memory, and the honesty gate on them.
    rekall = {
      url = "github:pr0d1r2/rekall/v0.4.0";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
      inputs.nix-hk.follows = "nix-hk";
    };
    # One language per file, 1:1 test mirror, TDD order (`xnl`).
    xenolith = {
      url = "github:pr0d1r2/xenolith/v0.1.0";
      inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    };
  };

  outputs =
    {
      nixpkgs,
      nix-hk,
      microlith,
      itok,
      sherd,
      characterminator,
      pklith,
      rekall,
      xenolith,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      # The overlay makes `pkgs.hk` nix-hk's hk rather than nixpkgs'.
      forAll =
        f: nixpkgs.lib.genAttrs systems (s: f (nixpkgs.legacyPackages.${s}.extend nix-hk.overlays.default));

      tools = {
        inherit
          microlith
          itok
          sherd
          characterminator
          pklith
          rekall
          xenolith
          ;
      };

      # Tools with no working build on a system, MEASURED 2026-10-14 by
      # evaluating each input's `packages.<system>.default`. rekall lists
      # x86_64-darwin but its build needs inputs that do not, so an `or`
      # fallback cannot catch it: the list is explicit. The shell still
      # evaluates there; the missing tools are named on entry, and every
      # gate that needs one FAILS rather than skips (`nix:V2`).
      unavailable = {
        x86_64-darwin = [
          "microlith"
          "itok"
          "characterminator"
          "pklith"
          "rekall"
          "xenolith"
        ];
      };
      toolFor =
        system: name:
        if builtins.elem name (unavailable.${system} or [ ]) then
          null
        else
          tools.${name}.packages.${system}.default;
      present = system: builtins.filter (p: p != null) (map (toolFor system) (builtins.attrNames tools));
      missing = system: unavailable.${system} or [ ];
      # Systems pklith's `lib.devShell` supports.
      pklithSystems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      devShells = forAll (
        pkgs:
        let
          system = pkgs.stdenv.hostPlatform.system;
          absent = missing system;
          coverageLlvm = pkgs.llvmPackages_21.llvm;
          coverageEnv = {
            LLVM_COV = "${coverageLlvm}/bin/llvm-cov";
            LLVM_PROFDATA = "${coverageLlvm}/bin/llvm-profdata";
          };
          toolchain = [
            pkgs.rustc
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            # Dependency policy gate (deny.toml, nix:V10).
            pkgs.cargo-deny
            # Releases (release.toml, nix:T11).
            pkgs.cargo-release
            # Line coverage (nix:V5); the llvm that reads its data must match
            # the LLVM major rustc was built against (21 for rustc 1.95).
            pkgs.cargo-llvm-cov
            coverageLlvm
            # `nixfmt --check` gates this file.
            pkgs.nixfmt
          ]
          ++ present system;
          warn = pkgs.lib.optionalString (absent != [ ]) ''
            echo "congestimancer(shell): no ${system} build of: ${builtins.concatStringsSep ", " absent} -- gates that need them FAIL here" >&2
          '';
        in
        {
          # pklith's shell adds hk, pkl, git, pklith and every tool its
          # generated `nix/pklith.nix` names, and points git at the tracked
          # `.githooks/` once `hk.pkl` exists (`nix:V15`). pklith has no
          # x86_64-darwin build, so that system gets a plain shell without
          # the hook wiring, and the gate cannot run there (`nix:V2`).
          default =
            if builtins.elem system pklithSystems then
              (pklith.lib.devShell {
                inherit pkgs;
                src = ./.;
                packages = toolchain;
              }).overrideAttrs
                (old: {
                  # Our pinned tools FIRST on PATH: pklith's catalog carries
                  # its own (older) microlith, sherd and itok, and mkShell puts
                  # `packages` after them.
                  nativeBuildInputs = toolchain ++ old.nativeBuildInputs;
                  # Deterministic tool output across machines.
                  LANG = "C.UTF-8";
                  inherit (coverageEnv) LLVM_COV LLVM_PROFDATA;
                  shellHook = old.shellHook + warn;
                })
            else
              pkgs.mkShell {
                packages = toolchain ++ [
                  pkgs.hk
                  pkgs.git
                ];
                LANG = "C.UTF-8";
                inherit (coverageEnv) LLVM_COV LLVM_PROFDATA;
                shellHook = warn;
              };
        }
      );

      formatter = forAll (pkgs: pkgs.nixfmt);
    };
}
