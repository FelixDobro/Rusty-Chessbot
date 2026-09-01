cutechess-cli \
  -engine name="After" cmd="./engines/after" proto=uci \
  -engine name="Before" cmd="./engines/before" proto=uci \
  -each tc=10+0.1 \
  -games 5000 \
  -repeat \
  -openings file=UHO_2024_8mvs_big_+090_+109.epd format=epd order=random \
  -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 \
  -ratinginterval 8 \
  -concurrency 12