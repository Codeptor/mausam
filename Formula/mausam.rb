class Mausam < Formula
  desc "Beautiful weather in your terminal"
  homepage "https://github.com/codeptor/mausam"
  url "https://github.com/codeptor/mausam/archive/refs/tags/v1.0.4.tar.gz"
  sha256 "e46b402ce65d1c2fbdca8266025da1e586cc60f9d3a7bb1260996c243b18718b"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mausam", shell_output("#{bin}/mausam --version")
  end
end
