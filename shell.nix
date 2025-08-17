{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  shellHook = '''';

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    libpq
  ];
}
