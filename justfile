build_plow:
    cargo build --release --bin plow && cp ./target/debug/plow ~/bin && echo "Installed plow to ~/bin"