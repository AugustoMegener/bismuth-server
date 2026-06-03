{ inputs, ... }:
{
  perSystem = { config, self', pkgs, lib, ... }: {
    devShells.default = pkgs.mkShell {
      name = "bismuth-server-shell";
      inputsFrom = [
        self'.devShells.rust
        config.pre-commit.devShell # See ./nix/modules/pre-commit.nix
      ];
      packages = with pkgs; [
        just
        nixd # Nix language server
        bacon
        fuse
        sqlite
        openssl
        pkg-config
      ];
      shellHook = ''
        export PATH="/home/kito/.cargo/bin:$PATH"
        export PATH="${config.packages.default}/bin:$PATH"
      '';
    };
  };
}
