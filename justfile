build_plow:
    cargo build --bin plow && cp ./target/debug/plow ~/bin && echo "Installed plow to ~/bin"