class Codopsy < Formula
  desc "AST-level code quality analyzer for 35 languages"
  homepage "https://github.com/O6lvl4/codopsy"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/O6lvl4/codopsy/releases/download/v#{version}/codopsy-aarch64-apple-darwin.tar.gz"
    end
    on_intel do
      url "https://github.com/O6lvl4/codopsy/releases/download/v#{version}/codopsy-x86_64-apple-darwin.tar.gz"
    end
  end

  on_linux do
    url "https://github.com/O6lvl4/codopsy/releases/download/v#{version}/codopsy-x86_64-unknown-linux-gnu.tar.gz"
  end

  def install
    bin.install "codopsy"
  end

  test do
    assert_match "codopsy", shell_output("#{bin}/codopsy --version")
  end
end
