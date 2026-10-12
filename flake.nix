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
          # `role = #rrggbb` lines of a palette file as an attrset.
          readPalette =
            file:
            lib.listToAttrs (
              lib.concatMap (
                line:
                let
                  entry = builtins.match "[[:space:]]*([^#=[:space:]]+)[[:space:]]*=[[:space:]]*(#[0-9a-fA-F]{6})[[:space:]]*" line;
                in
                lib.optional (entry != null) (lib.nameValuePair (lib.elemAt entry 0) (lib.elemAt entry 1))
              ) (lib.splitString "\n" (builtins.readFile file))
            );
          # Builds a theme crate in a bundled palette (crates/<style>/colors) with
          # any `colors` roles replaced, checks the crate in that palette, then
          # writes the theme. Override either with `.override { ... }`.
          theme = lib.makeOverridable (
            {
              style,
              id,
              palette ? "carburetor",
              colors ? { },
            }:
            let
              merged = readPalette ./crates/${style}/colors/${palette}.conf // colors;
              file = pkgs.writeTextDir "${palette}.conf" (
                lib.concatStrings (lib.mapAttrsToList (role: color: "${role} = ${color}\n") merged)
              );
              choice = if colors == { } then palette else "${file}/${palette}.conf";
            in
            pkgs.rustPlatform.buildRustPackage {
              pname = "${style}-cursors-${palette}";
              version = "0.1.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [
                "-p"
                "${style}-cursors"
              ];
              cargoTestFlags = [
                "-p"
                "${style}-cursors"
              ];
              CURSOR_PALETTE = choice;
              postInstall = ''
                $out/bin/${style}-cursors --palette ${choice} --output "$out/share/icons/${id}"
                rm -r "$out/bin"
              '';
              meta.license = lib.licenses.gpl3Plus;
            }
          );
          ids = {
            dot = "animated_dot_cursors";
            fluid = "animated_fluid_cursors";
            shapes = "animated_shapes_cursors";
          };
          # <style> is carburetor; <style>-<palette> installs as <id>_<palette>.
          themes = lib.concatMapAttrs (
            style: id:
            {
              ${style} = theme { inherit style id; };
            }
            // lib.genAttrs' [ "catppuccin" "gruvbox" ] (
              palette:
              lib.nameValuePair "${style}-${palette}" (theme {
                inherit style palette;
                id = "${id}_${palette}";
              })
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
