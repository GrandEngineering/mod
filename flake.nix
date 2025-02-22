{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    engine.url = "github:GrandEngineering/engine";
    nixify.url = "github:rvolosatovs/nixify";
    engine.flake=false;
  };

  outputs = { self, nixpkgs,nixify,... }: {

    packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;

    packages.x86_64-linux.default = self.packages.x86_64-linux.hello;

  };
}
