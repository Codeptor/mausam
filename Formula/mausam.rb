class Mausam < Formula
  desc "Beautiful weather in your terminal"
  homepage "https://github.com/codeptor/mausam"
  url "https://github.com/codeptor/mausam/archive/refs/tags/v1.0.3.tar.gz"
  sha256 "03966f9dbf3666249ab0aa46446bc40a16202ee3e748d5e24d4eb3c45a375df0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mausam", shell_output("#{bin}/mausam --version")
  end
end
