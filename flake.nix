{
  description = "Parametric cursor themes with reversible transitions";
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
          # Builds a theme crate, runs the workspace tests, then writes the theme.
          theme =
            crate: id:
            pkgs.rustPlatform.buildRustPackage {
              pname = crate;
              version = "0.1.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [ "-p" crate ];
              postInstall = ''
                $out/bin/${crate} --output "$out/share/icons/${id}"
                rm -r "$out/bin"
              '';
            };
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
          dot = theme "dot-cursors" "animated_dot_cursors";
          shapes = theme "shapes-cursors" "animated_shapes_cursors";
          default = self.packages.${system}.dot;
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
