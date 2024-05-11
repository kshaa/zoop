{
  description = "Real-time racing game zoop";

  # Dependencies for building everything in the flake
  inputs = {
    # All packages in the nix repo
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    # Utilities for building nix flakes for multiple architectures
    flake-utils.url = "github:numtide/flake-utils";
    # Helper for creating custom Rust toolchains
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Helper for building Node packages
    nixNpmBuildPackage.url = "github:serokell/nix-npm-buildpackage";
    # Helper for caching (avoid re-building by ignoring gitignored files)
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, flake-utils, nixNpmBuildPackage, gitignore }:
    # Build outputs for each default system
    # by default that is ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"]
    flake-utils.lib.eachDefaultSystem (system:
      let 
        # Build parameters
        version = "0.1.0";
        cargoBuildType = "release"; # cargo build --release or --debug
        conciseBuild = true; # if false, add some extra artifacts for debugging

        # System packages
        pkgs = nixpkgs.legacyPackages.${system};

        # Source reading with filters aliases
        cleanSourceWith = pkgs.lib.cleanSourceWith;
        gitignoreFilterWith = gitignore.lib.gitignoreFilterWith;
        sourceFilter = src: gitignoreFilterWith {
          basePath = src;
          extraRules = ''
            flake.nix
            README.md
          '';
        };
        gitignoreSource' = src: cleanSourceWith {
          filter = sourceFilter src;
          src = src;
        };

        # Rust build tools
        cargo = pkgs.cargo;
        rustc = pkgs.cargo;
        fenixSystem = fenix.packages.${system};
        fenixPkgs = fenix.inputs.nixpkgs.legacyPackages.${system};
        toolchain = fenixSystem.combine [
          # Default Rust tools
          fenixSystem.stable.cargo
          fenixSystem.stable.clippy
          fenixSystem.stable.rust-src
          fenixSystem.stable.rustc
          fenixSystem.stable.rustfmt
          # Needed by engine WASM build
          fenixSystem.targets.wasm32-unknown-unknown.stable.rust-std
        ];
        rustPlatform = pkgs.makeRustPlatform {
          inherit toolchain cargo rustc;
        };
        buildRustPackage = rustPlatform.buildRustPackage;

        # NodeJS build tools
        bp = pkgs.callPackage nixNpmBuildPackage {};
        buildNpmPackage = bp.buildNpmPackage;

        # Engine compile-time dependencies
        engineBuildDependencies = [
          # needed by many crates
          pkgs.pkg-config
          # needed by engine WASM build
          toolchain 
          # needed by engine WASM build
          fenixPkgs.wasm-pack 
          # needed by engine WASM build
          fenixPkgs.wasm-bindgen-cli
        ];

        # Engine runtime / dynamically-linked dependencies
        # i.e. Bevy dependencies
        engineLinkedDependencies = [
          # alsa-sys crate
          pkgs.alsa-lib
          # libudev-sys crate
          pkgs.systemd # dbus is supposedly baked into systemd
          # openssl-sys crate
          pkgs.openssl
        ];

        # Frontend compile-time dependencies
        frontendBuildDependencies = engineBuildDependencies ++ [
          # Frontend build tool
          fenixPkgs.cargo-tauri
        ];
        
        # Frontend runtime / dynamically-linked dependencies
        # i.e. Tauri dependencies
        frontendLinkedDependencies = engineLinkedDependencies ++ [
          # Build tool
          fenixPkgs.cargo-tauri
          # glib-sys crate
          pkgs.dbus-glib
          # soup2-sys crate
          pkgs.libsoup
          # gdk-sys crate
          pkgs.gtk3
          # javascriptcore-rs-sys crate
          pkgs.webkitgtk
        ];

        # Build with engine assets
        assetBuild = pkgs.buildEnv {
          name = "zoop_assets";
          extraPrefix = "/assets";
          paths = [
            ./zoop_cli/assets
          ];
        };

        # Build with CLI tool (used either manually or by frontend in native mode)
        cliBuildUnprefixed = buildRustPackage {
          pname = "zoop_cli";
          src = gitignoreSource' ./.;
          extraPrefix = "/cli";
          buildType = cargoBuildType;
          version = version;
          cargoSha256 = "sha256-4AUlVsEhOqzm8oMXNbP2Qs4ZktVuZTw1+W7p0YRCYv8=";
          nativeBuildInputs = engineBuildDependencies;
          buildInputs = engineLinkedDependencies;
          buildAndTestSubdir = "zoop_cli";
          extraOutputsToInstall = ["zoop_cli/assets"];
        };
        cliBuild = pkgs.stdenv.mkDerivation {
          name = "cli";
          src = cliBuildUnprefixed;
          installPhase = ''
            mkdir -p $out/cli
            cp -rfL bin/. $out/cli/
          '';
        };

        # Build with engine WASM output
        engineWasmBuild = buildRustPackage {
          pname = "zoop_engine";
          src = gitignoreSource' ./.;
          buildType = cargoBuildType;
          version = version;
          cargoSha256 = "sha256-HMxtqjLuro6Z96IOJLwqcNBVNPRerRYbWmPinef6mAU=";
          nativeBuildInputs = engineBuildDependencies;
          buildInputs = engineLinkedDependencies;
          buildAndTestSubdir = "zoop_engine";
          WASM_PACK_CACHE = ".wasm-pack-cache";
          RUST_LOG = "debug";
          RUSTFLAGS = "--cfg=web_sys_unstable_apis";
          dontCargoBuild = true;
          buildPhase = ''
            runHook preBuild
            (
              set -x
              mkdir -p $out/engine
              wasm-pack build --mode no-install ./zoop_engine --target web --${cargoBuildType} --out-dir $out/engine
            )
            runHook postBuild
          '';
          installPhase = ":";
        };

        # Build with NextJS standalone server
        webBuildUnprefixed = buildNpmPackage {
          src = gitignoreSource' ./zoop_web;
          npmBuild = ''
            # Copy WASM engine
            cp -rf "${engineWasmBuild}/engine/." "./public/"
            cp -rf "${engineWasmBuild}/engine/." "./src/services/"
            # Copy assets
            cp -rf "${assetBuild}/assets/." "./public/"
            # Build server
            npm run build
            # Move "dist" to a non-dotfile
            mv .next next_build
          '';
        };
        webBuildFull = pkgs.buildEnv {
          name = "web_full";
          extraPrefix = "/web_full";
          paths = [
            webBuildUnprefixed
          ];
        };
        webBuildStandalone = pkgs.stdenv.mkDerivation {
          name = "web";
          src = webBuildUnprefixed;
          installPhase = ''
            mkdir -p $out/web/.next/static
            cp -rfL next_build/standalone/. $out/web/
            cp -rfL next_build/static/. $out/web/.next/static/
          '';
        };

        # Build which contains all individual builds
        allBuilds = pkgs.buildEnv {
          name = "zoop_all";
          paths = [
            cliBuild
            webBuildStandalone
          ] ++ pkgs.lib.optionals (!conciseBuild) [
            assetBuild
            engineWasmBuild
            webBuildFull
          ];
        };
      in {
        # Export all built packages
        packages.assets = assetBuild;
        packages.cli = cliBuild;
        packages.engine = engineWasmBuild;
        packages.webStandalone = webBuildStandalone;
        packages.webFull = webBuildStandalone;
        packages.all = allBuilds;

        # Dev shell with tools for building manually
        devShells.default =
          pkgs.mkShell { 
            buildInputs = 
              engineBuildDependencies ++ 
              engineLinkedDependencies ++ 
              frontendBuildDependencies ++ 
              frontendLinkedDependencies ++ [ 
                pkgs.git
                pkgs.vim
              ]; 
          };
        
        # Default package (result of nix build)
        defaultPackage = allBuilds;

        # Run tests (result of nix flake check)
        checks = {
          inherit allBuilds;
        };
      });
}
