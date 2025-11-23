{
  description = "Gitlogue - A Rust-based Git tool";

  inputs = { nixpkgs.url = "github:NixOS/nixpkgs"; };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      }; # Define the system architecture here
      gitlogue = pkgs.stdenv.mkDerivation rec {
        pname = "gitlogue";
        version = "v0.3.0";

        src = pkgs.fetchurl {
          url =
            "https://github.com/unhappychoice/gitlogue/releases/download/${version}/gitlogue-${version}-x86_64-unknown-linux-gnu.tar.gz";
          sha256 = "09fxz8c21bq4mxmxz6yxxykwy8in17f36plaq526icwqd0wpa68a";
        };

        nativeBuildInputs = [ pkgs.curl pkgs.gnutar ];

        unpackPhase = ''
          mkdir -p $out/bin
          tar -xzf $src
          install -t $out/bin gitlogue
        '';

        meta = with pkgs.lib; {
          description = "Gitlogue - A Rust-based Git tool";
          license = licenses.mit;
          platforms = platforms.linux;
        };
      };
    in {

      packages.${system} = {
        gitlogue = gitlogue;
        default = gitlogue;
      };
    };
}
