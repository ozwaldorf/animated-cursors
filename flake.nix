{
  description = "Minimal dot cursors designed for reversible transitions";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/104240a772428cc2e20d8fd86c9ddbb886bbaff2";
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          niri = pkgs.niri.overrideAttrs (old: {
            patches = (old.patches or [ ]) ++ [ ./patches/niri-cursor-transitions.patch ];
            doCheck = true;
            cargoTestFlags = [
              "--lib"
              "cursor::tests"
            ];
          });
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "animated-dot-cursors";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            postInstall = ''
              $out/bin/dot-cursors --output "$out/share/icons/animated_dot_cursors"
              rm -r "$out/bin"
            '';
          };
        }
      );
      devShells = eachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
            ];
          };
        }
      );
    };
}
