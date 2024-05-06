{
  description = "Real-time racing game zoop";

  # Dependencies for building everything in the flake
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    # Build outputs for each default system
    # by default that is ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"]
    flake-utils.lib.eachDefaultSystem (system:
      let 
        # Alias for the source of build dependencies (specifically nixpkgs)
        pkgs = nixpkgs.legacyPackages.${system};
        rustPlatform = pkgs.rustPlatform;
        buildRustPackage = rustPlatform.buildRustPackage;
        version = "0.1.0";

        # Zoop engine build
        cargoBuild = buildRustPackage {
          pname = "zoopRacerEngine";
          version = version;

          src = ./.;

          nativeBuildInputs = [
            # alsa-sys crate)
            pkgs.pkg-config
          ];

          # Mostly dependencies for engine (Bevy) and frontend server (Tauri)
          buildInputs = [
            # alsa-sys crate
            pkgs.alsa-lib
            # libudev-sys crate
            pkgs.systemd # dbus is supposedly baked into systemd
            # glib-sys crate
            pkgs.dbus-glib
            # soup2-sys crate
            pkgs.libsoup
            # gdk-sys crate
            pkgs.gtk3
            # openssl-sys crate
            pkgs.openssl
            # javascriptcore-rs-sys crate
            pkgs.webkitgtk
          ];

          cargoHash = "sha256-EfD+JS8PE8y/TqwYvAItTWRWDmzF/HgBRGhqLb2JHHg=";

          meta = {
            description = "Racer engine";
          };
        };

        # Docker image for backend
        zoopRacerBackend = pkgs.dockerTools.buildImage {
          name = "zoopRacerBackend";
          tag = version;

          config = { Cmd = [ "${pkgs.hello}/bin/hello" ]; };

          created = "now";
        };

      in {
        packages.zoopRacerBackend = zoopRacerBackend;
        # packages.zoopRacerEngine = zoopRacerEngine;

        devShells.default =
          pkgs.mkShell { buildInputs = with pkgs; [ bat vim ]; };
        
        defaultPackage = cargoBuild;

        checks = {
          inherit zoopRacerBackend;
        };
      });
}
