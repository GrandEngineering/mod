{
  description = "Build a cargo project without extra checks";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";
    engine.url = "github:GrandEngineering/engine";
    engine.flake = false;
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    engine,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      # NB: we don't need to overlay our custom toolchain for the *entire*
      # pkgs (which would require rebuidling anything else which uses rust).
      # Instead, we just want to update the scope that crane will use by appending
      # our specific toolchain there.
      craneLib = (crane.mkLib pkgs).overrideToolchain (p:
        p.rust-bin.stable.latest.default.override {
          targets = ["x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu"];
        });

      my-crate = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;

        cargoExtraArgs = "--target x86_64-unknown-linux-gnu";

        # Tests currently need to be run via `cargo wasi` which
        # isn't packaged in nixpkgs yet...
        doCheck = false;

        buildInputs =
          [
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
      };
    in {
      checks = {
        inherit my-crate;
      };

      packages.default = my-crate;

      # apps.default = flake-utils.lib.mkApp {
      #   drv = pkgs.writeShellScriptBin "my-app" ''
      #     ${pkgs.wasmtime}/bin/wasmtime run ${my-crate}/bin/custom-toolchain.wasm
      #   '';
      # };

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        # Extra inputs can be added here; cargo and rustc are provided by default
        # from the toolchain that was specified earlier.
        packages = [
        ];
      };
    });
}
