{ pkgs ? import <nixpkgs> {} }:
pkgs.stdenv.mkDerivation rec {
    name = "shell";

    nativeBuildInputs = with pkgs; [ pkgconfig openssl ];
    buildInputs = with pkgs.xorg; [ libxcb ];
    libs = (with pkgs.xorg; [ libX11 libXcursor libXrandr libXi ]) ++ (with pkgs; [ libglvnd ]);

    LD_LIBRARY_PATH = "${builtins.getEnv "LD_LIBRARY_PATH"}:${pkgs.lib.concatMapStringsSep ":" (lib: "${lib}/lib") libs}";
}
