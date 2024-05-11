{
  description = "Real-time racing game zoop";

  # Another interesting way to debug the flake might be by using the nix repl
  # $ nix repl 
  # > zoop = builtins.getFlake "./." # Load the flake from root repo
  # > zoop.<tab>

  # Dependencies for building everything in the flake
  inputs = {
    # All packages in the nix repo
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    # Utilities for building nix flakes for multiple architectures
    flake-utils.url = "github:numtide/flake-utils";
    # Helper for building Rust packages with nix
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    # Build outputs for each default system
    # by default that is ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"]
    flake-utils.lib.eachDefaultSystem (system:
      let 
        # Build parameters
        version = "0.1.0";
        cargoBuildType = "release"; # cargo build --release or --debug

        # Aliases
        pkgs = nixpkgs.legacyPackages.${system};

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
        cliBuild = buildRustPackage {
          pname = "zoop_cli";
          src = ./.;
          extraPrefix = "/cli";
          buildType = cargoBuildType;
          version = version;
          cargoSha256 = "sha256-xRCIFr/7MnfFL7vpgO6OhKPWr4WhD8talfKOqB/AtqI=";
          nativeBuildInputs = engineBuildDependencies;
          buildInputs = engineLinkedDependencies;
          buildAndTestSubdir = "zoop_cli";
          extraOutputsToInstall = ["zoop_cli/assets"];
        };

        # Build with engine WASM output
        engineWasmBuild = buildRustPackage {
          pname = "zoop_engine";
          src = ./.;
          extraPrefix = "/engine";
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
              wasm-pack build --mode no-install ./zoop_engine --target web --${cargoBuildType} --out-dir $out
            )
            runHook postBuild
          '';
          installPhase = ":";
        };

        # Build which contains all individual builds
        allBuilds = pkgs.buildEnv {
          name = "zoop_all";
          paths = [
            assetBuild
            # cliBuild
            engineWasmBuild
          ];
        };
      in {
        # Export all built packages
        packages.zoopAssets = assetBuild;
        packages.zoopCli = cliBuild;
        packages.zoopEngine = engineWasmBuild;
        packages.zoopAll = allBuilds;

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
