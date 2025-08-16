class Twin < Formula
  desc "Git worktree wrapper with side effects (symlinks and hooks)"
  homepage "https://github.com/yourusername/twin"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yourusername/twin/releases/download/v0.1.0/twin-macos-arm64.tar.gz"
      sha256 "HASH_WILL_BE_CALCULATED_AFTER_RELEASE"
    else
      url "https://github.com/yourusername/twin/releases/download/v0.1.0/twin-macos-x64.tar.gz"
      sha256 "HASH_WILL_BE_CALCULATED_AFTER_RELEASE"
    end
  end

  on_linux do
    url "https://github.com/yourusername/twin/releases/download/v0.1.0/twin-linux-x64.tar.gz"
    sha256 "HASH_WILL_BE_CALCULATED_AFTER_RELEASE"
  end

  depends_on "git"

  def install
    bin.install "twin"
    
    # Install sample config
    (share/"twin").install ".twin.toml.sample" if File.exist?(".twin.toml.sample")
    
    # Install completions if they exist
    bash_completion.install "completions/twin.bash" if File.exist?("completions/twin.bash")
    zsh_completion.install "completions/_twin" if File.exist?("completions/_twin")
    fish_completion.install "completions/twin.fish" if File.exist?("completions/twin.fish")
  end

  test do
    system "#{bin}/twin", "--version"
    assert_match "twin", shell_output("#{bin}/twin --help")
  end
end