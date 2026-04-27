# Maintainer: Alex Trauthman
pkgname=bloatshot-git
pkgver=v0.1.0.r0.0f5f548
pkgrel=1
pkgdesc="A high-performance, hybrid CLI/GUI OCR screenshot utility for Hyprland"
arch=('x86_64')
url="https://github.com/Alex-Trauthman/bloatshot"
license=('MIT')
depends=('tesseract' 'tesseract-data-eng' 'tesseract-data-por' 'grim' 'slurp' 'wl-clipboard' 'libnotify')
makedepends=('rust' 'cargo' 'clang' 'git')
provides=('bloatshot')
conflicts=('bloatshot')
source=("git+https://github.com/Alex-Trauthman/bloatshot.git#branch=main")
sha256sums=('SKIP')

pkgver() {
  cd "bloatshot"
  git describe --long --tags | sed 's/\([^-]*-\)g/r\1/;s/-/./g'
}

build() {
  cd "bloatshot"
  cargo build --release --locked
}

package() {
  cd "bloatshot"
  install -Dm755 "target/release/bloatshot" "$pkgdir/usr/bin/bloatshot"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
