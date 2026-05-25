# Maintainer: Charlie Lambart <charlielambart@gmail.com>
pkgname=omarchy-circle-search-git
pkgver=r14.g33392e9
pkgrel=1
pkgdesc="Circle to Search for Hyprland/Wayland — draw a region and visual search it"
arch=('x86_64' 'aarch64')
url="https://github.com/Lambchwrlie95/omarchy-circle-search"
license=('MIT')
depends=(
    'gtk4'
    'gtk4-layer-shell'
    'grim'
    'wl-clipboard'
    'python'
    'curl'
    'xdg-utils'
    'libnotify'
)
optdepends=('wtype: auto-paste for AI chat mode')
install=omarchy-circle-search-git.install
makedepends=('cargo' 'git')
provides=('omarchy-circle-search')
conflicts=('omarchy-circle-search')
source=("${pkgname}::git+${url}.git")
sha256sums=('SKIP')

pkgver() {
    cd "$srcdir/$pkgname"
    printf "r%s.g%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

prepare() {
    cd "$srcdir/$pkgname"
    export CARGO_HOME="$srcdir/cargo-home"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/$pkgname"
    export CARGO_HOME="$srcdir/cargo-home"
    cargo build --release --locked --offline
}

package() {
    cd "$srcdir/$pkgname"
    install -Dm755 "target/release/omarchy-circle-search" \
        "$pkgdir/usr/bin/omarchy-circle-search"
    install -Dm644 LICENSE \
        "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md \
        "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm755 omarchy-circle-search-setup.sh \
        "$pkgdir/usr/bin/omarchy-circle-search-setup"
}
