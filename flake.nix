{
  description = "Simple flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flakebox = {
      url = "github:rustshop/flakebox?rev=12d5ee4f6c47bc01f07ec6f5848a83db265902d3";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakebox, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = pkgs.lib;

        flakeboxLib = flakebox.lib.${system} {
          config = {
          };
        };

        toolchainArgs = { };

        # all standard toolchains provided by flakebox
        toolchainsStd =
          flakeboxLib.mkStdFenixToolchains toolchainArgs;

        rustSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = "fedimint-dkg-cli-standalone";
            path = ./.;
          };
          paths = [
            "Cargo.toml"
            "Cargo.lock"
            ".cargo"
            "src"
          ];
        };

        outputs =
          (flakeboxLib.craneMultiBuild { toolchains = toolchainsStd; })
            (craneLib':
              let
                craneLib = (craneLib'.overrideArgs {
                  pname = "fedimint-dkg-cli-standalone";
                  src = rustSrc;
                  buildInputs = [
                    pkgs.protobuf
                    pkgs.openssl
                  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                  ];
                  nativeBuildInputs = [
                    pkgs.pkg-config
                  ];

                  PROTOC = "${pkgs.protobuf}/bin/protoc";
                  PROTOC_INCLUDE = "${pkgs.protobuf}/include";
                });
              in
              rec {
                workspaceDeps = craneLib.buildWorkspaceDepsOnly { };
                workspaceBuild = craneLib.buildWorkspace {
                  cargoArtifacts = workspaceDeps;
                };
                workspaceClippy = craneLib.cargoClippy {
                  cargoArtifacts = workspaceDeps;

                  cargoClippyExtraArgs = "--all-targets --no-deps -- --deny warnings --allow deprecated";
                  doInstallCargoArtifacts = false;
                };

                ciAllTests = craneLib.mkCargoDerivation {
                  pname = "ciAllTests";
                  cargoArtifacts = workspaceDeps;
                  nativeBuildInputs = [
                  ];
                  buildPhaseCargoCommand = "patchShebangs ./scripts ; ./scripts/ci-test-all.sh";
                };

                fedimint-dkg-cli-standalone =
                  craneLib.buildPackageGroup
                    { pname = "fedimint-dkg-cli-standalone"; packages = [ "fedimint-dkg-cli-standalone" ]; mainProgram = "fedimint-dkg-cli-standalone"; };

                fedimint-dkg-cli-standalone-image = pkgs.dockerTools.buildLayeredImage {
                  name = "fedimint-dkg-cli-standalone";
                  contents = [ fedimint-dkg-cli-standalone pkgs.bash pkgs.coreutils pkgs.cacert pkgs.curl ];
                  config = {
                    Cmd = [
                      "${fedimint-dkg-cli-standalone}/bin/fedimint-dkg-cli-standalone"
                    ];
                  };
                };
              });
      in
      {
        legacyPackages = outputs;
        packages = {
          fedimint-dkg-cli-standalone = outputs.fedimint-dkg-cli-standalone;
        };
        checks = {
          clippy = outputs.ci.workspaceClippy;
        };

        devShells = {
          lint = flakeboxLib.mkLintShell { packages = [ ]; };
          default = flakeboxLib.mkDevShell {
            buildInputs = [
              pkgs.protobuf
              pkgs.openssl
            ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              pkgs.psmisc
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            nativeBuildInputs = [
              pkgs.pkg-config
            ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            PROTOC = "${pkgs.protobuf}/bin/protoc";
            PROTOC_INCLUDE = "${pkgs.protobuf}/include";
          };
        };
      });
}
