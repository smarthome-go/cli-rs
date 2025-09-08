{
  description = "Smarthome-CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }: {
    packages.x86_64-linux.shome = nixpkgs.legacyPackages.x86_64-linux.rustPlatform.buildRustPackage {
      pname = "shome";
      version = "1.0.0";
      src = ./.;
      cargoHash = "sha256-oqg53yRnKGG/uFq0ti1ymzHgrKE66psgTQ7nDdpl1ns=";
    };

    defaultPackage.x86_64-linux = self.packages.x86_64-linux.shome;
  };
}
