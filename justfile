build_plow:
    cargo build --release --bin plow && cp ./target/release/plow ~/bin && echo "Installed plow to ~/bin"

build_plow_dbg:
    cargo build --bin plow && cp ./target/debug/plow ~/bin && echo "Installed plow to ~/bin"