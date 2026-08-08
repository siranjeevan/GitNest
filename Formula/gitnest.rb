class Gitnest < Formula
  desc "One Workspace. Multiple Git Identities."
  homepage "https://github.com/siranjeevan/GitNest"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/siranjeevan/GitNest/releases/download/v1.0.0/gitnest-v1.0.0-macos-arm64.tar.gz"
      sha256 "5f53af68db047585be1fe71b80aa5ec9b5edccac7435f3dfd6faad91b920bf82"
    else
      url "https://github.com/siranjeevan/GitNest/releases/download/v1.0.0/gitnest-v1.0.0-macos-x86_64.tar.gz"
      sha256 "68c3c4d6864df3c51888f4ff7bc4046522c0764e66c61f237efb99e7ca8565fb"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/siranjeevan/GitNest/releases/download/v1.0.0/gitnest-v1.0.0-linux-x86_64.tar.gz"
      sha256 "686d6dd42c19273c52e4630a9e223d6a45970c32608bdccbeeb512c1b269beed"
    end
  end

  def install
    bin.install "gitnest"
  end

  test do
    assert_match "1.0.0", shell_output("#{bin}/gitnest --version")
  end
end
