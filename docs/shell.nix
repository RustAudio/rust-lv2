{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    buildInputs = with pkgs.python3Packages; [recommonmark sphinx sphinx_rtd_theme];
}