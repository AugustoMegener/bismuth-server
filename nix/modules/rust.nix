{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    rust-project.crates."bismuth-server" = {
      path = ./../..;
      autoWire = [ "crate" "clippy" "doc" ];
      crane.args = {};
    };
    packages.default = self'.packages.bismuth-server;
  };
}
