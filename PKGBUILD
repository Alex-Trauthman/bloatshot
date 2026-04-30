# Maintainer: Alex Trauthman
pkgname=bloatshot-git
pkgver=v0.1.0.r7.5f53beb
pkgrel=1
pkgdesc="A high-performance, hybrid CLI/GUI OCR screenshot utility for Hyprland"
arch=('x86_64')
options=(!lto)
url="https://github.com/Alex-Trauthman/bloatshot"
license=('MIT')
depends=('grim' 'slurp' 'wl-clipboard' 'libnotify' 'onnxruntime')
makedepends=('rust' 'cargo' 'clang' 'git' 'mold')
provides=('bloatshot')
conflicts=('bloatshot')
source=("git+https://github.com/Alex-Trauthman/bloatshot.git#branch=main")
sha256sums=('SKIP')

pkgver() {
  cd "bloatshot"
  git describe --long --tags | sed 's/\([^-]*-\)g/r\1/;s/-/./g'
}

prepare() {
  cd "bloatshot"
  export ORT_STRATEGY=system
}

build() {
  cd "bloatshot"
  export ORT_STRATEGY=system
  export RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=mold -C target-cpu=native"
  cargo build --release --locked
}

package() {
  cd "bloatshot"
  install -Dm755 "target/release/bloatshot" "$pkgdir/usr/bin/bloatshot"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
