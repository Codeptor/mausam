class Mausam < Formula
  desc "Beautiful weather in your terminal"
  homepage "https://github.com/codeptor/mausam"
  url "https://github.com/codeptor/mausam/archive/refs/tags/v1.0.2.tar.gz"
  sha256 "746ff2324b114f56f144ecad57319ce3b6bf69b8d7908d742090d005196ce263"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "mausam", shell_output("#{bin}/mausam --version")
  end
end
