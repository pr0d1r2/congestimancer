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
    nix-hk.url = "github:pr0d1r2/nix-hk";
    nix-hk.inputs.nixpkgs-lock.follows = "nixpkgs-lock";

    # Spec form and spec operations (`mth`).
    microlith.url = "github:pr0d1r2/microlith/v0.7.4";
    microlith.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    microlith.inputs.nix-hk.follows = "nix-hk";
    # Context limits (`itok check`).
    itok.url = "github:pr0d1r2/itok/v0.3.1";
    itok.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    itok.inputs.hk.follows = "nix-hk";
    # SPEC-DAG federation, budgets, waves.
    sherd.url = "github:pr0d1r2/sherd/v0.5.4";
    sherd.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    sherd.inputs.nix-hk.follows = "nix-hk";
    # Allowed characters per file type (`ctrm`).
    characterminator.url = "github:pr0d1r2/characterminator/v0.1.1";
    characterminator.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    characterminator.inputs.nix-hk.follows = "nix-hk";
    # Hook guardrails materialised from the repository's files.
    pklith.url = "github:pr0d1r2/pklith/v0.1.0";
    pklith.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    pklith.inputs.nix-hk.follows = "nix-hk";
    # Rules extracted from agent memory, and the honesty gate on them.
    rekall.url = "github:pr0d1r2/rekall/v0.4.0";
    rekall.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
    rekall.inputs.nix-hk.follows = "nix-hk";
    # One language per file, 1:1 test mirror, TDD order (`xnl`).
    xenolith.url = "github:pr0d1r2/xenolith/v0.1.0";
    xenolith.inputs.nixpkgs-lock.follows = "nixpkgs-lock";
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
    in
    {
      devShells = forAll (
        pkgs:
        let
          system = pkgs.stdenv.hostPlatform.system;
          absent = missing system;
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.rustc
              pkgs.cargo
              pkgs.clippy
              pkgs.rustfmt
              pkgs.git
              # The gate runner; its ops live in `hk.pkl`.
              pkgs.hk
              # `nixfmt --check` gates this file.
              pkgs.nixfmt
            ]
            ++ present system;
            # Deterministic tool output across machines.
            LANG = "C.UTF-8";
            shellHook = pkgs.lib.optionalString (absent != [ ]) ''
              echo "congestimancer(shell): no ${system} build of: ${builtins.concatStringsSep ", " absent} -- gates that need them FAIL here" >&2
            '';
          };
        }
      );

      formatter = forAll (pkgs: pkgs.nixfmt);
    };
}
