class Mausam < Formula
  desc "Beautiful weather in your terminal"
  homepage "https://github.com/codeptor/mausam"
  url "https://github.com/codeptor/mausam/archive/refs/tags/v1.0.4.tar.gz"
  sha256 ""
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mausam", shell_output("#{bin}/mausam --version")
  end
end
