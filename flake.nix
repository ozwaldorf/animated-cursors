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
          inherit (pkgs) lib;
          # Builds a theme crate, runs the workspace tests, then writes the theme
          # in one of its bundled palettes (crates/<style>/colors).
          theme =
            style: id: palette:
            pkgs.rustPlatform.buildRustPackage {
              pname = "${style}-cursors-${palette}";
              version = "0.1.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [
                "-p"
                "${style}-cursors"
              ];
              postInstall = ''
                $out/bin/${style}-cursors --palette ${palette} --output "$out/share/icons/${id}"
                rm -r "$out/bin"
              '';
              meta.license = lib.licenses.gpl3Plus;
            };
          ids = {
            dot = "animated_dot_cursors";
            fluid = "animated_fluid_cursors";
            shapes = "animated_shapes_cursors";
          };
          # <style> is carburetor; <style>-<palette> installs as <id>_<palette>.
          themes = lib.concatMapAttrs (
            style: id:
            {
              ${style} = theme style id "carburetor";
            }
            // lib.genAttrs' [ "catppuccin" "gruvbox" ] (
              palette: lib.nameValuePair "${style}-${palette}" (theme style "${id}_${palette}" palette)
            )
          ) ids;
        in
        themes
        // {
          niri = pkgs.niri.overrideAttrs (old: {
            patches = (old.patches or [ ]) ++ [ ./patches/niri-cursor-transitions.patch ];
            doCheck = true;
            cargoTestFlags = [
              "--lib"
              "cursor::tests"
            ];
          });
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
