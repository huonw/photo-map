Process and display the space-time location of a collection of
GPS-tagged photos, including heuristic grouping of interesting
sections.

Not designed for general use.

```
cargo build --release
./target/release/init-database -d points.db

find /path/to/photos > photos.txt
./target/release/extract-gps -d points.db -f photos.txt

./target/release/cluster -d points.db
./target/release/jsonify -d points.db -s web/summary.json -c web/clusters.json
./target/release/web # displayed at http://localhost:4444/
```

`cluster` and `jsonify` in particular have options for tweaking how
things are grouped.
