{ lib
, stdenv
, fetchFromGitHub
, meson
, ninja
, pkg-config
, gobject-introspection
, gtk-doc
, docbook-xsl-nons
, docbook_xml_dtd_43
, vala
, wayland-scanner
, wayland
, gtk4
}:

stdenv.mkDerivation rec {
  pname = "gtk4-layer-shell";
  version = "1.0.0";

  outputs = [ "out" "dev" "devdoc" ];
  outputBin = "devdoc";

  src = fetchFromGitHub {
    owner = "wmww";
    repo = "gtk4-layer-shell";
    rev = "v${version}";
    sha256 = "sha256-8bf7O/y9gQohd9ZLc7wygUeZxtU5RAsn1PW8pg0NcAc=";
  };

  strictDeps = true;

  depsBuildBuild = [
    pkg-config
  ];

  nativeBuildInputs = [
    meson
    ninja
    pkg-config
    gobject-introspection
    gtk-doc
    docbook-xsl-nons
    docbook_xml_dtd_43
    vala
    wayland-scanner
  ];

  buildInputs =  [
    wayland
    gtk4
  ];

  mesonFlags = [
    "-Ddocs=true"
    "-Dexamples=true"
  ];

  meta = with lib; {
    description = "A library to create panels and other desktop components for Wayland using the Layer Shell protocol and GTK4";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
