{
  description = "Smarthome-CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
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
      cargoSha256 = "sha256-ItP2wDuz9XmJ3BUp+vZuqIU62OtVTBtG67ZqbaFLx+w=";
    };

    defaultPackage.x86_64-linux = self.packages.x86_64-linux.shome;
  };
}
