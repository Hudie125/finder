# Maintainer: Dima
pkgname=finder-bin
pkgver=2.0.0
pkgrel=1
pkgdesc="Monochrome Liquid Glass Spotlight Launcher with Voice (Whisper) and Cloud AI for Windows and Arch Linux"
arch=('x86_64')
url="https://github.com/dima/finder"
license=('MIT')
depends=('webkit2gtk-4.1' 'libayatana-appindicator' 'openssl' 'wmctrl')
makedepends=('rust' 'cargo' 'nodejs' 'npm')
provides=('finder')
conflicts=('finder')
source=("finder-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$srcdir/finder-$pkgver"
    npm install
    npm run build
}

package() {
    cd "$srcdir/finder-$pkgver"
    install -Dm755 "src-tauri/target/release/finder" "$pkgdir/usr/bin/finder"
    install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/finder.png"
    
    install -d "$pkgdir/usr/share/applications"
    cat << EOF > "$pkgdir/usr/share/applications/finder.desktop"
[Desktop Entry]
Name=Finder
Comment=Monochrome Liquid Glass Spotlight Launcher
Exec=/usr/bin/finder
Icon=finder
Terminal=false
Type=Application
Categories=Utility;System;
EOF
}
