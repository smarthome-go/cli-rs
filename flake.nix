{
  description = "shome: command-line interface for the Smarthome server";

  inputs = {
    # The dependencies require Rust >= 1.88
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

    # Cargo uses the SDK via the path `../sdk-rs`, which is outside of this flake and thus
    # invisible to Nix. The SDK is pinned here instead and placed at that path during the build.
    # To build against a local SDK checkout: `nix build --override-input sdk-rs path:../sdk-rs`
    sdk-rs = {
      url = "github:smarthome-go/sdk-rs";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      sdk-rs,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: rec {
        shome = pkgs.rustPlatform.buildRustPackage {
          pname = "shome";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;

          # Recreate the layout of the working copy so that `../sdk-rs` resolves
          postUnpack = ''
            cp -r ${sdk-rs} sdk-rs
            chmod -R u+w sdk-rs
          '';

          # Hashes the crates from `Cargo.lock` itself, so there is no `cargoHash` to keep in sync
          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            description = "Command-line interface for the Smarthome server";
            homepage = "https://github.com/smarthome-go/cli-rs";
            license = pkgs.lib.licenses.gpl2Only;
            mainProgram = "shome";
          };
        };
        default = shome;
      });
    };
}
