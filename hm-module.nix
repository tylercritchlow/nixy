{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.nixy;
  settingsFormat = pkgs.formats.toml { };
in
{
  options.programs.nixy = {
    enable = lib.mkEnableOption "Nixy, a Nix-native AI coding agent";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.nixy or (throw "nixy.package: nixpkgs does not provide a 'nixy' package. Set programs.nixy.package to inputs.nixy.packages.\${system}.default.");
      defaultText = lib.literalExpression "pkgs.nixy";
      description = "The Nixy package to install.";
    };

    settings = lib.mkOption {
      type = lib.types.attrsOf lib.types.anything;
      default = { };
      description = ''
        Configuration for Nixy, serialized to
        {file}`$XDG_CONFIG_HOME/nixy/config.toml`. Keybindings use
        the `modifier+key` string form, e.g. `"ctrl+c"`, `"alt+enter"`.
      '';
      example = {
        keybindings.app.quit = "ctrl+q";
        keybindings.editor.newline = "ctrl+j";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."nixy/config.toml" = lib.mkIf (cfg.settings != { }) {
      source = settingsFormat.generate "nixy-config.toml" cfg.settings;
    };
  };
}