all:
	cargo build --release
	cp target/release/chess-master chessbot
