{ pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
    librarySystemDepends = with pkgs; [ clang ];
    buildInputs = with pkgs; [
        rustup
        llvmPackages.clang.cc.lib
    ];
    LIBCLANG_PATH = ''${pkgs.llvmPackages.clang.cc.lib}/lib'';
}