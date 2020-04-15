{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    buildInputs = with pkgs.python3Packages; [sphinx recommonmark sphinx_rtd_theme];
}