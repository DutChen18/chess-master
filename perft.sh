cargo build --release

mkdir -p perft

depth=1

if [ ! -z $1 ]; then
    depth=$1
fi

while read -r fen; do
	echo "FEN: $fen"

	echo -e "position fen $fen moves ${@:2}\n go perft $depth" | stockfish | grep -v 'info string' | grep -v 'Stockfish developers' | sort | tail -n +3 > perft/sf
	echo -e "position fen $fen moves ${@:2}\n go perft $depth" | ./target/release/chess-master | sort > perft/cb

	diff perft/sf perft/cb
done < fens
