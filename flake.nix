{
  description = "Rust project with library and binary components";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    binary-src = {
      url = "github:GrandEngineering/engine/main";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    binary-src,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      crateName = cargoToml.package.name;
      toolchain = with fenix.packages.${system};
        combine [
          minimal.rustc
          minimal.cargo
          targets.x86_64-pc-windows-gnu.latest.rust-std
        ];

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
        doCheck = false;
        nativeBuildInputs = [pkgs.git];
        cargoAllowGemlockFailure = true;
        cargoGitCommand = "${pkgs.git}/bin/git";
      };

      # Build for Linux
      linuxLib = craneLib.buildPackage (commonArgs
        // {
          pname = "linux-lib";
          version = "1.0.0";

          postInstall = ''
            mkdir -p $out
            cp target/release/lib${crateName}.so $out/mod.so
          '';
        });

      # Build for Windows
      windowsLib = craneLib.buildPackage (commonArgs
        // {
          pname = "windows-lib";
          version = "1.0.0";

          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
          TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            pkgsCross.mingwW64.windows.pthreads
          ];

          postInstall = ''
            mkdir -p $out
            cp target/x86_64-pc-windows-gnu/release/${crateName}.dll $out/mod.dll
          '';
        });

      # Create the archive
      libArchive = pkgs.stdenv.mkDerivation {
        name = "${crateName}.rustforge";
        src = ./.; # Add this line to provide a source
        nativeBuildInputs = [pkgs.gnutar];

        buildPhase = ''
          mkdir -p build
          cp ${linuxLib}/mod.so build/
          cp ${windowsLib}/mod.dll build/
          cd build
          ${pkgs.gnutar}/bin/tar -cf mod.rustforge.tar *
          cp mod.rustforge.tar ..
          cd ..
        '';

        installPhase = ''
          mkdir -p $out
          cp mod.rustforge.tar $out/
        '';
      };

      # Binary compilation
      rustBinary = craneLib.buildPackage {
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        src = binary-src;
        doCheck = false;
        nativeBuildInputs = [pkgs.git pkgs.protobuf];
      };

      # Create wrapper scripts
      serverWrapper = pkgs.writeShellScriptBin "server-wrapper" ''
        TMPDIR=$(mktemp -d)
        trap 'rm -rf "$TMPDIR"' EXIT
        cd "$TMPDIR"
        cp ${libArchive}/mod.rustforge.tar .
        LD_LIBRARY_PATH=$TMPDIR exec ${rustBinary}/bin/server "$@"
      '';

      clientWrapper = pkgs.writeShellScriptBin "client-wrapper" ''
        TMPDIR=$(mktemp -d)
        trap 'rm -rf "$TMPDIR"' EXIT
        cd "$TMPDIR"
        cp ${libArchive}/mod.rustforge.tar .
        LD_LIBRARY_PATH=$TMPDIR exec ${rustBinary}/bin/client "$@"
      '';
    in {
      packages = {
        default = libArchive;
      };

      apps = {
        default = {
          type = "app";
          program = "${serverWrapper}/bin/server-wrapper";
        };

        server = self.apps.${system}.default;

        client = {
          type = "app";
          program = "${clientWrapper}/bin/client-wrapper";
        };
      };

      devShells.default = craneLib.devShell {
        packages = with pkgs; [
          git
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];
      };
    });
}
